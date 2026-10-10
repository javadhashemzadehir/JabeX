use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::leitner::{
    DeckAlgorithmSettings, LeitnerConfig, ProfileLeitnerSettings, PROFILE_ALGORITHM_SETTING_KEY,
};
use crate::domain::repositories::{CardRepository, DeckRepository, ReviewRepository};
use crate::domain::review::{Grade, Review};
use crate::domain::scheduler::{LeitnerScheduler, Scheduler};
use crate::shared::time::now_utc;

use super::settings_service::SettingsService;

pub struct ReviewService<C: CardRepository, R: ReviewRepository, D: DeckRepository> {
    cards: C,
    reviews: R,
    decks: D,
    settings: SettingsService,
}

impl<C: CardRepository, R: ReviewRepository, D: DeckRepository> ReviewService<C, R, D> {
    pub fn new(cards: C, reviews: R, decks: D, settings: SettingsService) -> Self {
        Self {
            cards,
            reviews,
            decks,
            settings,
        }
    }

    /// Default review window size — keeps IPC/memory bounded for large due queues.
    pub const DEFAULT_DUE_WINDOW: u32 = 40;
    const MAX_DUE_WINDOW: u32 = 100;

    pub fn list_due(
        &self,
        deck_id: Option<Uuid>,
        now: Option<DateTime<Utc>>,
    ) -> JabeXResult<Vec<Card>> {
        self.list_due_window(deck_id, now, Some(Self::DEFAULT_DUE_WINDOW))
    }

    pub fn list_due_window(
        &self,
        deck_id: Option<Uuid>,
        now: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> JabeXResult<Vec<Card>> {
        let now = now.unwrap_or_else(now_utc);
        let limit = limit
            .unwrap_or(Self::DEFAULT_DUE_WINDOW)
            .clamp(1, Self::MAX_DUE_WINDOW);
        self.cards.list_due_limited(deck_id, now, limit)
    }

    pub fn load_profile_algorithm(&self) -> JabeXResult<ProfileLeitnerSettings> {
        match self.settings.get(PROFILE_ALGORITHM_SETTING_KEY)? {
            Some(entry) => Ok(serde_json::from_value(entry.value).unwrap_or_default()),
            None => Ok(ProfileLeitnerSettings::default()),
        }
    }

    pub fn save_profile_algorithm(
        &self,
        settings: ProfileLeitnerSettings,
    ) -> JabeXResult<ProfileLeitnerSettings> {
        // Validate by resolving before persist.
        let _ = settings.resolve()?;
        let value = serde_json::to_value(&settings)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.settings
            .set(PROFILE_ALGORITHM_SETTING_KEY.to_string(), value)?;
        Ok(settings)
    }

    pub fn effective_config_for_deck(&self, deck_id: Uuid) -> JabeXResult<LeitnerConfig> {
        let deck = self.decks.find(deck_id)?.ok_or(JabeXError::NotFound {
            entity: "deck",
            id: deck_id,
        })?;
        let profile = self.load_profile_algorithm()?;
        deck.settings.algorithm.resolve(&profile)
    }

    pub fn submit_review(
        &self,
        card_id: Uuid,
        grade: Grade,
        duration_ms: Option<u32>,
        now: Option<DateTime<Utc>>,
    ) -> JabeXResult<ReviewOutcome> {
        let now = now.unwrap_or_else(now_utc);
        let mut card = self.cards.find(card_id)?.ok_or(JabeXError::NotFound {
            entity: "card",
            id: card_id,
        })?;

        let config = self.effective_config_for_deck(card.deck_id)?;
        let scheduler = LeitnerScheduler::new(config);
        let box_before = card.box_position;
        let schedule = scheduler.schedule(&card, grade, now);

        card.box_position = schedule.box_position;
        card.due_at = schedule.due_at;
        card.state = schedule.state;
        card.review_count = schedule.review_count;
        card.lapse_count = schedule.lapse_count;
        card.updated_at = now;
        card.bump_revision();
        self.cards.update(&card)?;

        let review = Review::new(
            card_id,
            grade,
            now,
            duration_ms,
            box_before,
            schedule.box_position,
        );
        self.reviews.create(&review)?;

        tracing::info!(
            card_id = %card_id,
            box_before,
            box_after = schedule.box_position,
            "review recorded"
        );

        Ok(ReviewOutcome { card, review })
    }

    /// Copy the profile algorithm onto every deck as an explicit override, or reset decks to inherit.
    pub fn apply_profile_algorithm_to_all_decks(&self, inherit: bool) -> JabeXResult<u32> {
        let profile = self.load_profile_algorithm()?;
        let mut updated = 0u32;
        for mut deck in self.decks.list()? {
            deck.settings.algorithm = if inherit {
                DeckAlgorithmSettings::inherit_profile()
            } else {
                DeckAlgorithmSettings::from_profile(&profile)
            };
            deck.updated_at = now_utc();
            deck.bump_revision();
            self.decks.update(&deck)?;
            updated = updated.saturating_add(1);
        }
        Ok(updated)
    }
}

#[derive(Debug, Clone)]
pub struct ReviewOutcome {
    pub card: Card,
    pub review: Review,
}
