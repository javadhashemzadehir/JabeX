use chrono::{DateTime, Duration, Utc};

use super::card::{Card, CardState};
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

/// Classic Leitner box scheduler.
#[derive(Debug, Default, Clone, Copy)]
pub struct LeitnerScheduler;

impl LeitnerScheduler {
    pub const BOX_INTERVALS_DAYS: [i64; 5] = [1, 3, 7, 14, 30];

    pub fn interval_for_box(box_position: u32) -> Duration {
        let idx = (Card::clamp_box(box_position) - 1) as usize;
        Duration::days(Self::BOX_INTERVALS_DAYS[idx])
    }
}

impl Scheduler for LeitnerScheduler {
    fn schedule(&self, card: &Card, grade: Grade, now: DateTime<Utc>) -> ScheduleResult {
        let review_count = card.review_count.saturating_add(1);

        if grade.is_correct() {
            let box_position = Card::clamp_box(card.box_position.saturating_add(1));
            ScheduleResult {
                box_position,
                due_at: now + Self::interval_for_box(box_position),
                state: CardState::Review,
                review_count,
                lapse_count: card.lapse_count,
            }
        } else {
            let box_position = Card::MIN_BOX;
            ScheduleResult {
                box_position,
                due_at: now + Self::interval_for_box(box_position),
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
        LeitnerScheduler.schedule(card, grade, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::Card;
    use uuid::Uuid;

    fn sample_card(box_position: u32) -> Card {
        let mut card = Card::new(Uuid::nil(), "front".into(), "back".into());
        card.box_position = box_position;
        card.state = CardState::Review;
        card
    }

    #[test]
    fn correct_answer_promotes_box() {
        let scheduler = LeitnerScheduler;
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
        let scheduler = LeitnerScheduler;
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
        let scheduler = LeitnerScheduler;
        let card = sample_card(5);
        let now = Utc::now();
        let result = scheduler.schedule(&card, Grade::Easy, now);
        assert_eq!(result.box_position, 5);
        assert_eq!(result.due_at, now + Duration::days(30));
    }

    #[test]
    fn intervals_match_defaults() {
        assert_eq!(LeitnerScheduler::interval_for_box(1), Duration::days(1));
        assert_eq!(LeitnerScheduler::interval_for_box(2), Duration::days(3));
        assert_eq!(LeitnerScheduler::interval_for_box(3), Duration::days(7));
        assert_eq!(LeitnerScheduler::interval_for_box(4), Duration::days(14));
        assert_eq!(LeitnerScheduler::interval_for_box(5), Duration::days(30));
    }

    #[test]
    fn new_card_starts_ready_for_box_one_path() {
        let card = Card::new(Uuid::nil(), "q".into(), "a".into());
        assert_eq!(card.box_position, 1);
        let result = LeitnerScheduler.schedule(&card, Grade::Good, Utc::now());
        assert_eq!(result.box_position, 2);
    }
}
