use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use super::activity::{ActivityIntensity, StudyActivity};
use super::study_streak::{calculate_streak, StudyStreak};

/// Consistency statistics for the Learning Progress UI (local profile only).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyConsistencyStats {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_study_days: u32,
    pub cards_reviewed_today: u32,
    pub cards_reviewed_this_week: u32,
    pub cards_reviewed_this_month: u32,
    pub average_cards_per_study_day: f64,
    pub minutes_spent_today: u32,
    /// Last 12 months of daily cells for the heatmap (oldest → newest).
    pub heatmap: Vec<HeatmapDay>,
}

/// One day cell in the learning calendar heatmap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDay {
    pub date: NaiveDate,
    pub cards_reviewed: u32,
    pub minutes_spent: u32,
    pub intensity: ActivityIntensity,
}

/// Build consistency stats from daily activity rows.
pub fn calculate_statistics(
    activities: &[StudyActivity],
    today: NaiveDate,
) -> StudyConsistencyStats {
    let streak = calculate_streak(activities, today);
    let week_start = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

    let mut cards_today = 0u32;
    let mut minutes_today = 0u32;
    let mut cards_week = 0u32;
    let mut cards_month = 0u32;
    let mut cards_on_study_days = 0u32;
    let mut study_day_count = 0u32;

    for activity in activities {
        if activity.cards_reviewed == 0 {
            continue;
        }
        study_day_count = study_day_count.saturating_add(1);
        cards_on_study_days = cards_on_study_days.saturating_add(activity.cards_reviewed);

        if activity.activity_date == today {
            cards_today = cards_today.saturating_add(activity.cards_reviewed);
            minutes_today = minutes_today.saturating_add(activity.minutes_spent);
        }
        if activity.activity_date >= week_start && activity.activity_date <= today {
            cards_week = cards_week.saturating_add(activity.cards_reviewed);
        }
        if activity.activity_date >= month_start && activity.activity_date <= today {
            cards_month = cards_month.saturating_add(activity.cards_reviewed);
        }
    }

    let average_cards_per_study_day = if study_day_count == 0 {
        0.0
    } else {
        f64::from(cards_on_study_days) / f64::from(study_day_count)
    };

    StudyConsistencyStats {
        current_streak: streak.current_streak,
        longest_streak: streak.longest_streak,
        total_study_days: streak.total_study_days,
        cards_reviewed_today: cards_today,
        cards_reviewed_this_week: cards_week,
        cards_reviewed_this_month: cards_month,
        average_cards_per_study_day,
        minutes_spent_today: minutes_today,
        heatmap: build_heatmap(activities, today),
    }
}

/// Last ~12 months of calendar days ending on `today` (inclusive).
pub fn build_heatmap(activities: &[StudyActivity], today: NaiveDate) -> Vec<HeatmapDay> {
    let start = today - Duration::days(364);
    let mut by_date: std::collections::BTreeMap<NaiveDate, &StudyActivity> =
        std::collections::BTreeMap::new();
    for activity in activities {
        if activity.activity_date >= start && activity.activity_date <= today {
            by_date.insert(activity.activity_date, activity);
        }
    }

    let mut days = Vec::with_capacity(365);
    let mut cursor = start;
    while cursor <= today {
        let (cards, minutes) = by_date
            .get(&cursor)
            .map(|a| (a.cards_reviewed, a.minutes_spent))
            .unwrap_or((0, 0));
        days.push(HeatmapDay {
            date: cursor,
            cards_reviewed: cards,
            minutes_spent: minutes,
            intensity: ActivityIntensity::from_cards(cards),
        });
        cursor += Duration::days(1);
    }
    days
}

impl StudyConsistencyStats {
    pub fn from_streak(streak: StudyStreak) -> Self {
        Self {
            current_streak: streak.current_streak,
            longest_streak: streak.longest_streak,
            total_study_days: streak.total_study_days,
            cards_reviewed_today: 0,
            cards_reviewed_this_week: 0,
            cards_reviewed_this_month: 0,
            average_cards_per_study_day: 0.0,
            minutes_spent_today: 0,
            heatmap: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("date")
    }

    fn activity(date: NaiveDate, cards: u32) -> StudyActivity {
        StudyActivity::new(Uuid::nil(), date, cards, 5)
    }

    #[test]
    fn daily_weekly_monthly_totals() {
        // Friday 2026-10-09; week starts Monday 2026-10-05
        let today = day(2026, 10, 9);
        let activities = vec![
            activity(day(2026, 9, 30), 10), // previous month
            activity(day(2026, 10, 1), 20), // month
            activity(day(2026, 10, 6), 30), // week
            activity(day(2026, 10, 9), 40), // today + week + month
        ];
        let stats = calculate_statistics(&activities, today);
        assert_eq!(stats.cards_reviewed_today, 40);
        assert_eq!(stats.cards_reviewed_this_week, 70); // 30 + 40
        assert_eq!(stats.cards_reviewed_this_month, 90); // 20 + 30 + 40
        assert_eq!(stats.total_study_days, 4);
        assert!((stats.average_cards_per_study_day - 25.0).abs() < f64::EPSILON);
        assert_eq!(stats.minutes_spent_today, 5);
    }

    #[test]
    fn heatmap_covers_365_days_with_empty_cells() {
        let today = day(2026, 10, 10);
        let activities = vec![activity(today, 12)];
        let heat = build_heatmap(&activities, today);
        assert_eq!(heat.len(), 365);
        assert_eq!(heat.last().map(|d| d.cards_reviewed), Some(12));
        assert_eq!(
            heat.last().map(|d| d.intensity),
            Some(ActivityIntensity::Low)
        );
        assert_eq!(heat[0].intensity, ActivityIntensity::None);
    }
}
