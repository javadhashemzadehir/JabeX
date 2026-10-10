-- JabeX initial schema (001)
-- Soft-delete via deleted_at; reviews are append-only (no deleted_at).

CREATE TABLE IF NOT EXISTS decks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    settings TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY NOT NULL,
    deck_id TEXT NOT NULL,
    card_type TEXT NOT NULL,
    front_content TEXT NOT NULL,
    back_content TEXT NOT NULL,
    box_position INTEGER NOT NULL DEFAULT 1,
    due_at TEXT NOT NULL,
    state TEXT NOT NULL,
    review_count INTEGER NOT NULL DEFAULT 0,
    lapse_count INTEGER NOT NULL DEFAULT 0,
    difficulty REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (deck_id) REFERENCES decks(id)
);

CREATE INDEX IF NOT EXISTS idx_cards_deck_id ON cards(deck_id);
CREATE INDEX IF NOT EXISTS idx_cards_due_at ON cards(due_at);
CREATE INDEX IF NOT EXISTS idx_cards_deleted_at ON cards(deleted_at);

CREATE TABLE IF NOT EXISTS reviews (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    grade TEXT NOT NULL,
    reviewed_at TEXT NOT NULL,
    duration_ms INTEGER,
    box_before INTEGER NOT NULL,
    box_after INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE INDEX IF NOT EXISTS idx_reviews_card_id ON reviews(card_id);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewed_at ON reviews(reviewed_at);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT,
    kind TEXT NOT NULL,
    path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_attachments_sha256 ON attachments(sha256) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS drawings (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT,
    path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_drawings_sha256 ON drawings(sha256) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
