use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::trash::{TrashEntityKind, TrashItem, TrashRepository};
use crate::infrastructure::database::Database;

use super::helpers::require_changed;
use super::mappers::{map_trash_attachment, map_trash_card, map_trash_deck, map_trash_ink};
use super::queries::trash;

pub struct SqliteTrashRepository {
    db: Database,
}

impl SqliteTrashRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn restore_sql(kind: TrashEntityKind) -> &'static str {
        match kind {
            TrashEntityKind::Deck => trash::RESTORE_DECK,
            TrashEntityKind::Card => trash::RESTORE_CARD,
            TrashEntityKind::Attachment => trash::RESTORE_ATTACHMENT,
            TrashEntityKind::InkDocument => trash::RESTORE_INK,
        }
    }
}

impl TrashRepository for SqliteTrashRepository {
    fn list_deleted_items(&self) -> JabeXResult<Vec<TrashItem>> {
        let mut items = Vec::new();

        self.db.with_conn(|conn| {
            let mut deck_stmt = conn.prepare(trash::LIST_DELETED_DECKS)?;
            for row in deck_stmt.query_map([], map_trash_deck)? {
                items.push(row?);
            }

            let mut card_stmt = conn.prepare(trash::LIST_DELETED_CARDS)?;
            for row in card_stmt.query_map([], map_trash_card)? {
                items.push(row?);
            }

            let mut att_stmt = conn.prepare(trash::LIST_DELETED_ATTACHMENTS)?;
            for row in att_stmt.query_map([], map_trash_attachment)? {
                items.push(row?);
            }

            let mut ink_stmt = conn.prepare(trash::LIST_DELETED_INK)?;
            for row in ink_stmt.query_map([], map_trash_ink)? {
                items.push(row?);
            }

            Ok(())
        })?;

        items.sort_by_key(|a| std::cmp::Reverse(a.deleted_at));
        Ok(items)
    }

    fn restore_item(&self, kind: TrashEntityKind, id: Uuid, at: DateTime<Utc>) -> JabeXResult<()> {
        self.db.with_transaction(|tx| {
            let at_s = at.to_rfc3339();
            match kind {
                TrashEntityKind::Deck => {
                    let changed = tx.execute(
                        Self::restore_sql(TrashEntityKind::Deck),
                        params![id.to_string(), at_s],
                    )?;
                    require_changed(changed, "deck", id)?;
                    let card_ids: Vec<String> = {
                        let mut stmt = tx.prepare(trash::CARD_IDS_DELETED_IN_DECK)?;
                        let rows = stmt
                            .query_map(params![id.to_string()], |row| row.get(0))?
                            .collect::<Result<Vec<_>, _>>()?;
                        rows
                    };
                    for card_id in &card_ids {
                        tx.execute(
                            Self::restore_sql(TrashEntityKind::Card),
                            params![card_id, at_s],
                        )?;
                        restore_card_children(tx, card_id, &at_s)?;
                    }
                }
                TrashEntityKind::Card => {
                    let changed = tx.execute(
                        Self::restore_sql(TrashEntityKind::Card),
                        params![id.to_string(), at_s],
                    )?;
                    require_changed(changed, "card", id)?;
                    let deck_id: String =
                        tx.query_row(trash::CARD_DECK_ID, params![id.to_string()], |row| {
                            row.get(0)
                        })?;
                    tx.execute(
                        Self::restore_sql(TrashEntityKind::Deck),
                        params![deck_id, at_s],
                    )?;
                    restore_card_children(tx, &id.to_string(), &at_s)?;
                }
                TrashEntityKind::Attachment => {
                    let changed = tx.execute(
                        Self::restore_sql(TrashEntityKind::Attachment),
                        params![id.to_string(), at_s],
                    )?;
                    require_changed(changed, "attachment", id)?;
                }
                TrashEntityKind::InkDocument => {
                    let changed = tx.execute(
                        Self::restore_sql(TrashEntityKind::InkDocument),
                        params![id.to_string(), at_s],
                    )?;
                    require_changed(changed, "ink_document", id)?;
                }
            }
            Ok(())
        })
    }

    fn permanently_delete_item(&self, kind: TrashEntityKind, id: Uuid) -> JabeXResult<()> {
        self.db.with_transaction(|tx| {
            match kind {
                TrashEntityKind::Deck => {
                    let card_ids: Vec<String> = {
                        let mut stmt = tx.prepare(trash::CARD_IDS_IN_DECK)?;
                        let rows = stmt
                            .query_map(params![id.to_string()], |row| row.get(0))?
                            .collect::<Result<Vec<_>, _>>()?;
                        rows
                    };
                    for card_id in &card_ids {
                        hard_delete_card_tree(tx, card_id)?;
                    }
                    let changed = tx.execute(trash::HARD_DELETE_DECK, params![id.to_string()])?;
                    require_changed(changed, "deck", id)?;
                }
                TrashEntityKind::Card => {
                    hard_delete_card_tree(tx, &id.to_string())?;
                }
                TrashEntityKind::Attachment => {
                    let changed =
                        tx.execute(trash::HARD_DELETE_ATTACHMENT, params![id.to_string()])?;
                    require_changed(changed, "attachment", id)?;
                }
                TrashEntityKind::InkDocument => {
                    let changed = tx.execute(trash::HARD_DELETE_INK, params![id.to_string()])?;
                    require_changed(changed, "ink_document", id)?;
                }
            }
            Ok(())
        })
    }
}

fn restore_card_children(conn: &Connection, card_id: &str, at_s: &str) -> Result<(), JabeXError> {
    conn.execute(trash::RESTORE_ATTACHMENTS_FOR_CARD, params![card_id, at_s])?;
    conn.execute(trash::RESTORE_INK_FOR_CARD, params![card_id, at_s])?;
    Ok(())
}

fn hard_delete_card_tree(conn: &Connection, card_id: &str) -> Result<(), JabeXError> {
    conn.execute(trash::HARD_DELETE_REVIEWS_FOR_CARD, params![card_id])?;
    conn.execute(trash::HARD_DELETE_ATTACHMENTS_FOR_OWNER, params![card_id])?;
    conn.execute(trash::HARD_DELETE_INK_FOR_OWNER, params![card_id])?;
    let changed = conn.execute(trash::HARD_DELETE_CARD, params![card_id])?;
    if changed == 0 {
        let id = Uuid::parse_str(card_id).unwrap_or(Uuid::nil());
        return Err(JabeXError::NotFound { entity: "card", id });
    }
    Ok(())
}
