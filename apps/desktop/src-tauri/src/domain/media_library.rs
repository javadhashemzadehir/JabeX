use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::attachment::{Attachment, AttachmentType};

/// Filter for browsing the profile media library.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryFilter {
    /// When true, include soft-deleted rows (trash).
    #[serde(default)]
    pub include_deleted: bool,
    /// Optional MIME-derived type filter.
    pub attachment_type: Option<AttachmentType>,
    /// Case-insensitive substring match on `file_name`.
    pub query: Option<String>,
}

/// Attachment row enriched for library browsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryItem {
    #[serde(flatten)]
    pub attachment: Attachment,
    pub attachment_type: Option<AttachmentType>,
    /// True when the binary is missing under the profile media root.
    pub file_missing: bool,
}

/// Aggregate media usage for the active local profile (no telemetry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryStats {
    pub total_count: u32,
    pub deleted_count: u32,
    pub total_bytes: u64,
    pub deleted_bytes: u64,
    pub image_count: u32,
    pub audio_count: u32,
    pub video_count: u32,
    pub pdf_count: u32,
    pub missing_file_count: u32,
    /// Attachment UUID folders on disk with no active/deleted DB row.
    pub orphan_dir_count: u32,
}

impl MediaLibraryStats {
    pub fn empty() -> Self {
        Self {
            total_count: 0,
            deleted_count: 0,
            total_bytes: 0,
            deleted_bytes: 0,
            image_count: 0,
            audio_count: 0,
            video_count: 0,
            pdf_count: 0,
            missing_file_count: 0,
            orphan_dir_count: 0,
        }
    }
}

/// Result of permanently removing soft-deleted media.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPurgeResult {
    pub purged_count: u32,
    pub freed_bytes: u64,
}

/// Local integrity check over attachment metadata + files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaIntegrityReport {
    pub checked: u32,
    pub missing_files: Vec<Uuid>,
    pub checksum_mismatches: Vec<Uuid>,
    pub orphan_dirs: Vec<Uuid>,
}

impl MediaIntegrityReport {
    pub fn healthy(&self) -> bool {
        self.missing_files.is_empty()
            && self.checksum_mismatches.is_empty()
            && self.orphan_dirs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attachment::{AttachmentOwner, AttachmentType};
    use crate::shared::ids::create_id;
    use crate::shared::time::now_utc;

    #[test]
    fn empty_stats_and_healthy_report() {
        assert_eq!(MediaLibraryStats::empty().total_count, 0);
        let report = MediaIntegrityReport {
            checked: 0,
            missing_files: vec![],
            checksum_mismatches: vec![],
            orphan_dirs: vec![],
        };
        assert!(report.healthy());
    }

    #[test]
    fn library_item_flattens_for_ipc() {
        let now = now_utc();
        let item = MediaLibraryItem {
            attachment: Attachment {
                id: create_id(),
                owner_type: AttachmentOwner::Card,
                owner_id: create_id(),
                file_name: "a.png".into(),
                mime_type: "image/png".into(),
                size_bytes: 12,
                hash: "abcd".into(),
                relative_path: "media/attachments/x/original.png".into(),
                revision: 1,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                deleted_by: None,
            },
            attachment_type: Some(AttachmentType::Image),
            file_missing: false,
        };
        let json = serde_json::to_value(&item).expect("json");
        assert!(json.get("fileName").is_some());
        assert!(json.get("attachment").is_none());
        assert_eq!(json["attachmentType"], "image");
        assert_eq!(json["fileMissing"], false);
    }
}
