pub const INK_INSERT: &str = "INSERT INTO ink_documents (
    id, owner_type, owner_id, relative_path, width, height, content_hash,
    revision, created_at, updated_at, deleted_at, deleted_by
 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

pub const INK_UPDATE: &str = "UPDATE ink_documents SET
    owner_type = ?2, owner_id = ?3, relative_path = ?4, width = ?5, height = ?6,
    content_hash = ?7, revision = ?8, updated_at = ?9, deleted_at = ?10,
    deleted_by = ?11
 WHERE id = ?1";

pub const INK_HARD_DELETE: &str = "DELETE FROM ink_documents WHERE id = ?1";

pub const INK_FIND: &str = "SELECT id, owner_type, owner_id, relative_path, width, height, content_hash,
        revision, created_at, updated_at, deleted_at, deleted_by
 FROM ink_documents WHERE id = ?1";

pub const INK_LIST_BY_OWNER_ACTIVE: &str = "SELECT id, owner_type, owner_id, relative_path, width, height, content_hash,
        revision, created_at, updated_at, deleted_at, deleted_by
 FROM ink_documents
 WHERE owner_type = ?1 AND owner_id = ?2 AND deleted_at IS NULL
 ORDER BY created_at ASC";

pub const INK_LIST_BY_OWNER_ALL: &str = "SELECT id, owner_type, owner_id, relative_path, width, height, content_hash,
        revision, created_at, updated_at, deleted_at, deleted_by
 FROM ink_documents
 WHERE owner_type = ?1 AND owner_id = ?2
 ORDER BY created_at ASC";

pub const INK_LIST_ACTIVE: &str = "SELECT id, owner_type, owner_id, relative_path, width, height, content_hash,
        revision, created_at, updated_at, deleted_at, deleted_by
 FROM ink_documents WHERE deleted_at IS NULL ORDER BY created_at ASC";

pub const INK_LIST_ALL: &str = "SELECT id, owner_type, owner_id, relative_path, width, height, content_hash,
        revision, created_at, updated_at, deleted_at, deleted_by
 FROM ink_documents ORDER BY created_at ASC";
