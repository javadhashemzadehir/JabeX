use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Deck-level study preferences (extensible JSON-friendly struct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeckSettings {
    #[serde(default, alias = "new_cards_per_day")]
    pub new_cards_per_day: Option<u32>,
    #[serde(default, alias = "reviews_per_day")]
    pub reviews_per_day: Option<u32>,
}

/// Pure domain model for a deck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub settings: DeckSettings,
    /// Monotonic write counter for future document sync.
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Deck {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            name,
            description,
            settings: DeckSettings::default(),
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
}
