# ADR 012: Soft Delete & Profile Trash

## Status

Accepted

## Context

JabeX already used `deleted_at` on study entities, but deletes were incomplete for product UX:

- Deck delete did not cascade to cards / media
- Soft-deleted decks and cards were omitted from `.jbx` backups
- There was no unified Trash screen for restore / permanent delete
- Future Git-based sync needs tombstones (`deleted_at`) and actor metadata without physical deletes during normal use

Profiles remain local-only (no accounts). Permanent delete must stay explicit.

## Decision

### Domain soft delete

Deletable study entities carry:

| Field | Meaning |
|-------|---------|
| `deleted_at` | Tombstone timestamp (ISO-8601 UTC); null = live |
| `deleted_by` | Local profile UUID that performed the soft-delete (optional; sync-ready) |

Normal repository queries exclude `deleted_at IS NOT NULL`. Reviews stay append-only (no soft-delete).

Migration **V6__soft_delete.sql** adds `deleted_by` (and indexes). `deleted_at` has existed since V1.

> Note: The product brief asked for `V3__soft_delete.sql`; V3/V4/V5 were already taken (rich text, rich content, language assist), so soft-delete metadata ships as **V6**.

### Trash repository APIs

Domain port `TrashRepository`:

- `list_deleted_items()`
- `restore_item(kind, id, at)` — restores parent and soft-deleted children
- `permanently_delete_item(kind, id)` — irreversible SQLite removal (media files cleaned by `TrashService`)

`TrashService` also owns cascading **soft** delete for decks/cards (children move to trash together). User delete IPC never hard-deletes.

### Backup

`.jbx` **v3** exports soft-deleted decks, cards, settings, attachments, and ink documents, preserving `deletedAt` / `deletedBy` for roundtrip and future sync.

### UI

Sidebar **Trash** → `/trash` lists deleted items with timestamps, Restore, and hold-to-confirm Delete forever.

### Sync readiness

Soft deletes are tombstones compatible with future Git document sync (ADR 008). Never physically delete during normal study actions; only Trash “Delete forever” (or media-library purge) removes bytes.

## Consequences

### Positive

- Recoverable mistakes without cloud undo
- Consistent parent/child lifecycle
- Backups preserve trash state
- Clear path for Git merge of delete tombstones

### Negative

- Soft-deleted rows accumulate until purge
- Cascade restore may revive cards the user deleted earlier inside a deleted deck (acceptable for v1 Trash)

## Alternatives Considered

- **Hard delete only**: rejected — conflicts with sync readiness and recovery UX
- **Media-library trash only**: rejected — decks/cards need the same model
- **Cloud recycle bin**: rejected — violates local-first / no-accounts
