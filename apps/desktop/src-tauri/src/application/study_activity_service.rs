use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::errors::JabeXResult;
use crate::domain::repositories::{ReviewRepository, StudyActivityRepository};
use crate::domain::study_activity::{
    calculate_statistics, format_activity_date, StudyActivity, StudyConsistencyStats,
};
use crate::shared::time::now_utc;

/// Internal application event emitted when a Leitner review is recorded.
///
/// Each graded card is one atomic study unit. Daily `study_activity` rows are
/// upserted from these events (no manual “complete habit” action).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCompletedEvent {
    pub profile_id: Uuid,
    pub deck_id: Uuid,
    pub cards_reviewed: u32,
    pub duration_seconds: u32,
}

impl ReviewCompletedEvent {
    pub fn from_review(
        profile_id: Uuid,
        deck_id: Uuid,
        duration_ms: Option<u32>,
    ) -> Self {
        Self {
            profile_id,
            deck_id,
            cards_reviewed: 1,
            duration_seconds: duration_ms.unwrap_or(0) / 1000,
        }
    }
}

/// Records study activity and serves consistency statistics.
pub struct StudyActivityService<A: StudyActivityRepository, R: ReviewRepository> {
    activities: A,
    reviews: R,
    profile_id: Uuid,
}

impl<A: StudyActivityRepository, R: ReviewRepository> StudyActivityService<A, R> {
    pub fn new(activities: A, reviews: R, profile_id: Uuid) -> Self {
        Self {
            activities,
            reviews,
            profile_id,
        }
    }

    pub fn record_study_activity(&self, event: &ReviewCompletedEvent) -> JabeXResult<StudyActivity> {
        let today = local_activity_date(now_utc());
        tracing::debug!(
            profile_id = %event.profile_id,
            deck_id = %event.deck_id,
            cards = event.cards_reviewed,
            duration_seconds = event.duration_seconds,
            activity_date = %format_activity_date(today),
            "recording study activity"
        );
        self.activities.upsert_add(
            event.profile_id,
            today,
            event.cards_reviewed,
            event.duration_seconds,
        )
    }

    pub fn handle_review_completed(
        &self,
        event: &ReviewCompletedEvent,
    ) -> JabeXResult<StudyActivity> {
        // If this is the first activity row, project the full review log (including
        // the review just written) and skip a second upsert to avoid double-counting.
        if self.activities.count(self.profile_id)? == 0 {
            self.backfill_from_reviews()?;
            let today = local_activity_date(now_utc());
            if let Some(row) = self.activities.find_by_date(self.profile_id, today)? {
                return Ok(row);
            }
        }
        self.record_study_activity(event)
    }

    pub fn get_consistency_stats(&self) -> JabeXResult<StudyConsistencyStats> {
        self.ensure_backfilled()?;
        let today = local_activity_date(now_utc());
        // Heatmap is 12 months — avoid loading the entire activity history.
        let from = today - Duration::days(366);
        let activities = self
            .activities
            .list_range(self.profile_id, from, today)?;
        Ok(calculate_statistics(&activities, today))
    }

    pub fn list_activity(&self) -> JabeXResult<Vec<StudyActivity>> {
        self.ensure_backfilled()?;
        let today = local_activity_date(now_utc());
        let from = today - Duration::days(366);
        self.activities.list_range(self.profile_id, from, today)
    }

    /// One-time projection of historical reviews into daily activity rows.
    fn ensure_backfilled(&self) -> JabeXResult<()> {
        if self.activities.count(self.profile_id)? > 0 {
            return Ok(());
        }
        self.backfill_from_reviews()
    }

    fn backfill_from_reviews(&self) -> JabeXResult<()> {
        let reviews = self.reviews.list_all()?;
        if reviews.is_empty() {
            return Ok(());
        }

        let mut by_day: std::collections::BTreeMap<NaiveDate, (u32, u32)> =
            std::collections::BTreeMap::new();
        for review in reviews {
            let day = local_activity_date(review.reviewed_at);
            let entry = by_day.entry(day).or_insert((0, 0));
            entry.0 = entry.0.saturating_add(1);
            entry.1 = entry.1.saturating_add(review.duration_ms.unwrap_or(0) / 1000);
        }

        for (date, (cards, seconds)) in by_day {
            self.activities
                .upsert_add(self.profile_id, date, cards, seconds)?;
        }
        Ok(())
    }
}

/// Local calendar date for streak boundaries (matches stats “reviews today”).
pub fn local_activity_date(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&Local).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::Card;
    use crate::domain::deck::Deck;
    use crate::domain::repositories::{CardRepository, DeckRepository, ReviewRepository};
    use crate::domain::review::{Grade, Review};
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::repositories::{
        SqliteCardRepository, SqliteDeckRepository, SqliteReviewRepository,
        SqliteStudyActivityRepository,
    };
    use tempfile::tempdir;

    #[test]
    fn records_activity_from_review_event() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let profile = db.paths().profile_id;
        let service = StudyActivityService::new(
            SqliteStudyActivityRepository::new(db.clone()),
            SqliteReviewRepository::new(db),
            profile,
        );

        let event = ReviewCompletedEvent {
            profile_id: profile,
            deck_id: Uuid::nil(),
            cards_reviewed: 3,
            duration_seconds: 120,
        };
        let row = service.handle_review_completed(&event).expect("record");
        assert_eq!(row.cards_reviewed, 3);
        assert_eq!(row.minutes_spent, 2);

        let stats = service.get_consistency_stats().expect("stats");
        assert_eq!(stats.cards_reviewed_today, 3);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.total_study_days, 1);
    }

    #[test]
    fn backfills_from_existing_reviews() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let profile = db.paths().profile_id;
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let reviews = SqliteReviewRepository::new(db.clone());

        let deck = Deck::new("D".into(), None);
        decks.create(&deck).expect("deck");
        let card = Card::new(deck.id, "f", "b");
        cards.create(&card).expect("card");
        let review = Review::new(card.id, Grade::Good, now_utc(), Some(5000), 1, 2);
        reviews.create(&review).expect("review");

        let service = StudyActivityService::new(
            SqliteStudyActivityRepository::new(db.clone()),
            reviews,
            profile,
        );
        let stats = service.get_consistency_stats().expect("stats");
        assert_eq!(stats.cards_reviewed_today, 1);
        assert_eq!(stats.total_study_days, 1);
    }
}
