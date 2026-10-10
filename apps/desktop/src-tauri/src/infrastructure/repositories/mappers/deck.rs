use crate::domain::deck::{Deck, DeckSettings};
use crate::infrastructure::repositories::helpers::{
    get_optional_rfc3339, get_optional_uuid, get_rfc3339, get_u32, get_uuid,
};

pub fn map_deck(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deck> {
    let settings_raw: String = row.get("settings")?;
    let settings: DeckSettings = serde_json::from_str(&settings_raw).unwrap_or_default();
    Ok(Deck {
        id: get_uuid(row, "id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        settings,
        revision: get_u32(row, "revision")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
        deleted_at: get_optional_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
    })
}
