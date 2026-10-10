# ADR 015: Global Search with SQLite FTS5

## Status

Accepted

## Context

JabeX needs a fast, offline, profile-scoped way to find cards, decks, tags, attachments, settings, and navigation commands from the app header (command palette / Ctrl·Cmd+K). Search must work for Persian and English (including mixed RTL/LTR), TipTap JSON faces, Markdown/HTML legacy text, and LaTeX formulas — without cloud services or telemetry.

Constraints:

- Local-first per-profile SQLite (ADR 003 / 006 / 008)
- Clean Architecture: domain extraction/ranking must not depend on rusqlite (ADR 007)
- Soft-delete aware: trashed entities leave the index
- Extensible entity kinds (future notes/tags tables should register without UI rewrites)

## Decision

### Unified search entity model

All hits share `SearchEntityType`: `card`, `deck`, `tag`, `review`, `note`, `attachment`, `setting`, `command`. Index documents are denormalized rows with `title`, `content`, `normalized_content`, and `metadata_json` (breadcrumb, href, due flag, etc.).

### SQLite FTS5 index

Migration **V9__global_search.sql** adds:

| Table | Role |
|-------|------|
| `search_index` | Content table (UUID id, profile_id, entity_type/id, text fields, metadata) |
| `search_index_fts` | FTS5 external-content virtual table (`unicode61` tokenizer) kept in sync via triggers |
| `recent_searches` | Last N queries per profile (UI chips) |
| `search_history` | Local selection log for future ranking (never uploaded) |

Incremental upserts run after card/deck/attachment mutations and trash restore; soft-delete removes index rows. Empty index triggers a full rebuild on first search.

### Text extraction pipeline

Domain `extract_search_text()` accepts plain text, Markdown, HTML, TipTap/Slate-like JSON, and LaTeX (`$...$` / math nodes). Index stores both display text and normalized forms (Arabic/Persian letter folding, latex `mc^2` → `mc2`).

### Ranking & fuzzy matching

FTS5 prefix MATCH is the primary candidate source. Short-prefix LIKE fallback + edit-distance variants feed typo tolerance (`javscript` → `javascript`, `فلسف` → `فلسفه`). Domain ranking prefers exact title → exact front → tag → prefix → fuzzy → content.

### UI

Header hosts a cmdk/shadcn Command palette (`components/search/*`) with filters, per-item removable recent searches, popular queries (aggregated from local `search_history`), highlight rendering, and keyboard open (default Ctrl/Cmd+K; remappable in Settings).

## Consequences

### Positive

- Instant local search without network
- Extensible entity registration via index documents
- Persian/English/RTL first-class
- Soft-delete and profile isolation preserved

### Trade-offs

- Denormalized index can drift if the DB is edited outside the app (rebuild command / empty-index rebuild recover)
- Hashtag tags are derived until a first-class Tag table exists
- Fuzzy search uses candidate expansion, not a dedicated trigram index (adequate for local deck sizes; can add later)

## Alternatives considered

| Alternative | Why not |
|-------------|---------|
| Frontend-only filter of loaded cards | Does not scale to 100k cards; misses decks/commands |
| External search engine | Violates offline/local-first |
| Store HTML and grep | Unsafe / slow; TipTap JSON is the source of truth |
