use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;

use crate::domain::attachment::{Attachment, AttachmentMetadata};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::ink::InkDocument;
use crate::domain::language_assist::PronunciationAudioStorage;
use crate::domain::media_ports::{AttachmentStorage, InkStorage, StoredAttachmentFiles};
use crate::infrastructure::database::DatabasePaths;

/// Local profile-scoped media store (ADR 010).
///
/// Layout:
/// ```text
/// media/attachments/{uuid}/original.ext
/// media/attachments/{uuid}/metadata.json
/// media/attachments/{uuid}/thumbnails/
/// media/ink/{uuid}.json
/// media/audio/pronunciation/{card_uuid}.mp3   # optional language-assist cache
/// ```
#[derive(Clone)]
pub struct LocalMediaStorage {
    paths: DatabasePaths,
}

impl LocalMediaStorage {
    pub fn new(paths: DatabasePaths) -> Self {
        Self { paths }
    }

    pub fn absolute_path(&self, relative: &str) -> PathBuf {
        self.paths.root.join(normalize_relative(relative))
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.absolute_path(relative).exists()
    }

    fn ensure_under_media(&self, absolute: &Path) -> JabeXResult<()> {
        let media_root = self
            .paths
            .media_root
            .canonicalize()
            .unwrap_or_else(|_| self.paths.media_root.clone());
        let abs = absolute
            .canonicalize()
            .unwrap_or_else(|_| absolute.to_path_buf());
        if !abs.starts_with(&media_root) && !absolute.starts_with(&self.paths.media_root) {
            return Err(JabeXError::Filesystem(
                "refusing path outside profile media root".into(),
            ));
        }
        Ok(())
    }

    fn attachment_dir(&self, attachment_id: Uuid) -> PathBuf {
        self.paths.attachments_dir.join(attachment_id.to_string())
    }

    fn extension_from_name(file_name: &str) -> String {
        Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or_else(|| "bin".into())
    }
}

impl AttachmentStorage for LocalMediaStorage {
    fn store_attachment(
        &self,
        attachment_id: Uuid,
        file_name: &str,
        bytes: &[u8],
        metadata: &AttachmentMetadata,
    ) -> JabeXResult<StoredAttachmentFiles> {
        let dir = self.attachment_dir(attachment_id);
        let thumbnails = dir.join("thumbnails");
        fs::create_dir_all(&thumbnails).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create attachment dir: {e}"))
        })?;

        let ext = Self::extension_from_name(file_name);
        let original_name = format!("original.{ext}");
        let absolute = dir.join(&original_name);
        self.ensure_under_media(&absolute)?;

        let hash = hex::encode(Sha256::digest(bytes));
        let mut file = fs::File::create(&absolute)
            .map_err(|e| JabeXError::Filesystem(format!("failed to create media file: {e}")))?;
        file.write_all(bytes)
            .map_err(|e| JabeXError::Filesystem(format!("failed to write media file: {e}")))?;

        let relative_path = format!(
            "media/attachments/{}/{original_name}",
            attachment_id
        );
        let metadata_relative = format!("media/attachments/{}/metadata.json", attachment_id);

        let mut meta = metadata.clone();
        meta.hash = hash.clone();
        meta.relative_path = relative_path.clone();
        meta.size_bytes = bytes.len() as u64;

        let meta_bytes = serde_json::to_vec_pretty(&meta)
            .map_err(|e| JabeXError::Filesystem(format!("failed to encode metadata: {e}")))?;
        fs::write(dir.join("metadata.json"), meta_bytes)
            .map_err(|e| JabeXError::Filesystem(format!("failed to write metadata: {e}")))?;

        info!(%attachment_id, %hash, "attachment stored");
        Ok(StoredAttachmentFiles {
            relative_path,
            hash,
            size_bytes: bytes.len() as u64,
            metadata_relative_path: metadata_relative,
        })
    }

    fn remove_attachment_files(&self, attachment: &Attachment) -> JabeXResult<()> {
        let dir = self.attachment_dir(attachment.id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| {
                JabeXError::Filesystem(format!("failed to remove attachment files: {e}"))
            })?;
        }
        Ok(())
    }

    fn verify_checksum(&self, relative_path: &str, expected_hash: &str) -> JabeXResult<bool> {
        let bytes = self.read_bytes(relative_path)?;
        let actual = hex::encode(Sha256::digest(&bytes));
        Ok(actual.eq_ignore_ascii_case(expected_hash))
    }

    fn read_bytes(&self, relative_path: &str) -> JabeXResult<Vec<u8>> {
        let absolute = self.absolute_path(relative_path);
        self.ensure_under_media(&absolute)?;
        fs::read(&absolute).map_err(|e| {
            JabeXError::Filesystem(format!("failed to read media file {}: {e}", absolute.display()))
        })
    }

    fn file_exists(&self, relative_path: &str) -> bool {
        self.exists(relative_path)
    }

    fn list_stored_attachment_ids(&self) -> JabeXResult<Vec<Uuid>> {
        let root = &self.paths.attachments_dir;
        if !root.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        let entries = fs::read_dir(root).map_err(|e| {
            JabeXError::Filesystem(format!("failed to list attachment dirs: {e}"))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                JabeXError::Filesystem(format!("failed to read attachment dir entry: {e}"))
            })?;
            if !entry
                .file_type()
                .map(|t| t.is_dir())
                .unwrap_or(false)
            {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if let Ok(id) = Uuid::parse_str(name) {
                ids.push(id);
            }
        }
        ids.sort_unstable();
        Ok(ids)
    }
}

