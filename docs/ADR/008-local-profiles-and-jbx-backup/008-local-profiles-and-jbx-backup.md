# ADR 008: Local Profiles & Versioned `.jbx` Backup

## Status

Accepted

## Context

JabeX must stay offline-first and account-free, yet support:

- Multiple people on one device (family / shared PC)
- Safe backup/restore without treating SQLite binaries as the primary format
- Future optional sync via Git of normalized documents (not DB files)

Cloud identity, passwords, phone/OTP, and remote accounts remain out of scope.

## Decision

### Local profiles

A **profile** is a local identity on the device (display name + optional avatar). Each profile owns an isolated data directory:

```
{app_data}/JabeX/
├── app.sqlite
├── profiles/{profile_uuid}/
│   ├── database.sqlite
│   └── media/
└── backups/*.jbx
```

- Profile registry lives in `app.sqlite` (not inside a profile DB)
- Learning data never crosses profile boundaries
- Active profile is switched locally; no authentication is required

### Sync-ready entity fields

Study entities keep UUIDv7 IDs and gain a monotonic `revision` integer bumped on every write. Soft-delete via `deleted_at` / `deleted_by` remains (reviews stay append-only without those fields). From `.jbx` v3, soft-deleted study entities are included in exports so Trash and tombstones survive backup roundtrips ([ADR 012](../012-soft-delete-and-trash/012-soft-delete-and-trash.md)).

Future Git sync will export normalized JSON documents from SQLite, merge in a repo, then import — never sync `database.sqlite` binaries.

### Backup format (`.jbx`)

Primary backup is a versioned ZIP with extension `.jbx`:

| Path | Purpose |
|------|---------|
| `manifest.json` | Format id, schema version, counts, timestamps |
| `profile.json` | Profile metadata |
| `data/*.json` | Normalized decks, cards, reviews, settings, attachment metadata |
| `media/**` | Referenced media files (when present) |

Import validates the manifest before applying data. Modes: create a new profile, or replace the active profile after explicit confirmation.

Legacy raw ZIP of `database.sqlite` is deprecated for user-facing backup.

## Consequences

### Positive

- Clear data ownership per person on a shared device
- Exportable, migratable backups independent of SQLite file format
- Room for Git-based sync without rewriting the domain model

### Negative

- App boot must open registry + active profile DB
- Profile switch requires rebuilding session services
- One-time migration from the previous single-DB layout

## Alternatives Considered

- **Single shared DB with `profile_id` columns**: weaker isolation; harder to delete/export one profile
- **Raw SQLite as backup**: brittle across versions; conflicts with future document sync
- **Online accounts for multi-user**: rejected by product privacy goals
