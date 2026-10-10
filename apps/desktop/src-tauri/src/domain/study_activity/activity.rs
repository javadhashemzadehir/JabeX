use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// One local calendar day of Leitner review activity for a profile.
///
/// `minutes_spent` is whole minutes (floor of accumulated review duration).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyActivity {
    pub id: Uuid,
    pub profile_id: Uuid,
    /// Local calendar day (`YYYY-MM-DD`).
    pub activity_date: NaiveDate,
    pub cards_reviewed: u32,
    pub minutes_spent: u32,
    pub created_at: DateTime<Utc>,
}

impl StudyActivity {
    pub fn new(
        profile_id: Uuid,
        activity_date: NaiveDate,
        cards_reviewed: u32,
        minutes_spent: u32,
    ) -> Self {
        let now = now_utc();
        Self {
            id: create_id(),
            profile_id,
            activity_date,
            cards_reviewed,
            minutes_spent,
            created_at: now,
        }
    }
}

/// GitHub-style heatmap bucket for a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityIntensity {
    /// 0 cards
    None,
    /// 1–20 cards
    Low,
    /// 21–99 cards
    Medium,
    /// 100+ cards
    High,
}

impl ActivityIntensity {
    pub fn from_cards(cards_reviewed: u32) -> Self {
        match cards_reviewed {
            0 => Self::None,
            1..=20 => Self::Low,
            21..=99 => Self::Medium,
            _ => Self::High,
        }
    }
}

/// Format a [`NaiveDate`] as `YYYY-MM-DD` for SQLite TEXT storage.
pub fn format_activity_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Parse `YYYY-MM-DD` activity dates from SQLite.
pub fn parse_activity_date(raw: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intensity_buckets() {
        assert_eq!(ActivityIntensity::from_cards(0), ActivityIntensity::None);
        assert_eq!(ActivityIntensity::from_cards(1), ActivityIntensity::Low);
        assert_eq!(ActivityIntensity::from_cards(20), ActivityIntensity::Low);
        assert_eq!(ActivityIntensity::from_cards(21), ActivityIntensity::Medium);
        assert_eq!(ActivityIntensity::from_cards(99), ActivityIntensity::Medium);
        assert_eq!(ActivityIntensity::from_cards(100), ActivityIntensity::High);
    }
}
