use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Basic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardState {
    New,
    Learning,
    Review,
    Suspended,
}

/// Pure domain model for a flashcard. No persistence concerns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub card_type: CardType,
    pub front_content: String,
    pub back_content: String,
    pub box_position: u32,
    pub due_at: DateTime<Utc>,
    pub state: CardState,
    pub review_count: u32,
    pub lapse_count: u32,
    pub difficulty: Option<f64>,
    /// Monotonic write counter for future document sync.
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Card {
    pub const MIN_BOX: u32 = 1;
    pub const MAX_BOX: u32 = 5;

    pub fn new(deck_id: Uuid, front_content: String, back_content: String) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            deck_id,
            card_type: CardType::Basic,
            front_content,
            back_content,
            box_position: Self::MIN_BOX,
            due_at: now,
            state: CardState::New,
            review_count: 0,
            lapse_count: 0,
            difficulty: None,
            revision: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn soft_delete(&mut self, at: DateTime<Utc>) {
        self.deleted_at = Some(at);
        self.updated_at = at;
        self.bump_revision();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn clamp_box(position: u32) -> u32 {
        position.clamp(Self::MIN_BOX, Self::MAX_BOX)
    }
}
