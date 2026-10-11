# Architecture

## Overview

JabeX desktop is a **Tauri** application embedding a **Next.js** frontend. All study data is stored locally through the Rust host. There is no cloud backend for core features.

```
┌─────────────────────────────────────────┐
│              Tauri Window               │
│  ┌───────────────────────────────────┐  │
│  │     Next.js frontend (webview)    │  │
│  │  UI · i18n · theme · Zustand      │  │
│  └───────────────┬───────────────────┘  │
│                  │ IPC (commands)       │
│  ┌───────────────▼───────────────────┐  │
│  │   Rust backend (Clean Architecture)│  │
│  │  commands → application → domain  │  │
│  │           ↓                       │  │
│  │      infrastructure               │  │
│  └───────────────┬───────────────────┘  │
│                  │                      │
│  app.sqlite + profiles/{id}/sqlite+media│
│  backups/*.jbx (user-initiated)         │
└─────────────────────────────────────────┘
```

Local **profiles** isolate study data per person on the device (no online identity). See [ADR 008](../ADR/008-local-profiles-and-jbx-backup/008-local-profiles-and-jbx-backup.md).

## Repository Structure

| Path | Role |
|------|------|
| `apps/desktop/frontend` | Next.js App Router UI |
| `apps/desktop/src-tauri` | Tauri + Rust backend |
| `packages/types` | Shared TypeScript contracts |
| `docs/` | Product, architecture, ADRs (each doc in its own folder) |

## Frontend Layers

- **`app/`** — routes and layouts (RSC by default)
- **`components/`** — shell and shared presentational UI
- **`features/`** — feature modules (decks, review, settings)
- **`lib/`** — pure utilities, i18n, theme helpers
- **`locales/`** — translation catalogs (`fa`, `en`)

## Backend Layers (Clean Architecture)

| Layer | Path | Responsibility |
|-------|------|----------------|
| IPC | `commands/` | Thin Tauri adapters |
| Application | `application/` | Use-cases / services |
| Domain | `domain/` | Models, scheduler, repository ports, errors |
| Infrastructure | `infrastructure/` | SQLite, media FS, repository adapters, optional provider adapters |
| Shared | `shared/` | UUIDv7 IDs, time helpers |

See [BACKEND.md](../BACKEND/BACKEND.md) for data flow, scheduler rules, and IPC command list.
Optional Smart Language Assist (detection / translation / TTS ports) is documented in [ADR 011](../ADR/011-smart-language-assist/011-smart-language-assist.md).

## Cross-Cutting Concerns

- **i18n**: default `fa` + RTL; `en` + LTR
- **Theming**: `next-themes` with CSS variables; Tailwind CSS v4 logical utilities for direction-aware layout

- **Validation**: Zod (TS) + Rust validation at IPC boundary
- **Security**: least-privilege Tauri capabilities; no telemetry
- **Storage**: SQLite (WAL) + profile media tree (`media/attachments/`, `media/ink/`) — binaries never as SQLite BLOBs ([ADR 010](../ADR/010-rich-content-storage/010-rich-content-storage.md))
- **Desktop shell**: system tray / window visibility live in `src-tauri/src/desktop/` (not in domain services)

## System tray (Windows)

Tray mode is a **desktop-shell** concern managed in Rust:

| Concern | Location |
|---------|----------|
| Tray icon + menu + close-to-hide | `desktop/tray.rs` |
| Device preferences (JSON in `app.sqlite` `app_meta`) | `AppPreferences` + `AppPreferencesService` |
| Launch on startup (HKCU Run) | `infrastructure/autostart.rs` (Windows) |
| UI toggles | Settings → Application (`/settings`) |
| Tray → UI actions | Event `tray://action` → frontend listener |

`tray://action` payloads include UI-only intents such as `{ "action": "switch-profile" }` (opens the existing profile switcher), `settings`, and `restoreBackup`. Rust restores the window before emitting; profile switching still goes through existing profile commands/stores.

Preferences are **device-level** (not per profile) so close-to-tray behaviour does not change when switching local profiles. Switch Profile is available from the system tray.

