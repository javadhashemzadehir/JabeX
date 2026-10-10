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

Local **profiles** isolate study data per person on the device (no online identity). See [ADR 008](./ADR/008-local-profiles-and-jbx-backup.md).

## Repository Structure

| Path | Role |
|------|------|
| `apps/desktop/frontend` | Next.js App Router UI |
| `apps/desktop/src-tauri` | Tauri + Rust backend |
| `packages/ui` | Shared UI package (shadcn-oriented) |
| `packages/types` | Shared TypeScript contracts |
| `packages/config` | Shared tooling config |
| `docs/` | Product, architecture, ADRs |
| `.cursor/rules/` | AI agent rules |

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
| Infrastructure | `infrastructure/` | SQLite, media FS, repository adapters |
| Shared | `shared/` | UUIDv7 IDs, time helpers |

See [BACKEND.md](./BACKEND.md) for data flow, scheduler rules, and IPC command list.

## Cross-Cutting Concerns

- **i18n**: default `fa` + RTL; `en` + LTR
- **Theming**: `next-themes` with CSS variables
- **Validation**: Zod (TS) + Rust validation at IPC boundary
- **Security**: least-privilege Tauri capabilities; no telemetry
- **Storage**: SQLite (WAL) + content-addressed media files

## Related ADRs

- [001 — Next.js + Tauri stack](./ADR/001-next-tauri-stack.md)
- [002 — UI framework choice](./ADR/002-ui-framework-choice.md)
- [003 — Local-first storage](./ADR/003-local-first-storage.md)
- [004 — Internationalization](./ADR/004-internationalization.md)
- [005 — Rust backend](./ADR/005-rust-backend.md)
- [006 — SQLite storage](./ADR/006-sqlite-storage.md)
- [007 — Clean architecture](./ADR/007-clean-architecture.md)
- [008 — Local profiles & `.jbx` backup](./ADR/008-local-profiles-and-jbx-backup.md)
