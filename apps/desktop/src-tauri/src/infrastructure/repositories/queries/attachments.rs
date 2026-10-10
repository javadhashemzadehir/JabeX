pub const ATTACHMENT_INSERT: &str = "INSERT INTO attachments (
    id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
    relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

pub const ATTACHMENT_UPDATE: &str = "UPDATE attachments SET
    owner_type = ?2, owner_id = ?3, file_name = ?4, mime_type = ?5,
    size_bytes = ?6, hash = ?7, relative_path = ?8, revision = ?9,
    updated_at = ?10, deleted_at = ?11, deleted_by = ?12
 WHERE id = ?1";

pub const ATTACHMENT_HARD_DELETE: &str = "DELETE FROM attachments WHERE id = ?1";

pub const ATTACHMENT_FIND: &str =
    "SELECT id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
        relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 FROM attachments WHERE id = ?1";

pub const ATTACHMENT_LIST_BY_OWNER_ACTIVE: &str =
    "SELECT id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
        relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 FROM attachments
 WHERE owner_type = ?1 AND owner_id = ?2 AND deleted_at IS NULL
 ORDER BY created_at ASC";

pub const ATTACHMENT_LIST_BY_OWNER_ALL: &str =
    "SELECT id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
        relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 FROM attachments
 WHERE owner_type = ?1 AND owner_id = ?2
 ORDER BY created_at ASC";

pub const ATTACHMENT_LIST_ACTIVE: &str =
    "SELECT id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
        relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 FROM attachments WHERE deleted_at IS NULL ORDER BY created_at ASC";

pub const ATTACHMENT_LIST_ALL: &str =
    "SELECT id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
        relative_path, revision, created_at, updated_at, deleted_at, deleted_by
 FROM attachments ORDER BY created_at ASC";
