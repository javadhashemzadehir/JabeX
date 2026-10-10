use std::collections::HashSet;

use uuid::Uuid;

use crate::domain::attachment::{Attachment, AttachmentType};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::media_library::{
    MediaIntegrityReport, MediaLibraryFilter, MediaLibraryItem, MediaLibraryStats, MediaPurgeResult,
};
use crate::domain::media_ports::AttachmentStorage;
use crate::domain::repositories::AttachmentRepository;

/// Profile-scoped media library: browse, stats, integrity, and purge (ADR 010 GC).
pub struct MediaLibraryService<R: AttachmentRepository, S: AttachmentStorage> {
    repo: R,
    storage: S,
}

impl<R: AttachmentRepository, S: AttachmentStorage> MediaLibraryService<R, S> {
    pub fn new(repo: R, storage: S) -> Self {
        Self { repo, storage }
    }

    pub fn list_library(&self, filter: &MediaLibraryFilter) -> JabeXResult<Vec<MediaLibraryItem>> {
        let rows = self.repo.list_all(filter.include_deleted)?;
        let query = filter
            .query
            .as_ref()
            .map(|q| q.trim().to_ascii_lowercase())
            .filter(|q| !q.is_empty());

        let mut items: Vec<MediaLibraryItem> = rows
            .into_iter()
            .filter(|row| {
                if !filter.include_deleted && row.is_deleted() {
                    return false;
                }
                let kind = row.attachment_type();
                if let Some(wanted) = filter.attachment_type {
                    if kind != Some(wanted) {
                        return false;
                    }
                }
                if let Some(ref q) = query {
                    if !row.file_name.to_ascii_lowercase().contains(q) {
                        return false;
                    }
                }
                true
            })
            .map(|attachment| self.to_item(attachment))
            .collect();

        items.sort_by_key(|a| std::cmp::Reverse(a.attachment.created_at));
        Ok(items)
    }

    pub fn get_stats(&self) -> JabeXResult<MediaLibraryStats> {
        let rows = self.repo.list_all(true)?;
        let mut stats = MediaLibraryStats::empty();
        let known: HashSet<Uuid> = rows.iter().map(|r| r.id).collect();

        for row in &rows {
            let missing = !self.storage.file_exists(&row.relative_path);
            if missing {
                stats.missing_file_count = stats.missing_file_count.saturating_add(1);
            }
            if row.is_deleted() {
                stats.deleted_count = stats.deleted_count.saturating_add(1);
                stats.deleted_bytes = stats.deleted_bytes.saturating_add(row.size_bytes);
            } else {
                stats.total_count = stats.total_count.saturating_add(1);
                stats.total_bytes = stats.total_bytes.saturating_add(row.size_bytes);
                match row.attachment_type() {
                    Some(AttachmentType::Image) => {
                        stats.image_count = stats.image_count.saturating_add(1)
                    }
                    Some(AttachmentType::Audio) => {
                        stats.audio_count = stats.audio_count.saturating_add(1)
                    }
                    Some(AttachmentType::Video) => {
                        stats.video_count = stats.video_count.saturating_add(1)
                    }
                    Some(AttachmentType::Pdf) => {
                        stats.pdf_count = stats.pdf_count.saturating_add(1)
                    }
                    None => {}
                }
            }
        }

        let on_disk = self.storage.list_stored_attachment_ids()?;
        stats.orphan_dir_count =
            on_disk.into_iter().filter(|id| !known.contains(id)).count() as u32;

        Ok(stats)
    }

    /// Permanently delete a soft-deleted attachment (files + row).
    pub fn purge_attachment(&self, id: Uuid) -> JabeXResult<MediaPurgeResult> {
        let attachment = self.repo.find(id)?.ok_or(JabeXError::NotFound {
            entity: "attachment",
            id,
        })?;
        if !attachment.is_deleted() {
            return Err(JabeXError::Validation(
                "only soft-deleted attachments can be purged; delete first".into(),
            ));
        }
        let freed = attachment.size_bytes;
        self.storage.remove_attachment_files(&attachment)?;
        self.repo.hard_delete(id)?;
        tracing::info!(attachment_id = %id, freed_bytes = freed, "attachment purged");
        Ok(MediaPurgeResult {
            purged_count: 1,
            freed_bytes: freed,
        })
    }

    /// Permanently delete all soft-deleted attachments.
    pub fn purge_deleted(&self) -> JabeXResult<MediaPurgeResult> {
        let rows = self.repo.list_all(true)?;
        let mut purged_count = 0u32;
        let mut freed_bytes = 0u64;
        for attachment in rows.into_iter().filter(|a| a.is_deleted()) {
            let size = attachment.size_bytes;
            let id = attachment.id;
            if let Err(err) = self.storage.remove_attachment_files(&attachment) {
                tracing::warn!(attachment_id = %id, error = %err, "purge file cleanup failed");
            }
            self.repo.hard_delete(id)?;
            purged_count = purged_count.saturating_add(1);
            freed_bytes = freed_bytes.saturating_add(size);
        }
        tracing::info!(purged_count, freed_bytes, "soft-deleted attachments purged");
        Ok(MediaPurgeResult {
            purged_count,
            freed_bytes,
        })
    }

