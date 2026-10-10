use chrono::{Local, TimeZone, Utc};
use rusqlite::params;

use crate::domain::card::Card;
use crate::domain::errors::JabeXResult;
use crate::domain::stats::StudyStats;
use crate::infrastructure::database::Database;
use crate::shared::time::now_utc;

/// Reads aggregate study stats from the active profile database.
pub struct StatsService {
    db: Database,
}

impl StatsService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get_study_stats(&self) -> JabeXResult<StudyStats> {
        let now = now_utc();
        let today_start = start_of_local_day(now);

        self.db.with_conn(|conn| {
            let deck_count: u32 = conn.query_row(
                "SELECT COUNT(*) FROM decks WHERE deleted_at IS NULL",
                [],
                |row| row.get::<_, i64>(0).map(|v| v as u32),
            )?;

            let card_count: u32 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE deleted_at IS NULL",
                [],
                |row| row.get::<_, i64>(0).map(|v| v as u32),
            )?;

            let due_count: u32 = conn.query_row(
                "SELECT COUNT(*) FROM cards
                 WHERE deleted_at IS NULL
                   AND due_at <= ?1
                   AND state != 'suspended'",
                params![now.to_rfc3339()],
                |row| row.get::<_, i64>(0).map(|v| v as u32),
            )?;

            let reviews_today: u32 = conn.query_row(
                "SELECT COUNT(*) FROM reviews WHERE reviewed_at >= ?1",
                params![today_start.to_rfc3339()],
                |row| row.get::<_, i64>(0).map(|v| v as u32),
            )?;

            let mut cards_by_box = [0u32; Card::MAX_BOX as usize];
            let mut stmt = conn.prepare(
                "SELECT box_position, COUNT(*)
                 FROM cards
                 WHERE deleted_at IS NULL
                 GROUP BY box_position",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)? as u32, row.get::<_, i64>(1)? as u32))
            })?;

            for row in rows {
                let (box_position, count) = row?;
                let clamped = Card::clamp_box(box_position);
                let idx = (clamped - Card::MIN_BOX) as usize;
                cards_by_box[idx] = cards_by_box[idx].saturating_add(count);
            }

            Ok(StudyStats {
                deck_count,
                card_count,
                due_count,
                reviews_today,
                cards_by_box,
            })
        })
    }
}

/// Start of the user's local calendar day, as UTC (for RFC3339 comparisons).
fn start_of_local_day(now: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    let local = now.with_timezone(&Local);
    let date = local.date_naive();
    match date
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
    {
        Some(start) => start.with_timezone(&Utc),
        None => {
            // Rare DST gap/overlap: fall back to UTC midnight.
            date.and_hms_opt(0, 0, 0)
                .map(|naive| naive.and_utc())
                .unwrap_or(now)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::Deck;
    use crate::domain::repositories::{CardRepository, DeckRepository, ReviewRepository};
    use crate::domain::review::{Grade, Review};
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::repositories::{
        SqliteCardRepository, SqliteDeckRepository, SqliteReviewRepository,
    };
    use crate::shared::ids::create_id;
    use tempfile::tempdir;

    #[test]
    fn aggregates_counts_and_box_distribution() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let reviews = SqliteReviewRepository::new(db.clone());
        let stats = StatsService::new(db);

        let deck = Deck::new("Persian".into(), None);
        decks.create(&deck).expect("deck");

        let mut a = Card::new(deck.id, "a", "1");
        a.box_position = 1;
        cards.create(&a).expect("card a");

        let mut b = Card::new(deck.id, "b", "2");
        b.box_position = 3;
        cards.create(&b).expect("card b");

        let mut c = Card::new(deck.id, "c", "3");
        c.box_position = 3;
        c.state = crate::domain::card::CardState::Suspended;
        // Suspended cards still count toward totals / box, but not due.
        c.due_at = now_utc() - chrono::Duration::days(1);
        cards.create(&c).expect("card c");

        let review = Review {
            id: create_id(),
            card_id: a.id,
            grade: Grade::Good,
            reviewed_at: now_utc(),
            duration_ms: Some(1200),
            box_before: 1,
            box_after: 2,
            revision: 1,
            created_at: now_utc(),
            updated_at: now_utc(),
        };
        reviews.create(&review).expect("review");

        let result = stats.get_study_stats().expect("stats");
        assert_eq!(result.deck_count, 1);
        assert_eq!(result.card_count, 3);
        // a and b are due (new cards due now); c is suspended
        assert_eq!(result.due_count, 2);
        assert_eq!(result.reviews_today, 1);
        assert_eq!(result.cards_by_box, [1, 0, 2, 0, 0]);
    }
}
