-- Offline speech & translation cache (ADR 016).
-- Additive only; existing language-assist columns remain the card-level summary.
-- `cards.pronunciation_ipa` is ensured idempotently in Rust after migrations
-- (so re-running this version in tests cannot fail on duplicate columns).

CREATE TABLE IF NOT EXISTS card_audio (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    provider TEXT NOT NULL,
    voice TEXT,
    file_path TEXT NOT NULL,
    duration_ms INTEGER,
    hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_card_audio_hash ON card_audio(hash);
CREATE INDEX IF NOT EXISTS idx_card_audio_card_id ON card_audio(card_id);

CREATE TABLE IF NOT EXISTS language_preferences (
    id TEXT PRIMARY KEY NOT NULL,
    language TEXT NOT NULL UNIQUE,
    preferred_voice TEXT,
    speed REAL NOT NULL DEFAULT 1.0,
    provider TEXT NOT NULL DEFAULT 'auto',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS translation_cache (
    id TEXT PRIMARY KEY NOT NULL,
    source_text TEXT NOT NULL,
    source_language TEXT NOT NULL,
    target_language TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_translation_cache_lookup
    ON translation_cache(source_language, target_language, source_text);
