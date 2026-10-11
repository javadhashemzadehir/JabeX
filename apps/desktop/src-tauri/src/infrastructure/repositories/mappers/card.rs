use crate::domain::card::{Card, CardState, CardType};
use crate::domain::language_assist::{LanguageAssistOverrides, LanguageAssistStatus};
use crate::domain::rich_text::RichTextDocument;
use crate::infrastructure::repositories::helpers::{
    decode_enum, get_optional_json, get_optional_rfc3339, get_optional_uuid, get_rfc3339, get_u32,
    get_uuid,
};

pub fn map_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    let card_type: CardType = decode_enum(&row.get::<_, String>("card_type")?)?;
    let state: CardState = decode_enum(&row.get::<_, String>("state")?)?;
    let front_raw: String = row.get("front_content")?;
    let back_raw: String = row.get("back_content")?;
    let status_raw: String = row.get("language_assist_status")?;
    let language_assist_status: LanguageAssistStatus =
        decode_enum(&status_raw).unwrap_or(LanguageAssistStatus::Idle);
    let language_assist_overrides: LanguageAssistOverrides =
        match row.get::<_, Option<String>>("language_assist_overrides_json")? {
            None => LanguageAssistOverrides::default(),
            Some(s) if s.trim().is_empty() => LanguageAssistOverrides::default(),
            Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        };

    Ok(Card {
        id: get_uuid(row, "id")?,
        deck_id: get_uuid(row, "deck_id")?,
        card_type,
        front_content: RichTextDocument::parse_stored(&front_raw),
        back_content: RichTextDocument::parse_stored(&back_raw),
        box_position: get_u32(row, "box_position")?,
        due_at: get_rfc3339(row, "due_at")?,
        state,
        review_count: get_u32(row, "review_count")?,
        lapse_count: get_u32(row, "lapse_count")?,
        difficulty: row.get("difficulty")?,
        revision: get_u32(row, "revision")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
        deleted_at: get_optional_rfc3339(row, "deleted_at")?,
        detected_language: row.get("detected_language")?,
        translation: row.get("translation")?,
        pronunciation_audio_path: row.get("pronunciation_audio_path")?,
        pronunciation_ipa: row.get("pronunciation_ipa")?,
        transliteration: get_optional_json(row, "transliteration_json")?,
        language_assist_status,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
        language_assist_overrides,
    })
}
