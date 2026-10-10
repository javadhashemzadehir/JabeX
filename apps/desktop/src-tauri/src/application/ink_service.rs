use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::ink::{InkDocument, InkDocumentRef, InkOwner};
use crate::domain::media_ports::InkStorage;
use crate::domain::repositories::InkDocumentRepository;
use crate::shared::time::now_utc;

pub struct InkService<R: InkDocumentRepository, S: InkStorage> {
    repo: R,
    storage: S,
}

impl<R: InkDocumentRepository, S: InkStorage> InkService<R, S> {
    pub fn new(repo: R, storage: S) -> Self {
        Self { repo, storage }
    }

    pub fn create_document(
        &self,
        owner_type: InkOwner,
        owner_id: Uuid,
        width: u32,
        height: u32,
    ) -> JabeXResult<(InkDocumentRef, InkDocument)> {
        let document = InkDocument::new(width, height);
        document.validate().map_err(JabeXError::Validation)?;
        let relative_path = self.storage.write_document(&document)?;
        let content_hash = hash_document(&document)?;
        let reference = InkDocumentRef::new(
            document.id,
            owner_type,
            owner_id,
            relative_path,
            document.width,
            document.height,
            content_hash,
        );
        self.repo.create(&reference)?;
        tracing::info!(ink_id = %document.id, owner_id = %owner_id, "ink document created");
        Ok((reference, document))
    }

    pub fn get_reference(&self, id: Uuid) -> JabeXResult<InkDocumentRef> {
        self.repo
            .find(id)?
            .ok_or(JabeXError::NotFound {
                entity: "ink_document",
                id,
            })
    }

    pub fn get_document(&self, id: Uuid) -> JabeXResult<(InkDocumentRef, InkDocument)> {
        let reference = self.get_reference(id)?;
        if reference.is_deleted() {
            return Err(JabeXError::NotFound {
                entity: "ink_document",
                id,
            });
        }
        let document = self.storage.read_document(&reference.relative_path)?;
        document.validate().map_err(JabeXError::Validation)?;
        Ok((reference, document))
    }

    pub fn list_by_owner(
        &self,
        owner_type: InkOwner,
        owner_id: Uuid,
    ) -> JabeXResult<Vec<InkDocumentRef>> {
        self.repo.list_by_owner(owner_type, owner_id, false)
    }

    pub fn list_all(&self) -> JabeXResult<Vec<InkDocumentRef>> {
        self.repo.list_all(false)
    }

    /// Persist a full document payload (debounced from the UI).
    pub fn save_document(&self, document: InkDocument) -> JabeXResult<(InkDocumentRef, InkDocument)> {
        document.validate().map_err(JabeXError::Validation)?;
        let mut reference = self.get_reference(document.id)?;
        if reference.is_deleted() {
            return Err(JabeXError::NotFound {
                entity: "ink_document",
                id: document.id,
            });
        }
        let relative_path = self.storage.write_document(&document)?;
        reference.relative_path = relative_path;
        reference.width = document.width;
        reference.height = document.height;
        reference.content_hash = hash_document(&document)?;
        reference.updated_at = now_utc();
        reference.bump_revision();
        self.repo.update(&reference)?;
        Ok((reference, document))
    }

    pub fn delete_document(&self, id: Uuid) -> JabeXResult<()> {
        let mut reference = self.get_reference(id)?;
        if reference.is_deleted() {
            return Ok(());
        }
        reference.soft_delete(now_utc(), None);
        self.repo.update(&reference)?;
        tracing::info!(ink_id = %id, "ink document soft-deleted");
        Ok(())
    }

}

fn hash_document(document: &InkDocument) -> JabeXResult<String> {
    let bytes = serde_json::to_vec(document)
        .map_err(|e| JabeXError::Internal(format!("failed to hash ink: {e}")))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ink::{InkTool, Point, Stroke};
    use crate::infrastructure::database::{open_database_at, DatabasePaths};
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use crate::infrastructure::repositories::SqliteInkDocumentRepository;
    use tempfile::tempdir;

    #[test]
    fn create_save_and_reload() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let svc = InkService::new(
            SqliteInkDocumentRepository::new(db),
            LocalMediaStorage::new(paths),
        );
        let owner = Uuid::now_v7();
        let (reference, mut doc) = svc
            .create_document(InkOwner::Card, owner, 640, 480)
            .expect("create");

        let mut stroke = Stroke::new(InkTool::Pen, "#111827", 2.0);
        stroke.points.push(Point {
            x: 1.0,
            y: 2.0,
            pressure: 0.5,
            timestamp: 0,
        });
        doc.layers[0].strokes.push(stroke);
        let (ref2, _saved) = svc.save_document(doc).expect("save");
        let (_ref3, loaded) = svc.get_document(reference.id).expect("get");
        assert_eq!(loaded.layers[0].strokes.len(), 1);
        assert_eq!(ref2.content_hash.len(), 64);
    }
}
