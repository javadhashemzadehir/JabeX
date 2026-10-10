use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::attachment::{Attachment, AttachmentOwner};
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::repositories::AttachmentRepository;
use crate::infrastructure::database::Database;

use super::helpers::require_changed;
use super::mappers::map_attachment;
use super::queries::attachments;

pub struct SqliteAttachmentRepository {
    db: Database,
}

impl SqliteAttachmentRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub(crate) fn create_on(conn: &Connection, attachment: &Attachment) -> JabeXResult<()> {
        conn.execute(
            attachments::ATTACHMENT_INSERT,
            params![
                attachment.id.to_string(),
                attachment.owner_type.as_str(),
                attachment.owner_id.to_string(),
                attachment.file_name,
                attachment.mime_type,
                attachment.size_bytes as i64,
                attachment.hash,
                attachment.relative_path,
                attachment.revision,
                attachment.created_at.to_rfc3339(),
                attachment.updated_at.to_rfc3339(),
                attachment.deleted_at.map(|t| t.to_rfc3339()),
                attachment.deleted_by.map(|id| id.to_string()),
            ],
        )?;
        Ok(())
    }
}

impl AttachmentRepository for SqliteAttachmentRepository {
    fn create(&self, attachment: &Attachment) -> JabeXResult<()> {
        self.db.with_conn(|conn| Self::create_on(conn, attachment))
    }

    fn find(&self, id: Uuid) -> JabeXResult<Option<Attachment>> {
        self.db.with_conn(|conn| {
            conn.query_row(
                attachments::ATTACHMENT_FIND,
                params![id.to_string()],
                map_attachment,
            )
            .optional()
            .map_err(JabeXError::from)
        })
    }

    fn list_by_owner(
        &self,
        owner_type: AttachmentOwner,
        owner_id: Uuid,
        include_deleted: bool,
    ) -> JabeXResult<Vec<Attachment>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                attachments::ATTACHMENT_LIST_BY_OWNER_ALL
            } else {
                attachments::ATTACHMENT_LIST_BY_OWNER_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map(
                    params![owner_type.as_str(), owner_id.to_string()],
                    map_attachment,
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn list_all(&self, include_deleted: bool) -> JabeXResult<Vec<Attachment>> {
        self.db.with_conn(|conn| {
            let sql = if include_deleted {
                attachments::ATTACHMENT_LIST_ALL
            } else {
                attachments::ATTACHMENT_LIST_ACTIVE
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt
                .query_map([], map_attachment)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    fn update(&self, attachment: &Attachment) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                attachments::ATTACHMENT_UPDATE,
                params![
                    attachment.id.to_string(),
                    attachment.owner_type.as_str(),
                    attachment.owner_id.to_string(),
                    attachment.file_name,
                    attachment.mime_type,
                    attachment.size_bytes as i64,
                    attachment.hash,
                    attachment.relative_path,
                    attachment.revision,
                    attachment.updated_at.to_rfc3339(),
                    attachment.deleted_at.map(|t| t.to_rfc3339()),
                    attachment.deleted_by.map(|id| id.to_string()),
                ],
            )?;
            require_changed(changed, "attachment", attachment.id)
        })
    }

    fn soft_delete(
        &self,
        id: Uuid,
        deleted_at: DateTime<Utc>,
        deleted_by: Option<Uuid>,
    ) -> JabeXResult<()> {
        let mut attachment = self.find(id)?.ok_or(JabeXError::NotFound {
            entity: "attachment",
            id,
        })?;
        if attachment.is_deleted() {
            return Ok(());
        }
        attachment.soft_delete(deleted_at, deleted_by);
        self.update(&attachment)
    }

    fn hard_delete(&self, id: Uuid) -> JabeXResult<()> {
        self.db.with_conn(|conn| {
            let changed =
                conn.execute(attachments::ATTACHMENT_HARD_DELETE, params![id.to_string()])?;
            require_changed(changed, "attachment", id)
        })
    }
}
