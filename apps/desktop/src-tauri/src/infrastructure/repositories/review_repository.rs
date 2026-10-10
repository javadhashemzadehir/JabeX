use rusqlite::params;
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::ReviewRepository;
use crate::domain::review::{Grade, Review};
use crate::infrastructure::database::Database;

use super::helpers::{decode_enum, encode_enum, parse_time, parse_uuid};

pub struct SqliteReviewRepository {
    db: Database,
}

impl SqliteReviewRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl ReviewRepository for SqliteReviewRepository {
    fn create(&self, review: &Review) -> JabeXResult<()> {
        let grade = encode_enum(&review.grade).map_err(JabeXError::from)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO reviews (
                    id, card_id, grade, reviewed_at, duration_ms,
                    box_before, box_after, revision, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    review.id.to_string(),
                    review.card_id.to_string(),
                    grade,
                    review.reviewed_at.to_rfc3339(),
                    review.duration_ms,
                    review.box_before,
                    review.box_after,
                    review.revision,
                    review.created_at.to_rfc3339(),
                    review.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    fn list_by_card(&self, card_id: Uuid) -> JabeXResult<Vec<Review>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, card_id, grade, reviewed_at, duration_ms,
                        box_before, box_after, revision, created_at, updated_at
                 FROM reviews
                 WHERE card_id = ?1
                 ORDER BY reviewed_at ASC",
            )?;
            let rows = stmt
                .query_map(params![card_id.to_string()], map_review)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_all(&self) -> JabeXResult<Vec<Review>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, card_id, grade, reviewed_at, duration_ms,
                        box_before, box_after, revision, created_at, updated_at
                 FROM reviews
                 ORDER BY reviewed_at ASC",
            )?;
            let rows = stmt
                .query_map([], map_review)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

fn map_review(row: &rusqlite::Row<'_>) -> rusqlite::Result<Review> {
    let grade: Grade = decode_enum(&row.get::<_, String>(2)?)?;
    Ok(Review {
        id: parse_uuid(&row.get::<_, String>(0)?)?,
        card_id: parse_uuid(&row.get::<_, String>(1)?)?,
        grade,
        reviewed_at: parse_time(&row.get::<_, String>(3)?)?,
        duration_ms: row.get::<_, Option<i64>>(4)?.map(|v| v as u32),
        box_before: row.get::<_, i64>(5)? as u32,
        box_after: row.get::<_, i64>(6)? as u32,
        revision: row.get::<_, i64>(7)? as u32,
        created_at: parse_time(&row.get::<_, String>(8)?)?,
        updated_at: parse_time(&row.get::<_, String>(9)?)?,
    })
}
