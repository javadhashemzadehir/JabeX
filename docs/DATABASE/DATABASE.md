# Database & Local Storage

## Principles

- **Local-first**: all decks, cards, and review history stay on device
- **User-owned**: export/import is explicit and user-driven (`.jbx`)
- **No cloud DB** for product data
- **Profile isolation**: each local profile has its own SQLite + media tree

## Layout

```
{app_data}/JabeX/
├── app.sqlite                 # local profile registry
├── profiles/{profile_uuid}/
│   ├── database.sqlite        # study data
│   └── media/
│       ├── attachments/{uuid}/
│       │   ├── original.{ext}
│       │   ├── metadata.json
│       │   └── thumbnails/
│       ├── ink/{uuid}.json
│       └── audio/pronunciation/{card_uuid}.mp3   # optional language-assist cache
└── backups/*.jbx
```

Legacy folders (`media/images`, `media/drawings`) may exist on upgraded installs; new attachment writes use `attachments/` and `ink/`. Pronunciation TTS from Smart Language Assist uses `media/audio/pronunciation/` (ADR 011).

Legacy single-file installs (`database.sqlite` at the app root) are migrated into a Default profile on first launch.

## Engines

| Database | Role |
|----------|------|
| `app.sqlite` | Profiles + active profile pointer |
| `profiles/*/database.sqlite` | Decks, cards, reviews, settings, attachment metadata |

Study DBs use **rusqlite** (bundled) + **refinery** migrations:

- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- Soft delete via `deleted_at` + `deleted_by` (local profile UUID; except append-only `reviews`)
- Normal queries exclude soft-deleted rows; Trash / backup may include them
- Every study entity has a monotonic `revision` for future document sync

## Persistence layer

No ORM. Infrastructure repositories stay thin:

| Piece | Location | Role |
|-------|----------|------|
| Query modules | `src-tauri/.../repositories/queries/` | SQL constants (SELECT/INSERT/UPDATE/DELETE) |
| Named mappers | `src-tauri/.../repositories/mappers/` | `Row` → domain entities by column name |
| Helpers | `src-tauri/.../repositories/helpers/` | UUID/time/enum/JSON + `require_changed` |
| Transactions | `Database::with_transaction` | Multi-step writes (trash hard-delete, `.jbx` entity import) |

Migrations remain versioned SQL under `infrastructure/database/migrations/` (`V{n}__name.sql`), embedded by refinery at build time.

## Study tables

| Table | Notes |
|-------|-------|
| `decks` | Soft-delete (`deleted_at` / `deleted_by`); `revision` |
| `cards` | Soft-delete; Leitner `box_position`, `due_at`, `revision`; faces are versioned TipTap JSON in `front_content` / `back_content` TEXT; optional language-assist columns (`detected_language`, `translation`, `pronunciation_audio_path`, `transliteration_json`, `language_assist_status`, `language_assist_overrides_json`) |
| `reviews` | Append-only; `revision` (always 1 on insert) |
| `study_activity` | Daily projection of reviews for streaks/heatmap (ADR 013); unique `(profile_id, activity_date)`; not a habit store |
| `attachments` | Owner-polymorphic metadata only; files under `media/attachments/`; `revision`; soft-delete + library purge (hard delete) via media library |
| `ink_documents` | Ink JSON references under `media/ink/`; `content_hash`; `revision` (replaces V1 `drawings`) |
| `settings` | Key/value JSON; soft-delete; `revision` |
| `search_index` + `search_index_fts` | Denormalized global search documents + FTS5 (ADR 015); soft-delete aware; profile-scoped |
| `recent_searches` | Last queries per profile for the command palette (capped; deletable per row) |
| `search_history` | Local selection history; also powers popular-query aggregation — never leaves the device |

Every table includes `created_at` / `updated_at` (ISO-8601 UTC), except `study_activity` which has `created_at` only (daily aggregate rows are upserted in place).

### Study activity

