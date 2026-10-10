use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::language_assist::LanguageAssistOverrides;
use super::leitner::DeckAlgorithmSettings;
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
    /// When unset / inherit, the profile Leitner algorithm is used.
    #[serde(default)]
    pub algorithm: DeckAlgorithmSettings,
    /// Per-deck language assist overrides (profile settings remain the master gate).
    #[serde(default)]
    pub language_assist: LanguageAssistOverrides,
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
    /// Local profile UUID that soft-deleted this deck (sync-ready; not an account).
    #[serde(default)]
    pub deleted_by: Option<Uuid>,
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
            deleted_by: None,
        }
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
