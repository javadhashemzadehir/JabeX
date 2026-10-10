-- Rich content storage (ADR 010):
-- - owner-polymorphic attachments (metadata only; binaries on disk)
-- - ink_documents replaces drawings (JSON files under media/ink/)

-- Rebuild attachments with the production schema.
CREATE TABLE attachments_new (
    id TEXT PRIMARY KEY NOT NULL,
    owner_type TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    hash TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

INSERT INTO attachments_new (
    id, owner_type, owner_id, file_name, mime_type, size_bytes, hash,
    relative_path, revision, created_at, updated_at, deleted_at
)
SELECT
    id,
    'card',
    COALESCE(card_id, id),
    COALESCE(
        NULLIF(substr(path, instr(path, '/') + 1), ''),
        path
    ),
    COALESCE(mime_type, 'application/octet-stream'),
    size_bytes,
    sha256,
    path,
    COALESCE(revision, 1),
    created_at,
    updated_at,
    deleted_at
FROM attachments;

DROP TABLE attachments;
ALTER TABLE attachments_new RENAME TO attachments;

CREATE INDEX IF NOT EXISTS idx_attachments_owner
    ON attachments(owner_type, owner_id);
CREATE INDEX IF NOT EXISTS idx_attachments_deleted_at
    ON attachments(deleted_at);
CREATE INDEX IF NOT EXISTS idx_attachments_hash
    ON attachments(hash);

-- Ink document references (file payload at relative_path).
CREATE TABLE IF NOT EXISTS ink_documents (
    id TEXT PRIMARY KEY NOT NULL,
    owner_type TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    width INTEGER NOT NULL DEFAULT 800,
    height INTEGER NOT NULL DEFAULT 600,
    content_hash TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- Migrate legacy drawings metadata into ink_documents when present.
INSERT INTO ink_documents (
    id, owner_type, owner_id, relative_path, width, height, content_hash,
    revision, created_at, updated_at, deleted_at
)
SELECT
    id,
    'card',
    COALESCE(card_id, id),
    path,
    800,
    600,
    sha256,
    COALESCE(revision, 1),
    created_at,
    updated_at,
    deleted_at
FROM drawings;

DROP TABLE IF EXISTS drawings;

CREATE INDEX IF NOT EXISTS idx_ink_documents_owner
    ON ink_documents(owner_type, owner_id);
CREATE INDEX IF NOT EXISTS idx_ink_documents_deleted_at
    ON ink_documents(deleted_at);
