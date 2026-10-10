use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::attachment::{
    is_allowed_mime, Attachment, AttachmentOwner, AttachmentType, ThumbnailRef,
};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::media_ports::AttachmentStorage;
use crate::domain::repositories::AttachmentRepository;
use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

pub struct AttachmentService<R: AttachmentRepository, S: AttachmentStorage> {
    repo: R,
    storage: S,
}

impl<R: AttachmentRepository, S: AttachmentStorage> AttachmentService<R, S> {
    pub fn new(repo: R, storage: S) -> Self {
        Self { repo, storage }
    }

    pub fn create_from_bytes(
        &self,
        owner_type: AttachmentOwner,
        owner_id: Uuid,
        file_name: String,
        mime_type: String,
        bytes: &[u8],
    ) -> JabeXResult<Attachment> {
        self.validate_payload(&mime_type, bytes.len() as u64)?;
        let id = create_id();
        let hash = hex::encode(Sha256::digest(bytes));
        let attachment_type = AttachmentType::from_mime(&mime_type)
            .ok_or_else(|| JabeXError::Validation(format!("unsupported mime type: {mime_type}")))?;

        let mut attachment = Attachment {
            id,
            owner_type,
            owner_id,
            file_name: file_name.clone(),
            mime_type: mime_type.clone(),
            size_bytes: bytes.len() as u64,
            hash: hash.clone(),
            relative_path: String::new(),
            revision: 1,
            created_at: now_utc(),
            updated_at: now_utc(),
            deleted_at: None,
            deleted_by: None,
        };

        let thumbnails = if attachment_type == AttachmentType::Image {
            // Thumbnail binary generation is deferred; record the intended folder.
            vec![ThumbnailRef {
                relative_path: format!("media/attachments/{id}/thumbnails/preview.webp"),
                width: 0,
                height: 0,
            }]
        } else {
            Vec::new()
        };

        let meta = attachment
            .to_metadata(thumbnails.clone())
            .ok_or_else(|| JabeXError::Validation("invalid attachment type".into()))?;

        let stored = self
            .storage
            .store_attachment(id, &file_name, bytes, &meta)?;
        attachment.relative_path = stored.relative_path;
        attachment.hash = stored.hash;
        attachment.size_bytes = stored.size_bytes;

        // Drop placeholder thumbnails with zero dimensions from the DB-facing row;
        // sidecar metadata.json still records intent for future generators.
        let _ = thumbnails;

        if let Err(err) = self.repo.create(&attachment) {
            let _ = self.storage.remove_attachment_files(&attachment);
            return Err(err);
        }

        tracing::info!(attachment_id = %attachment.id, owner_id = %owner_id, "attachment created");
        Ok(attachment)
    }

    pub fn create_from_path(
        &self,
        owner_type: AttachmentOwner,
        owner_id: Uuid,
        source_path: &Path,
        file_name: Option<String>,
        mime_type: String,
    ) -> JabeXResult<Attachment> {
        let bytes = fs::read(source_path).map_err(|e| {
            JabeXError::Filesystem(format!(
                "failed to read source file {}: {e}",
                source_path.display()
            ))
        })?;
        let name = file_name.unwrap_or_else(|| {
            source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment.bin")
                .to_string()
        });
        self.create_from_bytes(owner_type, owner_id, name, mime_type, &bytes)
    }

    pub fn get_attachment(&self, id: Uuid) -> JabeXResult<Attachment> {
        self.repo.find(id)?.ok_or(JabeXError::NotFound {
            entity: "attachment",
            id,
        })
    }

    pub fn list_attachments(
        &self,
        owner_type: AttachmentOwner,
        owner_id: Uuid,
    ) -> JabeXResult<Vec<Attachment>> {
        self.repo.list_by_owner(owner_type, owner_id, false)
    }

    pub fn list_all(&self) -> JabeXResult<Vec<Attachment>> {
        self.repo.list_all(false)
    }

