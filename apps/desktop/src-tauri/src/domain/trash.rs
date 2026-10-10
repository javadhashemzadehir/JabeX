use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::JabeXResult;

/// Entity kinds that appear in the profile Trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrashEntityKind {
    Deck,
    Card,
    Attachment,
    InkDocument,
}

impl TrashEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deck => "deck",
            Self::Card => "card",
            Self::Attachment => "attachment",
            Self::InkDocument => "ink_document",
        }
    }
}

/// Unified trash row for UI listing (normalized across entity tables).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub entity_kind: TrashEntityKind,
    pub id: Uuid,
    /// Human-readable label (deck name, card preview, file name, …).
    pub title: String,
    pub deleted_at: DateTime<Utc>,
    /// Local profile that performed the soft-delete (not an online account).
    pub deleted_by: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub parent_kind: Option<TrashEntityKind>,
}

/// Port for Trash persistence and cascading restore / permanent delete.
///
/// Soft-deleted rows remain in SQLite until an explicit permanent delete.
/// Normal queries on entity repositories exclude `deleted_at IS NOT NULL`.
pub trait TrashRepository: Send + Sync {
    fn list_deleted_items(&self) -> JabeXResult<Vec<TrashItem>>;

    /// Restore the entity and cascade-restore soft-deleted children.
    fn restore_item(&self, kind: TrashEntityKind, id: Uuid, at: DateTime<Utc>)
        -> JabeXResult<()>;

    /// Irreversibly remove the entity (and children) from SQLite.
    /// Callers must remove media files after this returns.
    fn permanently_delete_item(&self, kind: TrashEntityKind, id: Uuid) -> JabeXResult<()>;
}
