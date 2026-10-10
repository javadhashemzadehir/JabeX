use std::fs;
use std::io::Write;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tracing::info;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::infrastructure::database::DatabasePaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Audio,
    Drawing,
}

impl MediaKind {
    fn dir<'a>(&self, paths: &'a DatabasePaths) -> &'a PathBuf {
        match self {
            MediaKind::Image => &paths.images_dir,
            MediaKind::Audio => &paths.audio_dir,
            MediaKind::Drawing => &paths.drawings_dir,
        }
    }

    fn extension_fallback(self) -> &'static str {
        match self {
            MediaKind::Image => "bin",
            MediaKind::Audio => "bin",
            MediaKind::Drawing => "bin",
        }
    }
}

/// Stores media outside SQLite using SHA-256 content-addressed filenames.
#[derive(Clone)]
pub struct MediaManager {
    paths: DatabasePaths,
}

impl MediaManager {
    pub fn new(paths: DatabasePaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &DatabasePaths {
        &self.paths
    }

    pub fn store_bytes(
        &self,
        kind: MediaKind,
        bytes: &[u8],
        extension: Option<&str>,
    ) -> JabeXResult<StoredMedia> {
        let sha256 = hex::encode(Sha256::digest(bytes));
        let ext = extension
            .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| kind.extension_fallback().to_string());

        let file_name = format!("{sha256}.{ext}");
        let absolute = kind.dir(&self.paths).join(&file_name);

        if absolute.exists() {
            info!(%sha256, kind = ?kind, "media deduplicated");
            return Ok(StoredMedia {
                sha256,
                relative_path: relative_media_path(kind, &file_name),
                absolute_path: absolute,
                size_bytes: bytes.len() as u64,
                duplicated: true,
            });
        }

        let mut file = fs::File::create(&absolute).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create media file: {e}"))
        })?;
        file.write_all(bytes)
            .map_err(|e| JabeXError::Filesystem(format!("failed to write media file: {e}")))?;

        info!(%sha256, kind = ?kind, "media stored");
        Ok(StoredMedia {
            sha256,
            relative_path: relative_media_path(kind, &file_name),
            absolute_path: absolute,
            size_bytes: bytes.len() as u64,
            duplicated: false,
        })
    }

    pub fn absolute_path(&self, relative: &str) -> PathBuf {
        self.paths.root.join(relative)
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.absolute_path(relative).exists()
    }
}

fn relative_media_path(kind: MediaKind, file_name: &str) -> String {
    let folder = match kind {
        MediaKind::Image => "media/images",
        MediaKind::Audio => "media/audio",
        MediaKind::Drawing => "media/drawings",
    };
    format!("{folder}/{file_name}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMedia {
    pub sha256: String,
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub size_bytes: u64,
    pub duplicated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::DatabasePaths;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn stores_and_deduplicates_by_hash() {
        let dir = tempdir().expect("tempdir");
        let paths = DatabasePaths::from_root(dir.path().to_path_buf()).expect("paths");
        let media = MediaManager::new(paths);
        let first = media
            .store_bytes(MediaKind::Image, b"hello", Some("png"))
            .expect("store");
        let second = media
            .store_bytes(MediaKind::Image, b"hello", Some("png"))
            .expect("store again");
        assert!(!first.duplicated);
        assert!(second.duplicated);
        assert_eq!(first.sha256, second.sha256);
        assert!(Path::new(&first.absolute_path).exists());
    }
}
