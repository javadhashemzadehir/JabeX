pub const REVIEW_INSERT: &str = "INSERT INTO reviews (
    id, card_id, grade, reviewed_at, duration_ms,
    box_before, box_after, revision, created_at, updated_at
 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

pub const REVIEW_LIST_BY_CARD: &str = "SELECT id, card_id, grade, reviewed_at, duration_ms,
        box_before, box_after, revision, created_at, updated_at
 FROM reviews
 WHERE card_id = ?1
 ORDER BY reviewed_at ASC";

pub const REVIEW_LIST_ALL: &str = "SELECT id, card_id, grade, reviewed_at, duration_ms,
        box_before, box_after, revision, created_at, updated_at
 FROM reviews
 ORDER BY reviewed_at ASC";

pub const REVIEW_HARD_DELETE_BY_CARD: &str = "DELETE FROM reviews WHERE card_id = ?1";
