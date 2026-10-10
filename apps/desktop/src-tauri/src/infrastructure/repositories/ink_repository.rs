use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::ink::{InkDocumentRef, InkOwner};
use crate::domain::repositories::InkDocumentRepository;
use crate::infrastructure::database::Database;

use super::helpers::require_changed;
use super::mappers::map_ink_ref;
use super::queries::ink;

pub struct SqliteInkDocumentRepository {
    db: Database,
}

impl SqliteInkDocumentRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, doc: &InkDocumentRef) -> JabeXResult<()> {
        conn.execute(
            ink::INK_INSERT,
            params![
                doc.id.to_string(),
                doc.owner_type.as_str(),
                doc.owner_id.to_string(),
                doc.relative_path,
                doc.width,
                doc.height,
                doc.content_hash,
                doc.revision,
                doc.created_at.to_rfc3339(),
                doc.updated_at.to_rfc3339(),
                doc.deleted_at.map(|t| t.to_rfc3339()),
                doc.deleted_by.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }
}

impl InkDocumentRepository for SqliteInkDocumentRepository {
    fn create(&self, doc: &InkDocumentRef) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, doc))
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<InkDocumentRef>> {
        self.db.with_conn(|conn| {
            conn.query_row(ink::INK_FIND, params![id.to_string()], map_ink_ref)
                .optional()
                .map_err(JabeXError::from)
        })
    }

    fn list_by_owner(
        &self,
        owner_type: InkOwner,
        owner_id: Uuid,
        include_deleted: bool,
    ) -> JabeXResult<Vec<InkDocumentRef>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                ink::INK_LIST_BY_OWNER_ALL
            } else {
                ink::INK_LIST_BY_OWNER_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map(params![owner_type.as_str(), owner_id.to_string()], map_ink_ref)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<InkDocumentRef>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                ink::INK_LIST_ALL
            } else {
                ink::INK_LIST_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map([], map_ink_ref)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn update(&self, doc: &InkDocumentRef) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                ink::INK_UPDATE,
                params![
                    doc.id.to_string(),
                    doc.owner_type.as_str(),
                    doc.owner_id.to_string(),
                    doc.relative_path,
                    doc.width,
                    doc.height,
                    doc.content_hash,
                    doc.revision,
                    doc.updated_at.to_rfc3339(),
                    doc.deleted_at.map(|t| t.to_rfc3339()),
                    doc.deleted_by.map(|id| id.to_string()),
                ],
            )?;
            require_changed(changed, "ink_document", doc.id)
        })
    }

    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()> {
        let mut doc = self.find(id)?.ok_or(JabeXError::NotFound {
            entity: "ink_document",
            id,
        })?;
        if doc.is_deleted() {
            return Ok(());
        }
        doc.soft_delete(deleted_at, deleted_by);
        self.update(&doc)
    }

    fn hard_delete(&self, id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(ink::INK_HARD_DELETE, params![id.to_string()])?;
            require_changed(changed, "ink_document", id)
        })
    }
}
