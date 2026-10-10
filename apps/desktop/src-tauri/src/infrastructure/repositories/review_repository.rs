use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::ReviewRepository;
use crate::domain::review::Review;
use crate::infrastructure::database::Database;

use super::helpers::encode_enum;
use super::mappers::map_review;
use super::queries::reviews;

pub struct SqliteReviewRepository {
    db: Database,
}

impl SqliteReviewRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, review: &Review) -> JabeXResult<()> {
        let grade = encode_enum(&review.grade).map_err(JabeXError::from)?;
        conn.execute(
            reviews::REVIEW_INSERT,
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
    }
}

impl ReviewRepository for SqliteReviewRepository {
    fn create(&self, review: &Review) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, review))
    }

    fn list_by_card(&self, card_id: Uuid) -> JabeXResult<Vec<Review>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(reviews::REVIEW_LIST_BY_CARD)?;
            let rows = stmt
                .query_map(params![card_id.to_string()], map_review)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_all(&self) -> JabeXResult<Vec<Review>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(reviews::REVIEW_LIST_ALL)?;
            let rows = stmt
                .query_map([], map_review)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn hard_delete_by_card(&self, card_id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                reviews::REVIEW_HARD_DELETE_BY_CARD,
                params![card_id.to_string()],
            )?;
            Ok(())
        })
    }
}