| Column | Notes |
|--------|-------|
| `id` | UUIDv7 |
| `profile_id` | Owning local profile (DB is already profile-scoped; stored for backup clarity) |
| `activity_date` | Local calendar day `YYYY-MM-DD` |
| `cards_reviewed` | Cards graded that day |
| `minutes_spent` | Accumulated review duration in **seconds** (IPC exposes whole minutes) |
| `created_at` | First insert time for that day |

Streaks and period totals are **calculated** from these rows — never stored as separate counters. Empty tables are backfilled once from `reviews` when consistency stats are first requested. See [ADR 013](../ADR/013-study-activity-consistency/013-study-activity-consistency.md).

## Profile registry (`app.sqlite`)

| Table | Notes |
|-------|-------|
| `profiles` | `id`, `display_name`, `avatar`, timestamps, soft-delete |
| `app_meta` | Key/value (`active_profile_id`, `app_preferences` JSON for tray/window, `initial_profile_setup_pending`) |

`app_preferences` is device-level (not copied into `.jbx` study backups as profile settings).

## Card lifecycle (Leitner)

1. New card starts in box 1
2. Correct review promotes box (+1, max 5)
3. Incorrect review resets to box 1
4. Due set = `due_at <= now` and not suspended/deleted

Default (recommended) intervals: 1 / 3 / 7 / 14 / 30 days.

Profile default algorithm is stored in `settings` under key `leitner.algorithm`.
Each deck may inherit that default or override it via `decks.settings.algorithm`
(JSON: preset package or custom day intervals). Effective intervals are resolved
at review time.

Optional Smart Language Assist preferences live under `settings` key
`language_assist` (JSON: `autoTranslation`, `pronunciationGeneration`,
`provider`, `targetLanguage`). Defaults keep networking off.

Per-deck overrides: `decks.settings.languageAssist` (`translation` /
`pronunciation` ∈ `inherit` | `enabled` | `disabled`).
Per-card overrides: `cards.language_assist_overrides_json` (same shape).
Effective feature = profile master ∧ deck/card resolution (card wins).

## Backup (`.jbx`)

Versioned ZIP of normalized JSON documents + optional media. See [ADR 008](../ADR/008-local-profiles-and-jbx-backup/008-local-profiles-and-jbx-backup.md).

## Card face documents

`front_content` / `back_content` store JSON (not free HTML):

```json
{ "version": 2, "type": "doc", "content": [/* TipTap nodes */] }
```

- Current write schema version: **2** (`RICH_TEXT_DOCUMENT_VERSION`) — spoilers, math, tables, highlighted code ([ADR 014](../ADR/014-advanced-card-editor/014-advanced-card-editor.md))
- Readers accept `version` 1–2; legacy plain-text cells are accepted on read and rewritten as v2 on the next update
- No separate SQL `content_version` columns (version lives inside the JSON)
- See [ADR 009](../ADR/009-tiptap-rich-text-documents/009-tiptap-rich-text-documents.md)

## Migration Policy

- Versioned migrations only (`V{n}__name.sql`) per profile database
- Document each schema change here
- Keep TypeScript types in `packages/types` aligned with Rust models

| Version | Notes |
|---------|-------|
| V1 | Initial schema (`deleted_at` on deletable tables) |
| V2 | Entity `revision` columns |
| V3 | Document contract for TipTap JSON faces (columns unchanged) |
| V4 | Rich content: rebuild `attachments` (owner_type/owner_id/…); add `ink_documents`; drop `drawings` |
| V5 | Optional Smart Language Assist columns on `cards` |
| V6 | Soft-delete metadata: `deleted_by` on decks/cards/attachments/ink_documents/settings + trash indexes (ADR 012) |
| V7 | Per-card language-assist overrides (`cards.language_assist_overrides_json`); deck overrides remain in `decks.settings` JSON |
| V8 | `study_activity` daily aggregates for learning consistency (ADR 013) |
| V9 | Global search: `search_index` + FTS5, `recent_searches`, `search_history` (ADR 015) |
| V10 | Partial indexes for active card lists / due queues (`idx_cards_deck_active_created`, `idx_cards_due_active`) |

## Privacy Notes

- Do not log card contents
- Database files belong under the app data directory, not the repo
- Backups are local `.jbx` archives (user-initiated)
