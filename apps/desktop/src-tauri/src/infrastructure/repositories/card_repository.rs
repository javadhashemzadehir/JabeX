use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::CardRepository;
use crate::infrastructure::database::Database;

use super::helpers::{encode_enum, encode_json, encode_optional_json, require_changed};
use super::mappers::map_card;
use super::queries::cards;

pub struct SqliteCardRepository {
    db: Database,
}

impl SqliteCardRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, card: &Card) -> JabeXResult<()> {
        let card_type = encode_enum(&card.card_type).map_err(JabeXError::from)?;
        let state = encode_enum(&card.state).map_err(JabeXError::from)?;
        let status = encode_enum(&card.language_assist_status).map_err(JabeXError::from)?;
        let transliteration_json = encode_optional_json(&card.transliteration)?;
        let overrides_json = encode_json(&card.language_assist_overrides)?;
        conn.execute(
            cards::CARD_INSERT,
            params![
                card.id.to_string(),
                card.deck_id.to_string(),
                card_type,
                card.front_content.to_storage_string(),
                card.back_content.to_storage_string(),
                card.box_position,
                card.due_at.to_rfc3339(),
                state,
                card.review_count,
                card.lapse_count,
                card.difficulty,
                card.revision,
                card.created_at.to_rfc3339(),
                card.updated_at.to_rfc3339(),
                card.deleted_at.map(|t| t.to_rfc3339()),
                card.detected_language,
                card.translation,
                card.pronunciation_audio_path,
                transliteration_json,
                status,
                card.deleted_by.map(|id| id.to_string()),
                overrides_json,
            ],
        )?;
        Ok(())
    }
}