See [DEVELOPMENT.md](../DEVELOPMENT/DEVELOPMENT.md#system-tray-windows) for user-facing restore/exit behaviour.

## Related ADRs

- [001 — Next.js + Tauri stack](../ADR/001-next-tauri-stack/001-next-tauri-stack.md)
- [002 — UI framework choice](../ADR/002-ui-framework-choice/002-ui-framework-choice.md)
- [003 — Local-first storage](../ADR/003-local-first-storage/003-local-first-storage.md)
- [004 — Internationalization](../ADR/004-internationalization/004-internationalization.md)
- [005 — Rust backend](../ADR/005-rust-backend/005-rust-backend.md)
- [006 — SQLite storage](../ADR/006-sqlite-storage/006-sqlite-storage.md)
- [007 — Clean architecture](../ADR/007-clean-architecture/007-clean-architecture.md)
- [008 — Local profiles & `.jbx` backup](../ADR/008-local-profiles-and-jbx-backup/008-local-profiles-and-jbx-backup.md)
- [009 — TipTap rich-text documents](../ADR/009-tiptap-rich-text-documents/009-tiptap-rich-text-documents.md)
- [010 — Rich content storage (media + ink)](../ADR/010-rich-content-storage/010-rich-content-storage.md)
- [011 — Smart Language Assist](../ADR/011-smart-language-assist/011-smart-language-assist.md)
- [012 — Soft delete & Trash](../ADR/012-soft-delete-and-trash/012-soft-delete-and-trash.md)
- [013 — Study activity & learning consistency](../ADR/013-study-activity-consistency/013-study-activity-consistency.md)
- [014 — Advanced card editor](../ADR/014-advanced-card-editor/014-advanced-card-editor.md)
- [015 — Global search (FTS5)](../ADR/015-global-search-fts5/015-global-search-fts5.md)

## Global search

Header command palette (`components/search`) calls `search_global` over IPC. Rust maintains a per-profile SQLite FTS5 index (`search_index` / `search_index_fts`) with incremental updates on card/deck/attachment mutations. Domain modules own text extraction, Persian/Arabic normalization, fuzzy ranking, and highlighting; infrastructure owns FTS queries. Recent searches (clear-all or delete-one), popular queries (from local history), and selection history stay on-device (no telemetry). Keyboard shortcuts (including search open) are device-level in `app_preferences` with defaults and remapping in Settings. See [ADR 015](../ADR/015-global-search-fts5/015-global-search-fts5.md).

## Soft delete & Trash

Normal study actions never hard-delete. Soft-deleted decks, cards, attachments, and ink appear in `features/trash` (`/trash`). Restore cascades to children; permanent delete is explicit (hold-to-confirm) and removes SQLite rows plus media files. See [ADR 012](../ADR/012-soft-delete-and-trash/012-soft-delete-and-trash.md).

## Card editor (rich text)

Card faces are TipTap JSON (`document_v2` writes; v1 readable). UI lives in `features/editor` (Edit / Markdown modes). Spoilers, KaTeX math, tables, and highlighted code are schema nodes — never stored as free HTML. See [ADR 014](../ADR/014-advanced-card-editor/014-advanced-card-editor.md).

## Rich content (media + ink)

Card faces remain TipTap JSON. Attachments and ink are **separate layers**:

| Concern | SQLite | Disk |
|---------|--------|------|
| Attachment metadata | `attachments` | `media/attachments/{uuid}/` |
| Ink reference | `ink_documents` | `media/ink/{uuid}.json` |

Frontend feature modules:

- `features/editor` — TipTap card faces, modes, paste/Markdown helpers
- `features/attachments` — picker, list, preview, viewer (per-card)
- `features/media-library` — profile media browser, trash/purge, integrity (`/media`)
- `features/trash` — unified soft-delete Trash (restore / delete forever)
- `features/ink` — `InkRenderer` abstraction + canvas implementation, undo/redo, debounced save

Domain ports: `AttachmentStorage`, `InkStorage`, `AttachmentRepository`, `InkDocumentRepository`. Application: `MediaLibraryService` for library stats/GC. Infrastructure implements local FS + SQLite only.
