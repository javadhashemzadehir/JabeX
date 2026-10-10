-- Global search index (FTS5) + recent searches + local search history (ADR 015).
-- Profile DBs are already isolated; profile_id is stored for backup clarity.

CREATE TABLE IF NOT EXISTS search_index (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    normalized_content TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    language TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (entity_type, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_search_index_profile ON search_index(profile_id);
CREATE INDEX IF NOT EXISTS idx_search_index_entity_type ON search_index(entity_type);
CREATE INDEX IF NOT EXISTS idx_search_index_entity ON search_index(entity_type, entity_id);

CREATE VIRTUAL TABLE IF NOT EXISTS search_index_fts USING fts5(
    title,
    content,
    normalized_content,
    content='search_index',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS search_index_ai AFTER INSERT ON search_index BEGIN
    INSERT INTO search_index_fts(rowid, title, content, normalized_content)
    VALUES (new.rowid, new.title, new.content, new.normalized_content);
END;

CREATE TRIGGER IF NOT EXISTS search_index_ad AFTER DELETE ON search_index BEGIN
    INSERT INTO search_index_fts(search_index_fts, rowid, title, content, normalized_content)
    VALUES ('delete', old.rowid, old.title, old.content, old.normalized_content);
END;

CREATE TRIGGER IF NOT EXISTS search_index_au AFTER UPDATE ON search_index BEGIN
    INSERT INTO search_index_fts(search_index_fts, rowid, title, content, normalized_content)
    VALUES ('delete', old.rowid, old.title, old.content, old.normalized_content);
    INSERT INTO search_index_fts(rowid, title, content, normalized_content)
    VALUES (new.rowid, new.title, new.content, new.normalized_content);
END;

CREATE TABLE IF NOT EXISTS recent_searches (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    query TEXT NOT NULL,
    filters TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recent_searches_profile_created
    ON recent_searches(profile_id, created_at DESC);

CREATE TABLE IF NOT EXISTS search_history (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    query TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_search_history_profile_created
    ON search_history(profile_id, created_at DESC);
