use uuid::Uuid;

use super::attachment::{Attachment, AttachmentMetadata};
use super::errors::JabeXResult;
use super::ink::InkDocument;

/// Port for profile-scoped media files. Domain never sees absolute paths.
pub trait AttachmentStorage: Send + Sync {
    /// Persist bytes under `media/attachments/{id}/` and return relative path + hash.
    fn store_attachment(
        &self,
        attachment_id: Uuid,
        file_name: &str,
        bytes: &[u8],
        metadata: &AttachmentMetadata,
    ) -> JabeXResult<StoredAttachmentFiles>;

    fn remove_attachment_files(&self, attachment: &Attachment) -> JabeXResult<()>;

    fn verify_checksum(&self, relative_path: &str, expected_hash: &str) -> JabeXResult<bool>;

    fn read_bytes(&self, relative_path: &str) -> JabeXResult<Vec<u8>>;

    fn file_exists(&self, relative_path: &str) -> bool;

    /// UUID folder names present under `media/attachments/`.
    fn list_stored_attachment_ids(&self) -> JabeXResult<Vec<Uuid>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAttachmentFiles {
    pub relative_path: String,
    pub hash: String,
    pub size_bytes: u64,
    pub metadata_relative_path: String,
}

/// Port for ink JSON documents on disk.
pub trait InkStorage: Send + Sync {
    fn write_document(&self, document: &InkDocument) -> JabeXResult<String>;
    fn read_document(&self, relative_path: &str) -> JabeXResult<InkDocument>;
    fn remove_document(&self, relative_path: &str) -> JabeXResult<()>;
}
