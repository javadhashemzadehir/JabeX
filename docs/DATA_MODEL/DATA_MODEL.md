# Data Model

Local-first entities for JabeX study data. All IDs are UUIDv7. Soft delete uses `deleted_at` + optional `deleted_by` (local profile UUID) unless noted. Every mutable study entity has a monotonic `revision` for future document sync. See [ADR 012](../ADR/012-soft-delete-and-trash/012-soft-delete-and-trash.md).

## Profiles (device registry — `app.sqlite`)

| Entity | Notes |
|--------|-------|
| Profile | Local display identity; not an online account |
| App preferences | Device-level tray/window settings |

## Study entities (per-profile `database.sqlite`)

| Entity | Notes |
|--------|-------|
| Deck | Soft-delete (`deleted_at` / `deleted_by`); optional algorithm override in `settings` JSON |
| Card | Soft-delete; faces are versioned TipTap JSON (`document_v1`/`document_v2`; writers emit v2) |
| Review | Append-only grade history |
| Setting | Per-profile key/value JSON; soft-delete capable |
| Attachment | Metadata only; binary under `media/attachments/{id}/`; soft-delete |
| InkDocumentRef | Reference only; strokes under `media/ink/{id}.json`; soft-delete |
| TrashItem | Unified view of soft-deleted study entities (not a table) |
| SearchIndexDocument | Denormalized FTS row for cards/decks/tags/commands/… (ADR 015) |
| RecentSearch / SearchHistoryEntry | Local query + selection history (never uploaded) |

## Card content layers

```
Card
 ├── Front Document (RichText JSON)
 ├── Back Document (RichText JSON)
 ├── Attachments (files on disk, referenced by UUID)
 └── Ink Documents (vector stroke JSON on disk)
```

Rich text may later embed `attachmentRef` nodes; attachment rows remain the source of truth.

## Attachment

| Field | Type | Notes |
|-------|------|-------|
| id | UUID | Stable |
| owner_type | `card` \| `document` | Polymorphic owner |
| owner_id | UUID | |
| file_name | string | Original name |
| mime_type | string | Whitelisted |
| size_bytes | u64 | Validated per type |
| hash | string | SHA-256 hex |
| relative_path | string | Profile-relative POSIX path |
| revision | u32 | |
| created_at / updated_at / deleted_at | timestamps | Soft delete |

On disk:

```
media/attachments/{id}/original.{ext}
media/attachments/{id}/metadata.json
media/attachments/{id}/thumbnails/
```

## Ink

Vector model (never frame dumps):

| Type | Fields |
|------|--------|
| InkDocument | version, id, width, height, layers[] |
| InkLayer | id, order, visible, opacity, strokes[] |
| Stroke | id, tool (`pen`\|`highlighter`\|`eraser`), color, width, points[] |
| Point | x, y, pressure, timestamp |

SQLite stores `InkDocumentRef` (`relative_path`, `content_hash`, dimensions, revision, soft delete).

## Backup (`.jbx`)

Normalized JSON + `media/**`. Manifest includes `mediaVersion` and per-file checksums. See [ADR 010](../ADR/010-rich-content-storage/010-rich-content-storage.md).

## Sync preparation (not implemented)

Stable UUIDs, content hashes, `revision`, timestamps, and soft delete support future Git-based merge of metadata JSON + binary/ink files — never of `database.sqlite`.
