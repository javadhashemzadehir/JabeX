use crate::domain::ink::{InkDocumentRef, InkOwner};
use crate::infrastructure::repositories::helpers::{
    get_optional_rfc3339, get_optional_uuid, get_rfc3339, get_u32, get_uuid,
};

pub fn map_ink_ref(row: &rusqlite::Row<'_>) -> rusqlite::Result<InkDocumentRef> {
    let owner_raw: String = row.get("owner_type")?;
    let owner_type = InkOwner::parse(&owner_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;
    Ok(InkDocumentRef {
        id: get_uuid(row, "id")?,
        owner_type,
        owner_id: get_uuid(row, "owner_id")?,
        relative_path: row.get("relative_path")?,
        width: get_u32(row, "width")?,
        height: get_u32(row, "height")?,
        content_hash: row.get("content_hash")?,
        revision: get_u32(row, "revision")?,
        created_at: get_rfc3339(row, "created_at")?,
        updated_at: get_rfc3339(row, "updated_at")?,
        deleted_at: get_optional_rfc3339(row, "deleted_at")?,
        deleted_by: get_optional_uuid(row, "deleted_by")?,
    })
}
