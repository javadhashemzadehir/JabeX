pub const CARD_INSERT: &str = "INSERT INTO cards (
    id, deck_id, card_type, front_content, back_content,
    box_position, due_at, state, review_count, lapse_count,
    difficulty, revision, created_at, updated_at, deleted_at,
    detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
    transliteration_json, language_assist_status, deleted_by,
    language_assist_overrides_json
 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
           ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)";

pub const CARD_UPDATE: &str = "UPDATE cards SET
    deck_id = ?2,
    card_type = ?3,
    front_content = ?4,
    back_content = ?5,
    box_position = ?6,
    due_at = ?7,
    state = ?8,
    review_count = ?9,
    lapse_count = ?10,
    difficulty = ?11,
    revision = ?12,
    updated_at = ?13,
    deleted_at = ?14,
    detected_language = ?15,
    translation = ?16,
    pronunciation_audio_path = ?17,
    pronunciation_ipa = ?18,
    transliteration_json = ?19,
    language_assist_status = ?20,
    deleted_by = ?21,
    language_assist_overrides_json = ?22
 WHERE id = ?1";

pub const CARD_SOFT_DELETE: &str = "UPDATE cards
 SET deleted_at = ?2, deleted_by = ?3, updated_at = ?2, revision = revision + 1
 WHERE id = ?1 AND deleted_at IS NULL";

pub const CARD_HARD_DELETE: &str = "DELETE FROM cards WHERE id = ?1";

pub const CARD_FIND_ACTIVE: &str = "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards WHERE id = ?1 AND deleted_at IS NULL";

pub const CARD_FIND_ANY: &str = "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards WHERE id = ?1";

pub const CARD_LIST_BY_DECK_ACTIVE: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deck_id = ?1 AND deleted_at IS NULL
 ORDER BY created_at ASC";

pub const CARD_LIST_BY_DECK_ACTIVE_PAGE: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deck_id = ?1 AND deleted_at IS NULL
 ORDER BY created_at ASC
 LIMIT ?2 OFFSET ?3";

pub const CARD_COUNT_BY_DECK_ACTIVE: &str =
    "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND deleted_at IS NULL";

pub const CARD_LIST_BY_DECK_ALL: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deck_id = ?1
 ORDER BY created_at ASC";

pub const CARD_LIST_ACTIVE: &str = "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards WHERE deleted_at IS NULL ORDER BY created_at ASC";

pub const CARD_LIST_ALL: &str = "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards ORDER BY created_at ASC";

pub const CARD_LIST_DUE_BY_DECK: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deck_id = ?1 AND deleted_at IS NULL AND due_at <= ?2
   AND state != 'suspended'
 ORDER BY due_at ASC";

pub const CARD_LIST_DUE_BY_DECK_LIMITED: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deck_id = ?1 AND deleted_at IS NULL AND due_at <= ?2
   AND state != 'suspended'
 ORDER BY due_at ASC
 LIMIT ?3";

pub const CARD_LIST_DUE_ALL: &str = "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deleted_at IS NULL AND due_at <= ?1
   AND state != 'suspended'
 ORDER BY due_at ASC";

pub const CARD_LIST_DUE_ALL_LIMITED: &str =
    "SELECT id, deck_id, card_type, front_content, back_content,
        box_position, due_at, state, review_count, lapse_count,
        difficulty, revision, created_at, updated_at, deleted_at,
        detected_language, translation, pronunciation_audio_path, pronunciation_ipa,
        transliteration_json, language_assist_status, deleted_by,
        language_assist_overrides_json
 FROM cards
 WHERE deleted_at IS NULL AND due_at <= ?1
   AND state != 'suspended'
 ORDER BY due_at ASC
 LIMIT ?2";
