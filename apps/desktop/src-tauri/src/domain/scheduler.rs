use chrono::{DateTime, Duration, Utc};

use super::card::{Card, CardState};
use super::leitner::LeitnerConfig;
use super::review::Grade;

/// Result of scheduling a card after a review grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleResult {
    pub box_position: u32,
    pub due_at: DateTime<Utc>,
    pub state: CardState,
    pub review_count: u32,
    pub lapse_count: u32,
}

/// Pluggable spaced-repetition scheduler.
pub trait Scheduler: Send + Sync {
    fn schedule(&self, card: &Card, grade: Grade, now: DateTime<Utc>) -> ScheduleResult;
}

/// Classic Leitner box scheduler with configurable box intervals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeitnerScheduler {
    config: LeitnerConfig,
}

impl LeitnerScheduler {
    pub fn new(config: LeitnerConfig) -> Self {
        Self { config }
    }

    pub fn recommended() -> Self {
        Self::new(LeitnerConfig::recommended())
    }

    pub fn interval_for_box(&self, box_position: u32) -> Duration {
        Duration::days(self.config.interval_for_box(box_position))
    }
}

impl Default for LeitnerScheduler {
    fn default() -> Self {
        Self::recommended()
    }
}

impl Scheduler for LeitnerScheduler {
    fn schedule(&self, card: &Card, grade: Grade, now: DateTime<Utc>) -> ScheduleResult {
        let review_count = card.review_count.saturating_add(1);

        if grade.is_correct() {
            let box_position = Card::clamp_box(card.box_position.saturating_add(1));
            ScheduleResult {
                box_position,
                due_at: now + self.interval_for_box(box_position),
                state: CardState::Review,
                review_count,
                lapse_count: card.lapse_count,
            }
        } else {
            let box_position = Card::MIN_BOX;
            ScheduleResult {
                box_position,
                due_at: now + self.interval_for_box(box_position),
                state: CardState::Learning,
                review_count,
                lapse_count: card.lapse_count.saturating_add(1),
            }
        }
    }
}

/// Placeholder for a future FSRS implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct FsrsScheduler;

impl Scheduler for FsrsScheduler {
    fn schedule(&self, card: &Card, grade: Grade, now: DateTime<Utc>) -> ScheduleResult {
        // Temporary fallback until FSRS is implemented.
        LeitnerScheduler::recommended().schedule(card, grade, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::Card;
    use crate::domain::leitner::LeitnerPresetId;
    use uuid::Uuid;

    fn sample_card(box_position: u32) -> Card {
        let mut card = Card::new(Uuid::nil(), "front", "back");
        card.box_position = box_position;
        card.state = CardState::Review;
        card
    }

    #[test]
    fn correct_answer_promotes_box() {
        let scheduler = LeitnerScheduler::recommended();
        let card = sample_card(2);
        let now = Utc::now();
        let result = scheduler.schedule(&card, Grade::Good, now);
        assert_eq!(result.box_position, 3);
        assert_eq!(result.due_at, now + Duration::days(7));
        assert_eq!(result.state, CardState::Review);
        assert_eq!(result.review_count, 1);
        assert_eq!(result.lapse_count, 0);
    }

    #[test]
    fn wrong_answer_resets_to_box_one() {
        let scheduler = LeitnerScheduler::recommended();
        let mut card = sample_card(4);
        card.lapse_count = 2;
        let now = Utc::now();
        let result = scheduler.schedule(&card, Grade::Again, now);
        assert_eq!(result.box_position, 1);
        assert_eq!(result.due_at, now + Duration::days(1));
        assert_eq!(result.state, CardState::Learning);
        assert_eq!(result.lapse_count, 3);
    }

    #[test]
    fn correct_at_max_box_stays_at_five() {
        let scheduler = LeitnerScheduler::recommended();
        let card = sample_card(5);
        let now = Utc::now();
        let result = scheduler.schedule(&card, Grade::Easy, now);
        assert_eq!(result.box_position, 5);
        assert_eq!(result.due_at, now + Duration::days(30));
    }

    #[test]
    fn intensive_preset_uses_shorter_intervals() {
        let scheduler =
            LeitnerScheduler::new(LeitnerConfig::from_preset(LeitnerPresetId::Intensive));
        let card = sample_card(2);
        let now = Utc::now();
        let result = scheduler.schedule(&card, Grade::Good, now);
        assert_eq!(result.box_position, 3);
        assert_eq!(result.due_at, now + Duration::days(4));
    }

    #[test]
    fn new_card_starts_ready_for_box_one_path() {
        let card = Card::new(Uuid::nil(), "q", "a");
        assert_eq!(card.box_position, 1);
        let result = LeitnerScheduler::recommended().schedule(&card, Grade::Good, Utc::now());
        assert_eq!(result.box_position, 2);
    }
}
