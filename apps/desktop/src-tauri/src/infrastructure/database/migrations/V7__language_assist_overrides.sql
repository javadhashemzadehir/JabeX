-- Per-card language-assist overrides (inherit | enabled | disabled).
-- Deck overrides live in decks.settings JSON (no schema change).

ALTER TABLE cards ADD COLUMN language_assist_overrides_json TEXT;
