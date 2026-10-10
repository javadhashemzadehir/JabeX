use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Polymorphic owner of an attachment (card today; documents later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentOwner {
    Card,
    Document,
}

impl AttachmentOwner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Document => "document",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "card" => Ok(Self::Card),
            "document" => Ok(Self::Document),
            other => Err(format!("unknown attachment owner type: {other}")),
        }
    }
}

/// High-level media category derived from MIME type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    Image,
    Audio,
    Video,
    Pdf,
}

impl AttachmentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Pdf => "pdf",
        }
    }

    pub fn from_mime(mime: &str) -> Option<Self> {
        let mime = mime.to_ascii_lowercase();
        if mime == "application/pdf" {
            return Some(Self::Pdf);
        }
        if mime.starts_with("image/") {
            return Some(Self::Image);
        }
        if mime.starts_with("audio/") {
            return Some(Self::Audio);
        }
        if mime.starts_with("video/") {
            return Some(Self::Video);
        }
        None
    }

    pub fn max_bytes(self) -> u64 {
        match self {
            Self::Image => 20 * 1024 * 1024,
            Self::Audio => 50 * 1024 * 1024,
            Self::Video => 200 * 1024 * 1024,
            Self::Pdf => 50 * 1024 * 1024,
        }
    }
}

/// Sidecar metadata written next to the binary on disk (not a DB concern).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetadata {
    pub id: Uuid,
    pub owner_type: AttachmentOwner,
    pub owner_id: Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub attachment_type: AttachmentType,
    pub size_bytes: u64,
    pub hash: String,
    pub relative_path: String,
    pub thumbnails: Vec<ThumbnailRef>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRef {
    pub relative_path: String,
    pub width: u32,
    pub height: u32,
}

/// Attachment metadata row — binary bytes live on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Uuid,
    pub owner_type: AttachmentOwner,
    pub owner_id: Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub hash: String,
    pub relative_path: String,
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Local profile UUID that soft-deleted this attachment.
    #[serde(default)]
    pub deleted_by: Option<Uuid>,
}

impl Attachment {
    pub fn new(
        owner_type: AttachmentOwner,
        owner_id: Uuid,
        file_name: String,
        mime_type: String,
        size_bytes: u64,
        hash: String,
        relative_path: String,
    ) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            owner_type,
            owner_id,
            file_name,
            mime_type,
            size_bytes,
            hash,
            relative_path,
            revision: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            deleted_by: None,
        }
    }

    pub fn attachment_type(&self) -> Option<AttachmentType> {
        AttachmentType::from_mime(&self.mime_type)
    }

    pub fn to_metadata(&self, thumbnails: Vec<ThumbnailRef>) -> Option<AttachmentMetadata> {
        let attachment_type = self.attachment_type()?;
        Some(AttachmentMetadata {
            id: self.id,
            owner_type: self.owner_type,
            owner_id: self.owner_id,
            file_name: self.file_name.clone(),
            mime_type: self.mime_type.clone(),
            attachment_type,
            size_bytes: self.size_bytes,
            hash: self.hash.clone(),
            relative_path: self.relative_path.clone(),
            thumbnails,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn soft_delete(&mut self, at: DateTime<Utc>, deleted_by: Option<Uuid>) {
        self.deleted_at = Some(at);
        self.deleted_by = deleted_by;
        self.updated_at = at;
        self.bump_revision();
    }

    pub fn restore(&mut self, at: DateTime<Utc>) {
        self.deleted_at = None;
        self.deleted_by = None;
        self.updated_at = at;
        self.bump_revision();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Allowed MIME types for local attachments (no cloud upload).
pub fn is_allowed_mime(mime: &str) -> bool {
    AttachmentType::from_mime(mime).is_some_and(|kind| match kind {
        AttachmentType::Image => matches!(
            mime.to_ascii_lowercase().as_str(),
            "image/png" | "image/jpeg" | "image/jpg" | "image/webp" | "image/gif" | "image/svg+xml"
        ),
        AttachmentType::Audio => matches!(
            mime.to_ascii_lowercase().as_str(),
            "audio/mpeg" | "audio/mp3" | "audio/wav" | "audio/ogg" | "audio/webm" | "audio/mp4"
        ),
        AttachmentType::Video => matches!(
            mime.to_ascii_lowercase().as_str(),
            "video/mp4" | "video/webm" | "video/ogg" | "video/quicktime"
        ),
        AttachmentType::Pdf => mime.eq_ignore_ascii_case("application/pdf"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mime_and_limits() {
        assert_eq!(
            AttachmentType::from_mime("image/png"),
            Some(AttachmentType::Image)
        );
        assert_eq!(
            AttachmentType::from_mime("application/pdf"),
            Some(AttachmentType::Pdf)
        );
        assert!(AttachmentType::Image.max_bytes() < AttachmentType::Video.max_bytes());
        assert!(is_allowed_mime("image/png"));
        assert!(!is_allowed_mime("application/zip"));
    }
}
