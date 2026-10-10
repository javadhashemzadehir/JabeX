pub const LIST_DELETED_DECKS: &str = "SELECT id, name, deleted_at, deleted_by FROM decks
 WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC";

pub const LIST_DELETED_CARDS: &str =
    "SELECT id, deck_id, front_content, deleted_at, deleted_by FROM cards
 WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC";

pub const LIST_DELETED_ATTACHMENTS: &str =
    "SELECT id, owner_id, file_name, deleted_at, deleted_by FROM attachments
 WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC";

pub const LIST_DELETED_INK: &str =
    "SELECT id, owner_id, relative_path, deleted_at, deleted_by FROM ink_documents
 WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC";

pub const RESTORE_DECK: &str = "UPDATE decks
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NOT NULL";

pub const RESTORE_CARD: &str = "UPDATE cards
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NOT NULL";

pub const RESTORE_ATTACHMENT: &str = "UPDATE attachments
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NOT NULL";

pub const RESTORE_INK: &str = "UPDATE ink_documents
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NOT NULL";

pub const CARD_IDS_DELETED_IN_DECK: &str =
    "SELECT id FROM cards WHERE deck_id = ?1 AND deleted_at IS NOT NULL";

pub const CARD_IDS_IN_DECK: &str = "SELECT id FROM cards WHERE deck_id = ?1";

pub const CARD_DECK_ID: &str = "SELECT deck_id FROM cards WHERE id = ?1";

pub const RESTORE_ATTACHMENTS_FOR_CARD: &str = "UPDATE attachments
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE owner_id = ?1 AND deleted_at IS NOT NULL";

pub const RESTORE_INK_FOR_CARD: &str = "UPDATE ink_documents
 SET deleted_at = NULL, deleted_by = NULL, updated_at = ?2, revision = revision + 1
 WHERE owner_id = ?1 AND deleted_at IS NOT NULL";

pub const HARD_DELETE_REVIEWS_FOR_CARD: &str = "DELETE FROM reviews WHERE card_id = ?1";
pub const HARD_DELETE_ATTACHMENTS_FOR_OWNER: &str = "DELETE FROM attachments WHERE owner_id = ?1";
pub const HARD_DELETE_INK_FOR_OWNER: &str = "DELETE FROM ink_documents WHERE owner_id = ?1";
pub const HARD_DELETE_CARD: &str = "DELETE FROM cards WHERE id = ?1";
pub const HARD_DELETE_DECK: &str = "DELETE FROM decks WHERE id = ?1";
pub const HARD_DELETE_ATTACHMENT: &str = "DELETE FROM attachments WHERE id = ?1";
pub const HARD_DELETE_INK: &str = "DELETE FROM ink_documents WHERE id = ?1";
