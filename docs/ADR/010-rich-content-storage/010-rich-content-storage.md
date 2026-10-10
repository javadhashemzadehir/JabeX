# ADR 010: Rich Content Storage (Media Attachments & Ink Layers)

## Status

Accepted

## Context

JabeX cards already store versioned TipTap JSON faces ([ADR 009](../009-tiptap-rich-text-documents/009-tiptap-rich-text-documents.md)). Product direction requires:

1. **Media attachments** (images, audio, video, PDF) on cards and future documents
2. **Ink / handwriting annotation layers** as a separate content plane

Constraints from product and prior ADRs:

- Offline-first, no cloud, no accounts ([ADR 003](../003-local-first-storage/003-local-first-storage.md))
- SQLite holds relational study data; binaries must not live as opaque blobs in the DB ([ADR 006](../006-sqlite-storage/006-sqlite-storage.md))
- Profile-scoped isolation under `profiles/{uuid}/` ([ADR 008](../008-local-profiles-and-jbx-backup/008-local-profiles-and-jbx-backup.md))
- Clean Architecture: domain must not know filesystem, Tauri, or SQLite ([ADR 007](../007-clean-architecture/007-clean-architecture.md))
- Future optional sync is document/Git oriented — never sync `database.sqlite` binaries

V1 schema already reserved `attachments` and `drawings` tables plus a content-addressed `MediaManager` under `media/images|audio|drawings`. Those tables had no domain/application layer. This ADR replaces that layout with a production-ready, owner-polymorphic attachment model and a file-backed ink document model.

## Decision

### Why media is stored outside SQLite

| Concern | SQLite BLOBs | Files on disk |
|---------|--------------|---------------|
| Backup / Git sync | Opaque; poor diffs | Content-hashable; path-addressable |
| Large video/PDF | Inflates DB; WAL pain | Streamable copy/move |
| Dedup & integrity | Awkward | SHA-256 checksums + optional hard links later |
| Partial restore | Hard | Per-file validation |

**Decision:** SQLite stores **metadata and references only**. Binary payloads and ink stroke trees live under the profile media tree.

### On-disk layout (per profile)

```
profiles/{profile_id}/
├── database.sqlite
└── media/
    ├── attachments/
    │   └── {attachment_uuid}/
    │       ├── original.{ext}      # or original.bin when ext unknown
    │       ├── metadata.json       # mirror of DB row + thumbnail index
    │       └── thumbnails/         # optional generated previews
    └── ink/
        └── {ink_document_uuid}.json
```

Legacy folders (`media/images`, `media/audio`, `media/drawings`) are no longer created for new installs. Existing files remain readable if referenced; new writes use the layout above.

### Attachment references

Domain entity `Attachment` (UUID primary key) with:

| Field | Role |
|-------|------|
| `id` | Stable UUIDv7 |
| `owner_type` | `card` \| `document` (extensible) |
| `owner_id` | Owner UUID |
| `file_name` | Original display name |
| `mime_type` | Validated whitelist |
| `size_bytes` | Enforced per-type limits |
| `hash` | SHA-256 hex of file bytes |
| `relative_path` | Profile-relative POSIX path (e.g. `media/attachments/{id}/original.png`) |
| `revision` | Monotonic write counter |
| `created_at` / `updated_at` / `deleted_at` | Soft delete |

Rich-text documents may later embed `{ type: "attachmentRef", attrs: { id } }` nodes; the attachment row remains the source of truth. Domain never opens files — application use cases call an `AttachmentStorage` port implemented by infrastructure (`LocalMediaStorage`).

### Ink data representation

Ink is a **vector stroke document**, not canvas bitmaps:

```json
{
  "version": 1,
  "id": "<uuid>",
  "width": 800,
  "height": 600,
  "layers": [
    {
      "id": "<uuid>",
      "order": 0,
      "visible": true,
      "opacity": 1.0,
      "strokes": [
        {
          "id": "<uuid>",
          "tool": "pen",
          "color": "#111827",
          "width": 2.5,
          "points": [
            { "x": 10.0, "y": 20.0, "pressure": 0.7, "timestamp": 0 }
          ]
        }
      ]
    }
  ]
}
```

