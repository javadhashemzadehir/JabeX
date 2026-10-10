use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Local device profile — not an online account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub display_name: String,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Local profile that initiated deletion (usually self); sync-ready metadata.
    #[serde(default)]
    pub deleted_by: Option<Uuid>,
}

impl Profile {
    pub fn new(display_name: String, avatar: Option<String>) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            display_name,
            avatar,
            created_at: now,
            updated_at: now,
            last_active_at: now,
            deleted_at: None,
            deleted_by: None,
        }
    }

    pub fn soft_delete(&mut self, at: DateTime<Utc>, deleted_by: Option<Uuid>) {
        self.deleted_at = Some(at);
        self.deleted_by = deleted_by;
        self.updated_at = at;
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn touch_active(&mut self, at: DateTime<Utc>) {
        self.last_active_at = at;
        self.updated_at = at;
    }
}
