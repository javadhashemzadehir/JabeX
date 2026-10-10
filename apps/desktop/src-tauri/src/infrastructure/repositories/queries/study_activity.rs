pub const STUDY_ACTIVITY_INSERT: &str = "INSERT INTO study_activity (
    id, profile_id, activity_date, cards_reviewed, minutes_spent, created_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

pub const STUDY_ACTIVITY_UPSERT_ADD: &str = "INSERT INTO study_activity (
    id, profile_id, activity_date, cards_reviewed, minutes_spent, created_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(profile_id, activity_date) DO UPDATE SET
    cards_reviewed = cards_reviewed + excluded.cards_reviewed,
    minutes_spent = minutes_spent + excluded.minutes_spent";

pub const STUDY_ACTIVITY_FIND_BY_DATE: &str =
    "SELECT id, profile_id, activity_date, cards_reviewed,
        minutes_spent, created_at
 FROM study_activity
 WHERE profile_id = ?1 AND activity_date = ?2
 LIMIT 1";

pub const STUDY_ACTIVITY_LIST_BY_PROFILE: &str =
    "SELECT id, profile_id, activity_date, cards_reviewed,
        minutes_spent, created_at
 FROM study_activity
 WHERE profile_id = ?1
 ORDER BY activity_date ASC";

pub const STUDY_ACTIVITY_LIST_RANGE: &str = "SELECT id, profile_id, activity_date, cards_reviewed,
        minutes_spent, created_at
 FROM study_activity
 WHERE profile_id = ?1
   AND activity_date >= ?2
   AND activity_date <= ?3
 ORDER BY activity_date ASC";

pub const STUDY_ACTIVITY_COUNT: &str = "SELECT COUNT(*) FROM study_activity WHERE profile_id = ?1";
