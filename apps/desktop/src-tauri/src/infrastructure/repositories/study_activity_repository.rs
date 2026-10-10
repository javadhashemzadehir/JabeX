use chrono::NaiveDate;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::StudyActivityRepository;
use crate::domain::study_activity::{format_activity_date, StudyActivity};
use crate::infrastructure::database::Database;
use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

use super::mappers::map_study_activity;
use super::queries::study_activity as q;

pub struct SqliteStudyActivityRepository {
    db: Database,
}

impl SqliteStudyActivityRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, activity: &StudyActivity) -> JabeXResult<()> {
        // Persist minutes as seconds so short reviews accumulate accurately.
        let duration_seconds = activity.minutes_spent.saturating_mul(60);
        conn.execute(
            q::STUDY_ACTIVITY_INSERT,
            params![
                activity.id.to_string(),
                activity.profile_id.to_string(),
                format_activity_date(activity.activity_date),
                activity.cards_reviewed,
                duration_seconds,
                activity.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}

impl StudyActivityRepository for SqliteStudyActivityRepository {
    fn upsert_add(
        &self,
        profile_id: Uuid,
        activity_date: NaiveDate,
        cards_delta: u32,
        duration_seconds: u32,
    ) -> JabeXResult<StudyActivity> {
        self.db.with_conn(|conn| {
            let id = create_id();
            let created_at = now_utc();
            conn.execute(
                q::STUDY_ACTIVITY_UPSERT_ADD,
                params![
                    id.to_string(),
                    profile_id.to_string(),
                    format_activity_date(activity_date),
                    cards_delta,
                    duration_seconds,
                    created_at.to_rfc3339(),
                ],
            )?;

            let mut stmt = conn.prepare(q::STUDY_ACTIVITY_FIND_BY_DATE)?;
            let activity = stmt
                .query_row(
                    params![profile_id.to_string(), format_activity_date(activity_date)],
                    map_study_activity,
                )
                .map_err(|e| JabeXError::Database(e.to_string()))?;
            Ok(activity)
        })
    }

    fn find_by_date(
        &self,
        profile_id: Uuid,
        activity_date: NaiveDate,
    ) -> JabeXResult<Option<StudyActivity>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::STUDY_ACTIVITY_FIND_BY_DATE)?;
            let mut rows = stmt.query_map(
                params![profile_id.to_string(), format_activity_date(activity_date)],
                map_study_activity,
            )?;
            match rows.next() {
                Some(row) => Ok(Some(row?)),
                None => Ok(None),
            }
        })
    }

    fn list_by_profile(&self, profile_id: Uuid) -> JabeXResult<Vec<StudyActivity>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::STUDY_ACTIVITY_LIST_BY_PROFILE)?;
            let rows = stmt
                .query_map(params![profile_id.to_string()], map_study_activity)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_range(
        &self,
        profile_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> JabeXResult<Vec<StudyActivity>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(q::STUDY_ACTIVITY_LIST_RANGE)?;
            let rows = stmt
                .query_map(
                    params![
                        profile_id.to_string(),
                        format_activity_date(from),
                        format_activity_date(to)
                    ],
                    map_study_activity,
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn count(&self, profile_id: Uuid) -> JabeXResult<u32> {
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                q::STUDY_ACTIVITY_COUNT,
                params![profile_id.to_string()],
                |row| row.get(0),
            )?;
            Ok(count as u32)
        })
    }

    fn create(&self, activity: &StudyActivity) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, activity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::StudyActivityRepository;
    use crate::infrastructure::database::open_database_at;
    use chrono::NaiveDate;
    use tempfile::tempdir;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("date")
    }

    #[test]
    fn insert_query_range_and_aggregate() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let repo = SqliteStudyActivityRepository::new(db);
        let profile = Uuid::nil();

        let a = repo
            .upsert_add(profile, date(2026, 10, 10), 10, 90)
            .expect("upsert");
        assert_eq!(a.cards_reviewed, 10);
        assert_eq!(a.minutes_spent, 1); // 90 seconds → 1 minute

        let b = repo
            .upsert_add(profile, date(2026, 10, 10), 5, 30)
            .expect("upsert2");
        assert_eq!(b.cards_reviewed, 15);
        assert_eq!(b.minutes_spent, 2); // 120 seconds

        repo.upsert_add(profile, date(2026, 10, 11), 3, 0)
            .expect("day2");

        let range = repo
            .list_range(profile, date(2026, 10, 10), date(2026, 10, 11))
            .expect("range");
        assert_eq!(range.len(), 2);
        assert_eq!(repo.count(profile).expect("count"), 2);

        let found = repo
            .find_by_date(profile, date(2026, 10, 10))
            .expect("find")
            .expect("some");
        assert_eq!(found.cards_reviewed, 15);
    }
}
