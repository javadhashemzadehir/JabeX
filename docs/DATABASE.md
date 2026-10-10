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
│       ├── images/
│       ├── audio/
│       └── drawings/
└── backups/*.jbx
```

Legacy single-file installs (`database.sqlite` at the app root) are migrated into a Default profile on first launch.

## Engines

| Database | Role |
|----------|------|
| `app.sqlite` | Profiles + active profile pointer |
| `profiles/*/database.sqlite` | Decks, cards, reviews, settings, attachment metadata |

Study DBs use **rusqlite** (bundled) + **refinery** migrations:

- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- Soft delete via `deleted_at` (except append-only `reviews`)
- Every study entity has a monotonic `revision` for future document sync

## Study tables

| Table | Notes |
|-------|-------|
| `decks` | Soft-delete; `revision` |
| `cards` | Soft-delete; Leitner `box_position`, `due_at`, `revision` |
| `reviews` | Append-only; `revision` (always 1 on insert) |
| `attachments` | Metadata only; files under `media/`; `revision` |
| `drawings` | Metadata only; files under `media/drawings/`; `revision` |
| `settings` | Key/value JSON; soft-delete; `revision` |

Every table includes `created_at` / `updated_at` (ISO-8601 UTC).

## Profile registry (`app.sqlite`)

| Table | Notes |
|-------|-------|
| `profiles` | `id`, `display_name`, `avatar`, timestamps, soft-delete |
| `app_meta` | Key/value (`active_profile_id`, `app_preferences` JSON for tray/window) |

`app_preferences` is device-level (not copied into `.jbx` study backups as profile settings).

## Card lifecycle (Leitner)

1. New card starts in box 1
2. Correct review promotes box (+1, max 5)
3. Incorrect review resets to box 1
4. Due set = `due_at <= now` and not suspended/deleted

Intervals: 1 / 3 / 7 / 14 / 30 days.

## Backup (`.jbx`)

Versioned ZIP of normalized JSON documents + optional media. See [ADR 008](./ADR/008-local-profiles-and-jbx-backup.md).

## Migration Policy

- Versioned migrations only (`V{n}__name.sql`) per profile database
- Document each schema change here
- Keep TypeScript types in `packages/types` aligned with Rust models

## Privacy Notes

- Do not log card contents
- Database files belong under the app data directory, not the repo
- Backups are local `.jbx` archives (user-initiated)