impl CardRepository for SqliteCardRepository {
    fn create(&self, card: &Card) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, card))
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<Card>> {
        self.db.with_conn(|conn| {
            conn.query_row(cards::CARD_FIND_ACTIVE, params![id.to_string()], map_card)
                .optional()
                .map_err(JabeXError::from)
        })
    }

    fn find_any(&self, id: Uuid) -> JabeXResult<Option<Card>> {
        self.db.with_conn(|conn| {
            conn.query_row(cards::CARD_FIND_ANY, params![id.to_string()], map_card)
                .optional()
                .map_err(JabeXError::from)
        })
    }

    fn list_by_deck(&self, deck_id: Uuid) -> JabeXResult<Vec<Card>> {
        self.list_by_deck_all(deck_id, false)
    }

    fn list_by_deck_page(&self, deck_id: Uuid, limit: u32, offset: u32) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(cards::CARD_LIST_BY_DECK_ACTIVE_PAGE)?;
            let rows = stmt
                .query_map(
                    params![deck_id.to_string(), limit as i64, offset as i64],
                    map_card,
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn count_by_deck(&self, deck_id: Uuid) -> JabeXResult<u32> {
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                cards::CARD_COUNT_BY_DECK_ACTIVE,
                params![deck_id.to_string()],
                |row| row.get(0),
            )?;
            Ok(count.max(0) as u32)
        })
    }

    fn list_by_deck_all(&self, deck_id: Uuid, include_deleted: bool) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                cards::CARD_LIST_BY_DECK_ALL
            } else {
                cards::CARD_LIST_BY_DECK_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map(params![deck_id.to_string()], map_card)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                cards::CARD_LIST_ALL
            } else {
                cards::CARD_LIST_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map([], map_card)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_due(&self, deck_id: Option<Uuid>, now: DateTime<Utc>) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let now_s = now.to_rfc3339();
            if let Some(deck_id) = deck_id {
                let mut stmt = conn.prepare(cards::CARD_LIST_DUE_BY_DECK)?;
                let rows = stmt
                    .query_map(params![deck_id.to_string(), now_s], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            } else {
                let mut stmt = conn.prepare(cards::CARD_LIST_DUE_ALL)?;
                let rows = stmt
                    .query_map(params![now_s], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            }
        })
    }

    fn list_due_limited(
        &self,
        deck_id: Option<Uuid>,
        now: DateTime<Utc>,
        limit: u32,
    ) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let now_s = now.to_rfc3339();
            let limit_i = limit as i64;
            if let Some(deck_id) = deck_id {
                let mut stmt = conn.prepare(cards::CARD_LIST_DUE_BY_DECK_LIMITED)?;
                let rows = stmt
                    .query_map(params![deck_id.to_string(), now_s, limit_i], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            } else {
                let mut stmt = conn.prepare(cards::CARD_LIST_DUE_ALL_LIMITED)?;
                let rows = stmt
                    .query_map(params![now_s, limit_i], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            }
        })
    }

    fn update(&self, card: &Card) -> JabeXResult<()> {
        let card_type = encode_enum(&card.card_type).map_err(JabeXError::from)?;
        let state = encode_enum(&card.state).map_err(JabeXError::from)?;
        let status = encode_enum(&card.language_assist_status).map_err(JabeXError::from)?;
        let transliteration_json = encode_optional_json(&card.transliteration)?;
        let overrides_json = encode_json(&card.language_assist_overrides)?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                cards::CARD_UPDATE,
                params![
                    card.id.to_string(),
                    card.deck_id.to_string(),
                    card_type,
                    card.front_content.to_storage_string(),
                    card.back_content.to_storage_string(),
                    card.box_position,
                    card.due_at.to_rfc3339(),
                    state,
                    card.review_count,
                    card.lapse_count,
                    card.difficulty,
                    card.revision,
                    card.updated_at.to_rfc3339(),
                    card.deleted_at.map(|t| t.to_rfc3339()),
                    card.detected_language,
                    card.translation,
                    card.pronunciation_audio_path,
                    transliteration_json,
                    status,
                    card.deleted_by.map(|id| id.to_string()),
                    overrides_json,
                ],
            )?;
            require_changed(changed, "card", card.id)
        })
    }

    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                cards::CARD_SOFT_DELETE,
                params![
                    id.to_string(),
                    deleted_at.to_rfc3339(),
                    deleted_by.map(|u| u.to_string()),
                ],
            )?;
            require_changed(changed, "card", id)
        })
    }

    fn hard_delete(&self, id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(cards::CARD_HARD_DELETE, params![id.to_string()])?;
            require_changed(changed, "card", id)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::Deck;
    use crate::domain::language_assist::{LanguageAssistStatus, TransliterationMetadata};
    use crate::domain::repositories::DeckRepository;
    use crate::domain::rich_text::RichTextDocument;
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::repositories::SqliteDeckRepository;
    use tempfile::tempdir;

    #[test]
    fn create_and_update_card() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db);

        let deck = Deck::new("Test".into(), None);
        decks.create(&deck).expect("deck");

        let mut card = Card::new(deck.id, "front", "back");
        cards.create(&card).expect("create");

        let found = cards.find(card.id).expect("find").expect("exists");
        assert_eq!(found.front_content.plain_text(), "front");
        assert_eq!(found.language_assist_status, LanguageAssistStatus::Idle);

        card.front_content = RichTextDocument::from_plain_text("updated");
        card.detected_language = Some("en".into());
        card.translation = Some("به روز".into());
        cards.update(&card).expect("update");
        let updated = cards.find(card.id).expect("find").expect("exists");
        assert_eq!(updated.front_content.plain_text(), "updated");
        assert_eq!(updated.detected_language.as_deref(), Some("en"));
        assert_eq!(updated.translation.as_deref(), Some("به روز"));
        assert!(updated
            .front_content
            .to_storage_string()
            .contains("\"version\":2"));
    }

    #[test]
    fn reads_legacy_plain_text_rows() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());

        let deck = Deck::new("Test".into(), None);
        decks.create(&deck).expect("deck");

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO cards (
                    id, deck_id, card_type, front_content, back_content,
                    box_position, due_at, state, review_count, lapse_count,
                    difficulty, revision, created_at, updated_at, deleted_at
                 ) VALUES (?1, ?2, 'basic', 'legacy front', 'legacy back',
                    1, '2020-01-01T00:00:00Z', 'new', 0, 0, NULL, 1,
                    '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z', NULL)",
                params![Uuid::now_v7().to_string(), deck.id.to_string()],
            )?;
            Ok(())
        })
        .expect("insert legacy");

        let listed = cards.list_by_deck(deck.id).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].front_content.plain_text(), "legacy front");
        assert_eq!(listed[0].back_content.plain_text(), "legacy back");
    }

    #[test]
    fn persists_transliteration_metadata() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db);

        let deck = Deck::new("Test".into(), None);
        decks.create(&deck).expect("deck");

        let mut card = Card::new(deck.id, "کامپیوتر", "computer");
        card.transliteration = Some(TransliterationMetadata {
            is_transliteration: true,
            source_script: "arab".into(),
            resolved_word: Some("computer".into()),
            resolved_language: Some("en".into()),
            confidence: 0.95,
        });
        card.language_assist_status = LanguageAssistStatus::Ready;
        cards.create(&card).expect("create");

        let found = cards.find(card.id).expect("find").expect("exists");
        let meta = found.transliteration.expect("meta");
        assert_eq!(meta.resolved_word.as_deref(), Some("computer"));
        assert_eq!(found.language_assist_status, LanguageAssistStatus::Ready);
    }
}