- **SQLite `ink_documents`**: `id`, `owner_type`, `owner_id`, `relative_path`, `width`, `height`, `content_hash`, `revision`, timestamps, soft delete
- **File**: full `InkDocument` JSON at `media/ink/{id}.json`
- Frontend talks to an `InkRenderer` abstraction (canvas impl first); undo/redo and tools live in the feature module; persistence is debounced through application services

The V1 `drawings` table is superseded by `ink_documents` (migrated in V4). Bitmap “drawing” blobs are not used for handwriting.

### Future Git-based sync (not implemented)

Prepare only — do not implement sync now:

1. Export normalized JSON for attachment metadata + ink document envelopes (already true for `.jbx`)
2. Binary files and ink JSON files live beside metadata with stable UUIDs and content hashes
3. Merge strategy sketch:
   - **Metadata rows**: last-writer-wins on `revision` / `updated_at`, or CRDT later; soft-deleted wins if `deleted_at` set
   - **Attachment binaries**: content-addressed by `hash`; conflicts are rare (same UUID different hash → keep both under conflict suffix, flag in UI)
   - **Ink JSON**: file-level merge is hard for stroke trees; prefer whole-document LWW by `revision`, or future operational transform on stroke IDs
4. Never sync `database.sqlite` or WAL files

### Migration strategy

| Version | Change |
|---------|--------|
| V1–V3 | Existing schema (`attachments`, `drawings` stubs + TipTap faces) |
| **V4** | Rebuild `attachments` to owner-polymorphic columns; create `ink_documents`; drop `drawings` after empty/legacy copy |

V4 is additive for data safety:

1. Create `attachments_v2` / rename dance with `INSERT…SELECT` mapping `card_id → owner_id`, `kind → mime/type hints`, `path → relative_path`, `sha256 → hash`
2. Create `ink_documents` from any `drawings` rows (path/hash preserved; width/height defaults)
3. Drop old tables only after copy
4. Ensure media directories `attachments/` and `ink/` exist

No destructive wipe of user media files.

### Backup / export implications (`.jbx`)

Bump backup format awareness (manifest fields; keep backward-compatible import of v1):

```
manifest.json          # + mediaVersion, attachment/ink counts, optional file checksums
data/attachments.json  # metadata rows
data/ink_documents.json
media/attachments/**   # binaries + per-attachment metadata.json / thumbnails
media/ink/**           # ink JSON files
```

Import must:

- Reject zip entries that escape `media/` via `..` or absolute paths
- Verify listed checksums when present
- Restore SQLite rows then copy files under the target profile root
- Accept older backups missing attachment/ink payloads (empty lists)

### Frontend boundaries

| Module | Responsibility |
|--------|----------------|
| `features/attachments` | Picker, list, preview, viewer; Zustand optional |
| `features/ink` | `InkRenderer` port, canvas renderer, tools, undo/redo, serialization helpers |
| `features/editor` | May reference attachment IDs later; no FS access |
| IPC | Typed commands only; large files prefer path-based create over byte arrays |

### Limits & validation (application layer)

- MIME whitelist: common image/audio/video types + `application/pdf`
- Size caps (configurable constants): images 20 MiB, audio 50 MiB, video 200 MiB, PDF 50 MiB
- Soft delete leaves files until explicit purge/GC via `MediaLibraryService` (`purge_attachment` / `purge_deleted_attachments`); restore clears `deleted_at`

## Consequences

### Positive

- Aligns media/ink with profile isolation and `.jbx` / future Git sync
- Domain stays pure; storage details stay in infrastructure
- Vector ink scales better than frame dumps; debounced persistence protects the DB/FS

### Negative

- Two stores to keep consistent (SQLite row + file); create/delete must be transactional in intent (best-effort cleanup on failure)
- Thumbnail generation is optional and may lag for video/PDF
- Ink whole-document LWW is a sync compromise until stroke-level merge exists

## Alternatives Considered

- **Keep content-addressed `media/images/{sha}.ext` only**: good dedup, weaker per-attachment metadata/thumbnail folders and owner lifecycle
- **Store ink strokes as SQLite rows**: poor for large drawings; harder `.jbx`/Git story
- **HTML/`src` blobs inside TipTap JSON**: rejected — couples documents to binaries and bloats faces
- **Cloud object storage**: rejected by product privacy goals