    /// Remove orphan attachment directories that have no DB row.
    pub fn remove_orphan_dirs(&self) -> JabeXResult<u32> {
        let rows = self.repo.list_all(true)?;
        let known: HashSet<Uuid> = rows.into_iter().map(|r| r.id).collect();
        let on_disk = self.storage.list_stored_attachment_ids()?;
        let mut removed = 0u32;
        for id in on_disk.into_iter().filter(|id| !known.contains(id)) {
            let stub = Attachment {
                id,
                owner_type: crate::domain::attachment::AttachmentOwner::Card,
                owner_id: id,
                file_name: String::new(),
                mime_type: String::new(),
                size_bytes: 0,
                hash: String::new(),
                relative_path: format!("media/attachments/{id}/original.bin"),
                revision: 0,
                created_at: crate::shared::time::now_utc(),
                updated_at: crate::shared::time::now_utc(),
                deleted_at: None,
                deleted_by: None,
            };
            self.storage.remove_attachment_files(&stub)?;
            removed = removed.saturating_add(1);
        }
        tracing::info!(removed, "orphan attachment directories removed");
        Ok(removed)
    }

    pub fn verify_integrity(&self) -> JabeXResult<MediaIntegrityReport> {
        let rows = self.repo.list_all(true)?;
        let known: HashSet<Uuid> = rows.iter().map(|r| r.id).collect();
        let mut missing_files = Vec::new();
        let mut checksum_mismatches = Vec::new();
        let checked = rows.len() as u32;

        for row in &rows {
            if !self.storage.file_exists(&row.relative_path) {
                missing_files.push(row.id);
                continue;
            }
            match self.storage.verify_checksum(&row.relative_path, &row.hash) {
                Ok(true) => {}
                Ok(false) => checksum_mismatches.push(row.id),
                Err(err) => {
                    tracing::warn!(attachment_id = %row.id, error = %err, "checksum verify failed");
                    checksum_mismatches.push(row.id);
                }
            }
        }

        let orphan_dirs = self
            .storage
            .list_stored_attachment_ids()?
            .into_iter()
            .filter(|id| !known.contains(id))
            .collect();

        Ok(MediaIntegrityReport {
            checked,
            missing_files,
            checksum_mismatches,
            orphan_dirs,
        })
    }

    fn to_item(&self, attachment: Attachment) -> MediaLibraryItem {
        let attachment_type = attachment.attachment_type();
        let file_missing = !self.storage.file_exists(&attachment.relative_path);
        MediaLibraryItem {
            attachment,
            attachment_type,
            file_missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::AttachmentService;
    use crate::domain::attachment::AttachmentOwner;
    use crate::infrastructure::database::{open_database_at, DatabasePaths};
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use crate::infrastructure::repositories::SqliteAttachmentRepository;
    use tempfile::tempdir;

    fn services() -> (
        tempfile::TempDir,
        AttachmentService<SqliteAttachmentRepository, LocalMediaStorage>,
        MediaLibraryService<SqliteAttachmentRepository, LocalMediaStorage>,
    ) {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        let attachments =
            AttachmentService::new(SqliteAttachmentRepository::new(db.clone()), storage.clone());
        let library = MediaLibraryService::new(SqliteAttachmentRepository::new(db), storage);
        (dir, attachments, library)
    }

    #[test]
    fn lists_stats_purge_and_integrity() {
        let (_dir, attachments, library) = services();
        let owner = Uuid::now_v7();
        let created = attachments
            .create_from_bytes(
                AttachmentOwner::Card,
                owner,
                "clip.webm".into(),
                "audio/webm".into(),
                b"voice-bytes",
            )
            .expect("create");

        let active = library
            .list_library(&MediaLibraryFilter::default())
            .expect("list");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].attachment.id, created.id);
        assert!(!active[0].file_missing);

        let stats = library.get_stats().expect("stats");
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.audio_count, 1);
        assert_eq!(stats.deleted_count, 0);

        attachments
            .delete_attachment(created.id)
            .expect("soft delete");
        let with_trash = library
            .list_library(&MediaLibraryFilter {
                include_deleted: true,
                ..Default::default()
            })
            .expect("trash list");
        assert_eq!(with_trash.len(), 1);
        assert!(with_trash[0].attachment.is_deleted());

        let report = library.verify_integrity().expect("verify");
        assert_eq!(report.checked, 1);
        assert!(report.healthy());

        let purge = library.purge_deleted().expect("purge");
        assert_eq!(purge.purged_count, 1);
        assert!(purge.freed_bytes > 0);
        assert!(library
            .list_library(&MediaLibraryFilter {
                include_deleted: true,
                ..Default::default()
            })
            .expect("empty")
            .is_empty());
    }

    #[test]
    fn rejects_purge_of_active_attachment() {
        let (_dir, attachments, library) = services();
        let created = attachments
            .create_from_bytes(
                AttachmentOwner::Card,
                Uuid::now_v7(),
                "a.png".into(),
                "image/png".into(),
                b"png",
            )
            .expect("create");
        let err = library.purge_attachment(created.id).expect_err("active");
        assert!(matches!(err, JabeXError::Validation(_)));
    }
}