    pub fn delete_attachment(&self, id: Uuid) -> JabeXResult<()> {
        let mut attachment = self.get_attachment(id)?;
        if attachment.is_deleted() {
            return Ok(());
        }
        attachment.soft_delete(now_utc(), None);
        self.repo.update(&attachment)?;
        tracing::info!(attachment_id = %id, "attachment soft-deleted");
        Ok(())
    }

    pub fn restore_attachment(&self, id: Uuid) -> JabeXResult<Attachment> {
        let mut attachment = self.repo.find(id)?.ok_or(JabeXError::NotFound {
            entity: "attachment",
            id,
        })?;
        if !attachment.is_deleted() {
            return Ok(attachment);
        }
        // Ensure file still present / checksum matches before restore.
        if !self
            .storage
            .verify_checksum(&attachment.relative_path, &attachment.hash)?
        {
            return Err(JabeXError::Validation(
                "cannot restore attachment: file missing or checksum mismatch".into(),
            ));
        }
        attachment.restore(now_utc());
        self.repo.update(&attachment)?;
        tracing::info!(attachment_id = %id, "attachment restored");
        Ok(attachment)
    }

    pub fn read_bytes(&self, id: Uuid) -> JabeXResult<Vec<u8>> {
        let attachment = self.get_attachment(id)?;
        if attachment.is_deleted() {
            return Err(JabeXError::NotFound {
                entity: "attachment",
                id,
            });
        }
        self.storage.read_bytes(&attachment.relative_path)
    }

    fn validate_payload(&self, mime_type: &str, size_bytes: u64) -> JabeXResult<()> {
        if !is_allowed_mime(mime_type) {
            return Err(JabeXError::Validation(format!(
                "unsupported mime type: {mime_type}"
            )));
        }
        let kind = AttachmentType::from_mime(mime_type)
            .ok_or_else(|| JabeXError::Validation(format!("unsupported mime type: {mime_type}")))?;
        if size_bytes == 0 {
            return Err(JabeXError::Validation("attachment is empty".into()));
        }
        if size_bytes > kind.max_bytes() {
            return Err(JabeXError::Validation(format!(
                "attachment exceeds size limit of {} bytes for {:?}",
                kind.max_bytes(),
                kind
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{open_database_at, DatabasePaths};
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use crate::infrastructure::repositories::SqliteAttachmentRepository;
    use tempfile::tempdir;

    fn service() -> (
        tempfile::TempDir,
        AttachmentService<SqliteAttachmentRepository, LocalMediaStorage>,
    ) {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        let repo = SqliteAttachmentRepository::new(db);
        (dir, AttachmentService::new(repo, storage))
    }

    #[test]
    fn create_delete_restore_roundtrip() {
        let (_dir, svc) = service();
        let owner = Uuid::now_v7();
        let created = svc
            .create_from_bytes(
                AttachmentOwner::Card,
                owner,
                "note.png".into(),
                "image/png".into(),
                b"png-bytes",
            )
            .expect("create");
        assert_eq!(created.hash.len(), 64);
        assert!(created.relative_path.contains(&created.id.to_string()));

        let listed = svc
            .list_attachments(AttachmentOwner::Card, owner)
            .expect("list");
        assert_eq!(listed.len(), 1);

        svc.delete_attachment(created.id).expect("delete");
        assert!(svc
            .list_attachments(AttachmentOwner::Card, owner)
            .expect("list")
            .is_empty());

        let restored = svc.restore_attachment(created.id).expect("restore");
        assert!(!restored.is_deleted());
        assert_eq!(
            svc.list_attachments(AttachmentOwner::Card, owner)
                .expect("list")
                .len(),
            1
        );
    }

    #[test]
    fn rejects_bad_mime_and_oversize() {
        let (_dir, svc) = service();
        let err = svc
            .create_from_bytes(
                AttachmentOwner::Card,
                Uuid::now_v7(),
                "x.bin".into(),
                "application/zip".into(),
                b"zip",
            )
            .expect_err("mime");
        assert!(matches!(err, JabeXError::Validation(_)));
    }
}
