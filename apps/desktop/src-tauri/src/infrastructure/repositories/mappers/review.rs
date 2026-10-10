use crate::domain::review::{Grade, Review};
use crate::infrastructure::repositories::helpers::{decode_enum, get_rfc3339, get_u32, get_uuid};

pub fn map_review(row: &rusqlite::Row<'_>) -> rusqlite::Result<Review> {
    let grade: Grade = decode_enum(&row.get::<_, String>("grade")?)?;
    Ok(Review {
        id: get_uuid(row, "id")?,
        card_id: get_uuid(row, "card_id")?,
        grade,
        reviewed_at: get_rfc3339(row, "reviewed_at")?,
        duration_ms: row.get::<_, Option<i64>>("duration_ms")?.map(|v| v as u32),
        box_before: get_u32(row, "box_before")?,
        box_after: get_u32(row, "box_after")?,
        revision: get_u32(row, "revision")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
    })
}
