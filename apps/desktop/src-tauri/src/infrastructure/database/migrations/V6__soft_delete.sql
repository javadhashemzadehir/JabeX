-- Soft-delete metadata for Trash + future Git-based sync.
-- deleted_at has existed since V1; this adds deleted_by (local profile UUID)
-- and ensures indexes for trash listing. Normal user actions never hard-delete.

ALTER TABLE decks ADD COLUMN deleted_by TEXT;
ALTER TABLE cards ADD COLUMN deleted_by TEXT;
ALTER TABLE attachments ADD COLUMN deleted_by TEXT;
ALTER TABLE ink_documents ADD COLUMN deleted_by TEXT;
ALTER TABLE settings ADD COLUMN deleted_by TEXT;

CREATE INDEX IF NOT EXISTS idx_decks_deleted_at ON decks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_settings_deleted_at ON settings(deleted_at);
