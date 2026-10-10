use crate::domain::study_activity::{parse_activity_date, StudyActivity};
use crate::infrastructure::repositories::helpers::{get_rfc3339, get_u32, get_uuid};

pub fn map_study_activity(row: &rusqlite::Row<'_>) -> rusqlite::Result<StudyActivity> {
    let date_raw: String = row.get("activity_date")?;
    let activity_date = parse_activity_date(&date_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    // Column stores accumulated seconds; domain exposes whole minutes.
    let duration_seconds = get_u32(row, "minutes_spent")?;
    Ok(StudyActivity {
        id: get_uuid(row, "id")?,
        profile_id: get_uuid(row, "profile_id")?,
        activity_date,
        cards_reviewed: get_u32(row, "cards_reviewed")?,
        minutes_spent: duration_seconds / 60,
        created_at: get_rfc3339(row, "created_at")?,
    })
}
