pub const TRANSLATION_CACHE_UPSERT: &str = "INSERT INTO translation_cache (
    id, source_text, source_language, target_language, translated_text, created_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(source_language, target_language, source_text) DO UPDATE SET
    translated_text = excluded.translated_text,
    id = excluded.id,
    created_at = excluded.created_at";

pub const TRANSLATION_CACHE_FIND: &str = "SELECT id, source_text, source_language, target_language,
        translated_text, created_at
 FROM translation_cache
 WHERE source_language = ?1 AND target_language = ?2 AND source_text = ?3
 LIMIT 1";

pub const CARD_AUDIO_UPSERT: &str = "INSERT INTO card_audio (
    id, card_id, language, provider, voice, file_path, duration_ms, hash,
    created_at, updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
ON CONFLICT(hash) DO UPDATE SET
    card_id = excluded.card_id,
    language = excluded.language,
    provider = excluded.provider,
    voice = excluded.voice,
    file_path = excluded.file_path,
    duration_ms = excluded.duration_ms,
    updated_at = excluded.updated_at";

pub const CARD_AUDIO_FIND_BY_HASH: &str = "SELECT id, card_id, language, provider, voice, file_path,
        duration_ms, hash, created_at, updated_at
 FROM card_audio WHERE hash = ?1 LIMIT 1";

pub const CARD_AUDIO_FIND_BY_CARD: &str = "SELECT id, card_id, language, provider, voice, file_path,
        duration_ms, hash, created_at, updated_at
 FROM card_audio WHERE card_id = ?1 ORDER BY created_at DESC";

pub const LANGUAGE_PREF_UPSERT: &str = "INSERT INTO language_preferences (
    id, language, preferred_voice, speed, provider, created_at, updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT(language) DO UPDATE SET
    preferred_voice = excluded.preferred_voice,
    speed = excluded.speed,
    provider = excluded.provider,
    updated_at = excluded.updated_at";

pub const LANGUAGE_PREF_FIND: &str = "SELECT id, language, preferred_voice, speed, provider,
        created_at, updated_at
 FROM language_preferences WHERE language = ?1 LIMIT 1";

pub const LANGUAGE_PREF_LIST: &str = "SELECT id, language, preferred_voice, speed, provider,
        created_at, updated_at
 FROM language_preferences ORDER BY language ASC";
