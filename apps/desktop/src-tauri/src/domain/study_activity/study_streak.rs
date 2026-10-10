use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use super::study_activity::StudyActivity;

/// Streak summary calculated from daily study activity (never persisted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StudyStreak {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_study_days: u32,
}

/// Calculate streaks from unique activity days that have at least one card reviewed.
///
/// `today` is the caller's local calendar date. Current streak counts consecutive
/// days ending on `today` or yesterday (so a streak is not broken until a full
/// local day is missed).
pub fn calculate_streak(activities: &[StudyActivity], today: NaiveDate) -> StudyStreak {
    let mut days: Vec<NaiveDate> = activities
        .iter()
        .filter(|a| a.cards_reviewed > 0)
        .map(|a| a.activity_date)
        .collect();
    days.sort_unstable();
    days.dedup();

    let total_study_days = days.len() as u32;
    if days.is_empty() {
        return StudyStreak::default();
    }

    let longest_streak = longest_run(&days);
    let current_streak = current_run(&days, today);

    StudyStreak {
        current_streak,
        longest_streak,
        total_study_days,
    }
}

fn longest_run(sorted_unique_days: &[NaiveDate]) -> u32 {
    let mut best = 1u32;
    let mut run = 1u32;
    for window in sorted_unique_days.windows(2) {
        if window[1] == window[0] + Duration::days(1) {
            run = run.saturating_add(1);
            best = best.max(run);
        } else {
            run = 1;
        }
    }
    best
}

fn current_run(sorted_unique_days: &[NaiveDate], today: NaiveDate) -> u32 {
    let last = *sorted_unique_days.last().expect("non-empty");
    // Streak is alive if the latest study day is today or yesterday.
    if last != today && last != today - Duration::days(1) {
        return 0;
    }

    let mut streak = 1u32;
    let mut expected = last - Duration::days(1);
    for day in sorted_unique_days.iter().rev().skip(1) {
        if *day == expected {
            streak = streak.saturating_add(1);
            expected = expected - Duration::days(1);
        } else if *day < expected {
            break;
        }
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("date")
    }

    fn activity(date: NaiveDate, cards: u32) -> StudyActivity {
        StudyActivity::new(Uuid::nil(), date, cards, 0)
    }

    #[test]
    fn no_activity() {
        let streak = calculate_streak(&[], day(2026, 10, 10));
        assert_eq!(streak, StudyStreak::default());
    }

    #[test]
    fn one_day_activity() {
        let today = day(2026, 10, 10);
        let streak = calculate_streak(&[activity(today, 5)], today);
        assert_eq!(
            streak,
            StudyStreak {
                current_streak: 1,
                longest_streak: 1,
                total_study_days: 1,
            }
        );
    }

    #[test]
    fn consecutive_days() {
        let today = day(2026, 10, 10);
        let activities = vec![
            activity(day(2026, 10, 8), 10),
            activity(day(2026, 10, 9), 20),
            activity(day(2026, 10, 10), 87),
        ];
        let streak = calculate_streak(&activities, today);
        assert_eq!(streak.current_streak, 3);
        assert_eq!(streak.longest_streak, 3);
        assert_eq!(streak.total_study_days, 3);
    }

    #[test]
    fn missing_day_breaks_current_streak() {
        let today = day(2026, 10, 10);
        let activities = vec![
            activity(day(2026, 10, 7), 5),
            activity(day(2026, 10, 8), 5),
            // gap on 9th
            activity(day(2026, 10, 10), 5),
        ];
        let streak = calculate_streak(&activities, today);
        assert_eq!(streak.current_streak, 1);
        assert_eq!(streak.longest_streak, 2);
        assert_eq!(streak.total_study_days, 3);
    }

    #[test]
    fn longest_streak_across_history() {
        let today = day(2026, 10, 10);
        let activities = vec![
            activity(day(2026, 9, 1), 1),
            activity(day(2026, 9, 2), 1),
            activity(day(2026, 9, 3), 1),
            activity(day(2026, 9, 4), 1),
            activity(day(2026, 10, 9), 1),
            activity(day(2026, 10, 10), 1),
        ];
        let streak = calculate_streak(&activities, today);
        assert_eq!(streak.current_streak, 2);
        assert_eq!(streak.longest_streak, 4);
        assert_eq!(streak.total_study_days, 6);
    }

    #[test]
    fn yesterday_keeps_current_streak_alive() {
        let today = day(2026, 10, 10);
        let activities = vec![
            activity(day(2026, 10, 8), 1),
            activity(day(2026, 10, 9), 1),
        ];
        let streak = calculate_streak(&activities, today);
        assert_eq!(streak.current_streak, 2);
    }

    #[test]
    fn zero_card_days_ignored() {
        let today = day(2026, 10, 10);
        let activities = vec![
            activity(day(2026, 10, 9), 0),
            activity(day(2026, 10, 10), 3),
        ];
        let streak = calculate_streak(&activities, today);
        assert_eq!(streak.current_streak, 1);
        assert_eq!(streak.total_study_days, 1);
    }
}
