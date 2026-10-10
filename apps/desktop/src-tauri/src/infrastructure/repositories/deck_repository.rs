use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::domain::deck::{Deck, DeckSettings};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::DeckRepository;
use crate::infrastructure::database::Database;

use super::helpers::{parse_time, parse_uuid};

pub struct SqliteDeckRepository {
    db: Database,
}

impl SqliteDeckRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl DeckRepository for SqliteDeckRepository {
    fn create(&self, deck: &Deck) -> JabeXResult<()> {
        let settings = serde_json::to_string(&deck.settings)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO decks (id, name, description, settings, revision, created_at, updated_at, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    deck.id.to_string(),
                    deck.name,
                    deck.description,
                    settings,
                    deck.revision,
                    deck.created_at.to_rfc3339(),
                    deck.updated_at.to_rfc3339(),
                    deck.deleted_at.map(|t| t.to_rfc3339()),
                ],
            )?;
            Ok(())
        })
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<Deck>> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, description, settings, revision, created_at, updated_at, deleted_at
                 FROM decks WHERE id = ?1 AND deleted_at IS NULL",
                params![id.to_string()],
                map_deck,
            )
            .optional()
            .map_err(JabeXError::from)
        })
    }

    fn list(&self) -> JabeXResult<Vec<Deck>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, settings, revision, created_at, updated_at, deleted_at
                 FROM decks WHERE deleted_at IS NULL ORDER BY created_at ASC",
            )?;
            let rows = stmt
                .query_map([], map_deck)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn update(&self, deck: &Deck) -> JabeXResult<()> {
        let settings = serde_json::to_string(&deck.settings)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE decks
                 SET name = ?2, description = ?3, settings = ?4, revision = ?5,
                     updated_at = ?6, deleted_at = ?7
                 WHERE id = ?1",
                params![
                    deck.id.to_string(),
                    deck.name,
                    deck.description,
                    settings,
                    deck.revision,
                    deck.updated_at.to_rfc3339(),
                    deck.deleted_at.map(|t| t.to_rfc3339()),
                ],
            )?;
            if changed == 0 {
                return Err(JabeXError::NotFound {
                    entity: "deck",
                    id: deck.id,
                });
            }
            Ok(())
        })
    }

    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE decks
                 SET deleted_at = ?2, updated_at = ?2, revision = revision + 1
                 WHERE id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), deleted_at.to_rfc3339()],
            )?;
            if changed == 0 {
                return Err(JabeXError::NotFound { entity: "deck", id });
            }
            Ok(())
        })
    }
}

fn map_deck(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deck> {
    let settings_raw: String = row.get(3)?;
    let settings: DeckSettings = serde_json::from_str(&settings_raw).unwrap_or_default();
    Ok(Deck {
        id: parse_uuid(&row.get::<_, String>(0)?)?,
        name: row.get(1)?,
        description: row.get(2)?,
        settings,
        revision: row.get::<_, i64>(4)? as u32,
        created_at: parse_time(&row.get::<_, String>(5)?)?,
        updated_at: parse_time(&row.get::<_, String>(6)?)?,
        deleted_at: row
            .get::<_, Option<String>>(7)?
            .map(|s| parse_time(&s))
            .transpose()?,
    })
}
