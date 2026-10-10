use crate::domain::rich_text::RichTextDocument;
use crate::domain::trash::{TrashEntityKind, TrashItem};
use crate::infrastructure::repositories::helpers::{get_optional_uuid, get_rfc3339, get_uuid};

pub fn map_trash_deck(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrashItem> {
    Ok(TrashItem {
        entity_kind: TrashEntityKind::Deck,
        id: get_uuid(row, "id")?,
        title: row.get("name")?,
        deleted_at: get_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
        parent_id: None,
        parent_kind: None,
    })
}

pub fn map_trash_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrashItem> {
    let front_raw: String = row.get("front_content")?;
    let title = truncate_title(&RichTextDocument::parse_stored(&front_raw).plain_text());
    Ok(TrashItem {
        entity_kind: TrashEntityKind::Card,
        id: get_uuid(row, "id")?,
        title,
        deleted_at: get_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
        parent_id: Some(get_uuid(row, "deck_id")?),
        parent_kind: Some(TrashEntityKind::Deck),
    })
}

pub fn map_trash_attachment(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrashItem> {
    Ok(TrashItem {
        entity_kind: TrashEntityKind::Attachment,
        id: get_uuid(row, "id")?,
        title: row.get("file_name")?,
        deleted_at: get_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
        parent_id: Some(get_uuid(row, "owner_id")?),
        parent_kind: Some(TrashEntityKind::Card),
    })
}

pub fn map_trash_ink(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrashItem> {
    let path: String = row.get("relative_path")?;
    let title = path.rsplit('/').next().unwrap_or("ink").to_string();
    Ok(TrashItem {
        entity_kind: TrashEntityKind::InkDocument,
        id: get_uuid(row, "id")?,
        title,
        deleted_at: get_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
        parent_id: Some(get_uuid(row, "owner_id")?),
        parent_kind: Some(TrashEntityKind::Card),
    })
}

pub fn truncate_title(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "Untitled card".into();
    }
    let mut chars = trimmed.chars();
    let preview: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}
