-- Optional Smart Language Assistant metadata on cards (ADR 011).
-- All columns nullable / defaulted so offline study is unchanged.

ALTER TABLE cards ADD COLUMN detected_language TEXT;
ALTER TABLE cards ADD COLUMN translation TEXT;
ALTER TABLE cards ADD COLUMN pronunciation_audio_path TEXT;
ALTER TABLE cards ADD COLUMN transliteration_json TEXT;
ALTER TABLE cards ADD COLUMN language_assist_status TEXT NOT NULL DEFAULT 'idle';
