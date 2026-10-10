-- Add monotonic revision for future document-based sync (Git export/import).
-- Bumped on every entity write. Soft-delete tables already have deleted_at.

ALTER TABLE decks ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE cards ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE reviews ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE attachments ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE drawings ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE settings ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
