use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::{CardRepository, ReviewRepository};
use crate::domain::review::{Grade, Review};
use crate::domain::scheduler::{LeitnerScheduler, Scheduler};
use crate::shared::time::now_utc;

pub struct ReviewService<C: CardRepository, R: ReviewRepository, S: Scheduler> {
    cards: C,
    reviews: R,
    scheduler: S,
}

impl<C: CardRepository, R: ReviewRepository, S: Scheduler> ReviewService<C, R, S> {
    pub fn new(cards: C, reviews: R, scheduler: S) -> Self {
        Self {
            cards,
            reviews,
            scheduler,
        }
    }

    pub fn list_due(
        &self,
        deck_id: Option<Uuid>,
        now: Option<DateTime<Utc>>,
    ) -> JabeXResult<Vec<Card>> {
        let now = now.unwrap_or_else(now_utc);
        self.cards.list_due(deck_id, now)
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

        let box_before = card.box_position;
        let schedule = self.scheduler.schedule(&card, grade, now);

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
}

pub type DefaultReviewService<C, R> = ReviewService<C, R, LeitnerScheduler>;

impl<C: CardRepository, R: ReviewRepository> ReviewService<C, R, LeitnerScheduler> {
    pub fn with_leitner(cards: C, reviews: R) -> Self {
        Self::new(cards, reviews, LeitnerScheduler)
    }
}

#[derive(Debug, Clone)]
pub struct ReviewOutcome {
    pub card: Card,
    pub review: Review,
}