impl PronunciationAudioStorage for LocalMediaStorage {
    fn store_pronunciation(
        &self,
        card_id: Uuid,
        bytes: &[u8],
        extension: &str,
    ) -> JabeXResult<String> {
        let ext = extension
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let ext = if ext.is_empty()
            || !ext.chars().all(|c| c.is_ascii_alphanumeric())
        {
            "mp3".into()
        } else {
            ext
        };

        let dir = self.paths.audio_dir.join("pronunciation");
        fs::create_dir_all(&dir).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create pronunciation audio dir: {e}"))
        })?;

        let relative = format!("media/audio/pronunciation/{card_id}.{ext}");
        let absolute = self.absolute_path(&relative);
        self.ensure_under_media(&absolute)?;

        let mut file = fs::File::create(&absolute).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create pronunciation file: {e}"))
        })?;
        file.write_all(bytes).map_err(|e| {
            JabeXError::Filesystem(format!("failed to write pronunciation file: {e}"))
        })?;

        info!(%card_id, %relative, "pronunciation audio cached");
        Ok(relative)
    }

    fn remove_pronunciation(&self, relative_path: &str) -> JabeXResult<()> {
        if relative_path.trim().is_empty() {
            return Ok(());
        }
        let absolute = self.absolute_path(relative_path);
        if absolute.exists() {
            self.ensure_under_media(&absolute)?;
            fs::remove_file(&absolute).map_err(|e| {
                JabeXError::Filesystem(format!("failed to remove pronunciation audio: {e}"))
            })?;
        }
        Ok(())
    }

    fn pronunciation_exists(&self, relative_path: &str) -> bool {
        !relative_path.trim().is_empty() && self.exists(relative_path)
    }
}

impl InkStorage for LocalMediaStorage {
    fn write_document(&self, document: &InkDocument) -> JabeXResult<String> {
        fs::create_dir_all(&self.paths.ink_dir)
            .map_err(|e| JabeXError::Filesystem(format!("failed to create ink dir: {e}")))?;
        let relative = format!("media/ink/{}.json", document.id);
        let absolute = self.absolute_path(&relative);
        self.ensure_under_media(&absolute)?;
        let bytes = serde_json::to_vec_pretty(document)
            .map_err(|e| JabeXError::Filesystem(format!("failed to encode ink: {e}")))?;
        fs::write(&absolute, bytes)
            .map_err(|e| JabeXError::Filesystem(format!("failed to write ink: {e}")))?;
        Ok(relative)
    }

    fn read_document(&self, relative_path: &str) -> JabeXResult<InkDocument> {
        let bytes = self.read_bytes(relative_path)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| JabeXError::Filesystem(format!("invalid ink document JSON: {e}")))
    }

    fn remove_document(&self, relative_path: &str) -> JabeXResult<()> {
        let absolute = self.absolute_path(relative_path);
        if absolute.exists() {
            self.ensure_under_media(&absolute)?;
            fs::remove_file(&absolute)
                .map_err(|e| JabeXError::Filesystem(format!("failed to remove ink: {e}")))?;
        }
        Ok(())
    }
}

fn normalize_relative(relative: &str) -> String {
    relative.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attachment::{AttachmentOwner, AttachmentType, ThumbnailRef};
    use crate::domain::media_ports::AttachmentStorage;
    use crate::shared::time::now_utc;
    use tempfile::tempdir;

    #[test]
    fn stores_attachment_and_verifies_checksum() {
        let dir = tempdir().expect("tempdir");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        let id = Uuid::now_v7();
        let now = now_utc();
        let meta = AttachmentMetadata {
            id,
            owner_type: AttachmentOwner::Card,
            owner_id: Uuid::now_v7(),
            file_name: "photo.png".into(),
            mime_type: "image/png".into(),
            attachment_type: AttachmentType::Image,
            size_bytes: 0,
            hash: String::new(),
            relative_path: String::new(),
            thumbnails: Vec::<ThumbnailRef>::new(),
            created_at: now,
            updated_at: now,
        };
        let stored = storage
            .store_attachment(id, "photo.png", b"hello-image", &meta)
            .expect("store");
        assert!(storage.exists(&stored.relative_path));
        assert!(storage
            .verify_checksum(&stored.relative_path, &stored.hash)
            .expect("verify"));
        assert!(!storage
            .verify_checksum(&stored.relative_path, "deadbeef")
            .expect("verify bad"));
    }

    #[test]
    fn writes_and_reads_ink_document() {
        let dir = tempdir().expect("tempdir");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        let doc = InkDocument::new(320, 240);
        let relative = storage.write_document(&doc).expect("write");
        let loaded = storage.read_document(&relative).expect("read");
        assert_eq!(loaded.id, doc.id);
        assert_eq!(loaded.width, 320);
    }

    #[test]
    fn stores_pronunciation_under_media_audio() {
        let dir = tempdir().expect("tempdir");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let storage = LocalMediaStorage::new(paths);
        let card_id = Uuid::now_v7();
        let relative = storage
            .store_pronunciation(card_id, b"fake-mp3", "mp3")
            .expect("store");
        assert!(relative.starts_with("media/audio/pronunciation/"));
        assert!(storage.pronunciation_exists(&relative));
        storage
            .remove_pronunciation(&relative)
            .expect("remove");
        assert!(!storage.pronunciation_exists(&relative));
    }
}
