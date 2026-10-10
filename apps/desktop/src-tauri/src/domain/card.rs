use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::language_assist::{
    LanguageAssistOverrides, LanguageAssistStatus, TransliterationMetadata,
};
use crate::domain::rich_text::RichTextDocument;
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
    /// Versioned TipTap JSON document (front face).
    pub front_content: RichTextDocument,
    /// Versioned TipTap JSON document (back face).
    pub back_content: RichTextDocument,
    pub box_position: u32,
    pub due_at: DateTime<Utc>,
    pub state: CardState,
    pub review_count: u32,
    pub lapse_count: u32,
    pub difficulty: Option<f64>,
    /// BCP-47-ish code from local detection (e.g. `en`, `fa`, `und`).
    pub detected_language: Option<String>,
    /// Cached translation text (optional; never required for study).
    pub translation: Option<String>,
    /// Profile-relative path under `media/audio/` when pronunciation was generated.
    pub pronunciation_audio_path: Option<String>,
    /// Present when the written form is a Persian (etc.) transliteration of a foreign lemma.
    pub transliteration: Option<TransliterationMetadata>,
    /// Async language-assist job status; failures never block card CRUD.
    pub language_assist_status: LanguageAssistStatus,
    /// Per-card overrides for text translation / spoken pronunciation (default inherit).
    #[serde(default)]
    pub language_assist_overrides: LanguageAssistOverrides,
    /// Monotonic write counter for future document sync.
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Local profile UUID that soft-deleted this card (sync-ready; not an account).
    #[serde(default)]
    pub deleted_by: Option<Uuid>,
}

impl Card {
    pub const MIN_BOX: u32 = 1;
    pub const MAX_BOX: u32 = 5;

    pub fn new(
        deck_id: Uuid,
        front_content: impl Into<RichTextDocument>,
        back_content: impl Into<RichTextDocument>,
    ) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            deck_id,
            card_type: CardType::Basic,
            front_content: front_content.into(),
            back_content: back_content.into(),
            box_position: Self::MIN_BOX,
            due_at: now,
            state: CardState::New,
            review_count: 0,
            lapse_count: 0,
            difficulty: None,
            detected_language: None,
            translation: None,
            pronunciation_audio_path: None,
            transliteration: None,
            language_assist_status: LanguageAssistStatus::Idle,
            language_assist_overrides: LanguageAssistOverrides::default(),
            revision: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            deleted_by: None,
        }
    }

    /// Clear generated language metadata (e.g. after content edit before re-queue).
    pub fn clear_language_assist(&mut self) {
        self.detected_language = None;
        self.translation = None;
        self.pronunciation_audio_path = None;
        self.transliteration = None;
        self.language_assist_status = LanguageAssistStatus::Idle;
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

    pub fn clamp_box(position: u32) -> u32 {
        position.clamp(Self::MIN_BOX, Self::MAX_BOX)
    }

    /// Lightweight list row — omits full rich-text documents.
    pub fn to_summary(&self) -> CardSummary {
        CardSummary {
            id: self.id,
            deck_id: self.deck_id,
            front_preview: truncate_preview(&self.front_content.plain_text()),
            back_preview: truncate_preview(&self.back_content.plain_text()),
            box_position: self.box_position,
            due_at: self.due_at,
            state: self.state,
            detected_language: self.detected_language.clone(),
            language_assist_status: self.language_assist_status,
            revision: self.revision,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

const PREVIEW_MAX_CHARS: usize = 160;

fn truncate_preview(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= PREVIEW_MAX_CHARS {
        return trimmed.to_string();
    }
    let truncated: String = trimmed.chars().take(PREVIEW_MAX_CHARS).collect();
    format!("{truncated}…")
}

/// Card list projection for UI lists (no full front/back documents).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub front_preview: String,
    pub back_preview: String,
    pub box_position: u32,
    pub due_at: DateTime<Utc>,
    pub state: CardState,
    pub detected_language: Option<String>,
    pub language_assist_status: LanguageAssistStatus,
    pub revision: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Paginated card list response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardListPage {
    pub items: Vec<CardSummary>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub has_more: bool,
}
