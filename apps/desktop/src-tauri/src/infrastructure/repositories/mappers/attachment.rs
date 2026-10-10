use crate::domain::attachment::{Attachment, AttachmentOwner};
use crate::infrastructure::repositories::helpers::{
    get_optional_rfc3339, get_optional_uuid, get_rfc3339, get_u32, get_u64_from_i64, get_uuid,
};

pub fn map_attachment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Attachment> {
    let owner_raw: String = row.get("owner_type")?;
    let owner_type = AttachmentOwner::parse(&owner_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;
    Ok(Attachment {
        id: get_uuid(row, "id")?,
        owner_type,
        owner_id: get_uuid(row, "owner_id")?,
        file_name: row.get("file_name")?,
        mime_type: row.get("mime_type")?,
        size_bytes: get_u64_from_i64(row, "size_bytes")?,
        hash: row.get("hash")?,
        relative_path: row.get("relative_path")?,
        revision: get_u32(row, "revision")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
        deleted_at: get_optional_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
    })
}
