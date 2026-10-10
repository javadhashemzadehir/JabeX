use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Review grade. Leitner currently treats `Again` as fail and others as pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Grade {
    pub fn is_correct(self) -> bool {
        !matches!(self, Grade::Again)
    }
}

/// Append-only review log entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: Uuid,
    pub card_id: Uuid,
    pub grade: Grade,
    pub reviewed_at: DateTime<Utc>,
    pub duration_ms: Option<u32>,
    pub box_before: u32,
    pub box_after: u32,
    /// Always `1` for new rows; retained for document-export symmetry.
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Review {
    pub fn new(
        card_id: Uuid,
        grade: Grade,
        reviewed_at: DateTime<Utc>,
        duration_ms: Option<u32>,
        box_before: u32,
        box_after: u32,
    ) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            card_id,
            grade,
            reviewed_at,
            duration_ms,
            box_before,
            box_after,
            revision: 1,
            created_at: now,
            updated_at: now,
        }
    }
}
