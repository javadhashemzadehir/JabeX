use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::deck::Deck;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::DeckRepository;
use crate::infrastructure::database::Database;

use super::helpers::require_changed;
use super::mappers::map_deck;
use super::queries::decks;

pub struct SqliteDeckRepository {
    db: Database,
}

impl SqliteDeckRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, deck: &Deck) -> JabeXResult<()> {
        let settings = serde_json::to_string(&deck.settings)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        conn.execute(
            decks::DECK_INSERT,
            params![
                deck.id.to_string(),
                deck.name,
                deck.description,
                settings,
                deck.revision,
                deck.created_at.to_rfc3339(),
                deck.updated_at.to_rfc3339(),
                deck.deleted_at.map(|t| t.to_rfc3339()),
                deck.deleted_by.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }
}

impl DeckRepository for SqliteDeckRepository {
    fn create(&self, deck: &Deck) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, deck))
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<Deck>> {
        self.db.with_conn(|conn| {
            conn.query_row(decks::DECK_FIND_ACTIVE, params![id.to_string()], map_deck)
                .optional()
                .map_err(JabeXError::from)
        })
    }

    fn find_any(&self, id: Uuid) -> JabeXResult<Option<Deck>> {
        self.db.with_conn(|conn| {
            conn.query_row(decks::DECK_FIND_ANY, params![id.to_string()], map_deck)
                .optional()
                .map_err(JabeXError::from)
        })
    }

    fn list(&self) -> JabeXResult<Vec<Deck>> {
        self.list_all(false)
    }

    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Deck>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                decks::DECK_LIST_ALL
            } else {
                decks::DECK_LIST_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
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
                decks::DECK_UPDATE,
                params![
                    deck.id.to_string(),
                    deck.name,
                    deck.description,
                    settings,
                    deck.revision,
                    deck.updated_at.to_rfc3339(),
                    deck.deleted_at.map(|t| t.to_rfc3339()),
                    deck.deleted_by.map(|id| id.to_string()),
                ],
            )?;
            require_changed(changed, "deck", deck.id)
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
                decks::DECK_SOFT_DELETE,
                params![
                    id.to_string(),
                    deleted_at.to_rfc3339(),
                    deleted_by.map(|u| u.to_string()),
                ],
            )?;
            require_changed(changed, "deck", id)
        })
    }

    fn hard_delete(&self, id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(decks::DECK_HARD_DELETE, params![id.to_string()])?;
            require_changed(changed, "deck", id)
        })
    }
}
