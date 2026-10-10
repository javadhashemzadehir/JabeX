-- Composite indexes for large-deck list and due-queue scans.
-- Soft-deleted rows are filtered with deleted_at IS NULL in hot paths.

CREATE INDEX IF NOT EXISTS idx_cards_deck_active_created
  ON cards(deck_id, created_at)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_cards_due_active
  ON cards(due_at, deck_id)
  WHERE deleted_at IS NULL AND state != 'suspended';
