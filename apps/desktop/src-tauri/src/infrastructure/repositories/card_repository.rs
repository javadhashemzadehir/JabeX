use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::domain::card::{Card, CardState, CardType};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::CardRepository;
use crate::infrastructure::database::Database;

use super::helpers::{decode_enum, encode_enum, parse_time, parse_uuid};

pub struct SqliteCardRepository {
    db: Database,
}

impl SqliteCardRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl CardRepository for SqliteCardRepository {
    fn create(&self, card: &Card) -> JabeXResult<()> {
        let card_type = encode_enum(&card.card_type).map_err(JabeXError::from)?;
        let state = encode_enum(&card.state).map_err(JabeXError::from)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO cards (
                    id, deck_id, card_type, front_content, back_content,
                    box_position, due_at, state, review_count, lapse_count,
                    difficulty, revision, created_at, updated_at, deleted_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    card.id.to_string(),
                    card.deck_id.to_string(),
                    card_type,
                    card.front_content,
                    card.back_content,
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
                ],
            )?;
            Ok(())
        })
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<Card>> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, deck_id, card_type, front_content, back_content,
                        box_position, due_at, state, review_count, lapse_count,
                        difficulty, revision, created_at, updated_at, deleted_at
                 FROM cards WHERE id = ?1 AND deleted_at IS NULL",
                params![id.to_string()],
                map_card,
            )
            .optional()
            .map_err(JabeXError::from)
        })
    }

    fn list_by_deck(&self, deck_id: Uuid) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, deck_id, card_type, front_content, back_content,
                        box_position, due_at, state, review_count, lapse_count,
                        difficulty, revision, created_at, updated_at, deleted_at
                 FROM cards
                 WHERE deck_id = ?1 AND deleted_at IS NULL
                 ORDER BY created_at ASC",
            )?;
            let rows = stmt
                .query_map(params![deck_id.to_string()], map_card)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_due(&self, deck_id: Option<Uuid>, now: DateTime<Utc>) -> JabeXResult<Vec<Card>> {
        self.db.with_conn(|conn| {
            let now_s = now.to_rfc3339();
            if let Some(deck_id) = deck_id {
                let mut stmt = conn.prepare(
                    "SELECT id, deck_id, card_type, front_content, back_content,
                            box_position, due_at, state, review_count, lapse_count,
                            difficulty, revision, created_at, updated_at, deleted_at
                     FROM cards
                     WHERE deck_id = ?1 AND deleted_at IS NULL AND due_at <= ?2
                       AND state != 'suspended'
                     ORDER BY due_at ASC",
                )?;
                let rows = stmt
                    .query_map(params![deck_id.to_string(), now_s], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, deck_id, card_type, front_content, back_content,
                            box_position, due_at, state, review_count, lapse_count,
                            difficulty, revision, created_at, updated_at, deleted_at
                     FROM cards
                     WHERE deleted_at IS NULL AND due_at <= ?1
                       AND state != 'suspended'
                     ORDER BY due_at ASC",
                )?;
                let rows = stmt
                    .query_map(params![now_s], map_card)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            }
        })
    }

    fn update(&self, card: &Card) -> JabeXResult<()> {
        let card_type = encode_enum(&card.card_type).map_err(JabeXError::from)?;
        let state = encode_enum(&card.state).map_err(JabeXError::from)?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE cards SET
                    deck_id = ?2,
                    card_type = ?3,
                    front_content = ?4,
                    back_content = ?5,
                    box_position = ?6,
                    due_at = ?7,
                    state = ?8,
                    review_count = ?9,
                    lapse_count = ?10,
                    difficulty = ?11,
                    revision = ?12,
                    updated_at = ?13,
                    deleted_at = ?14
                 WHERE id = ?1",
                params![
                    card.id.to_string(),
                    card.deck_id.to_string(),
                    card_type,
                    card.front_content,
                    card.back_content,
                    card.box_position,
                    card.due_at.to_rfc3339(),
                    state,
                    card.review_count,
                    card.lapse_count,
                    card.difficulty,
                    card.revision,
                    card.updated_at.to_rfc3339(),
                    card.deleted_at.map(|t| t.to_rfc3339()),
                ],
            )?;
            if changed == 0 {
                return Err(JabeXError::NotFound {
                    entity: "card",
                    id: card.id,
                });
            }
            Ok(())
        })
    }

    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE cards
                 SET deleted_at = ?2, updated_at = ?2, revision = revision + 1
                 WHERE id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), deleted_at.to_rfc3339()],
            )?;
            if changed == 0 {
                return Err(JabeXError::NotFound { entity: "card", id });
            }
            Ok(())
        })
    }
}

fn map_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    let card_type: CardType = decode_enum(&row.get::<_, String>(2)?)?;
    let state: CardState = decode_enum(&row.get::<_, String>(7)?)?;
    Ok(Card {
        id: parse_uuid(&row.get::<_, String>(0)?)?,
        deck_id: parse_uuid(&row.get::<_, String>(1)?)?,
        card_type,
        front_content: row.get(3)?,
        back_content: row.get(4)?,
        box_position: row.get::<_, i64>(5)? as u32,
        due_at: parse_time(&row.get::<_, String>(6)?)?,
        state,
        review_count: row.get::<_, i64>(8)? as u32,
        lapse_count: row.get::<_, i64>(9)? as u32,
        difficulty: row.get(10)?,
        revision: row.get::<_, i64>(11)? as u32,
        created_at: parse_time(&row.get::<_, String>(12)?)?,
        updated_at: parse_time(&row.get::<_, String>(13)?)?,
        deleted_at: row
            .get::<_, Option<String>>(14)?
            .map(|s| parse_time(&s))
            .transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::Deck;
    use crate::domain::repositories::DeckRepository;
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

        let mut card = Card::new(deck.id, "front".into(), "back".into());
        cards.create(&card).expect("create");

        let found = cards.find(card.id).expect("find").expect("exists");
        assert_eq!(found.front_content, "front");

        card.front_content = "updated".into();
        cards.update(&card).expect("update");
        let updated = cards.find(card.id).expect("find").expect("exists");
        assert_eq!(updated.front_content, "updated");
    }
}
