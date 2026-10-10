pub const DECK_INSERT: &str = "INSERT INTO decks (
    id, name, description, settings, revision,
    created_at, updated_at, deleted_at, deleted_by
 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";

pub const DECK_UPDATE: &str = "UPDATE decks
 SET name = ?2, description = ?3, settings = ?4, revision = ?5,
     updated_at = ?6, deleted_at = ?7, deleted_by = ?8
 WHERE id = ?1";

pub const DECK_SOFT_DELETE: &str = "UPDATE decks
 SET deleted_at = ?2, deleted_by = ?3, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NULL";

pub const DECK_HARD_DELETE: &str = "DELETE FROM decks WHERE id = ?1";

pub const DECK_FIND_ACTIVE: &str = "SELECT id, name, description, settings, revision,
        created_at, updated_at, deleted_at, deleted_by
 FROM decks WHERE id = ?1 AND deleted_at IS NULL";

pub const DECK_FIND_ANY: &str = "SELECT id, name, description, settings, revision,
        created_at, updated_at, deleted_at, deleted_by
 FROM decks WHERE id = ?1";

pub const DECK_LIST_ACTIVE: &str = "SELECT id, name, description, settings, revision,
        created_at, updated_at, deleted_at, deleted_by
 FROM decks WHERE deleted_at IS NULL ORDER BY created_at ASC";

pub const DECK_LIST_ALL: &str = "SELECT id, name, description, settings, revision,
        created_at, updated_at, deleted_at, deleted_by
 FROM decks ORDER BY created_at ASC";
