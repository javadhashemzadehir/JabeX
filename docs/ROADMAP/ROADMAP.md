# Roadmap

## Phase 0 — Foundation (current)

- [x] Cursor agent rules
- [x] Product & architecture docs + ADRs
- [x] Monorepo workspace layout
- [x] Next.js + Tailwind + TypeScript
- [x] shadcn/ui integration path
- [x] i18n (`fa` / `en`) + RTL/LTR
- [x] Theme system (light / dark / system)
- [x] Tauri scaffold (secure defaults)
- [x] CI (lint, typecheck, test)
- [x] App shell UI (sidebar, header, switchers)

## Phase 1 — Core Leitner MVP

- [x] Deck create / rename / delete
- [x] Card CRUD (front / back)
- [x] Local persistence (SQLite or chosen engine)
- [x] Review session with box promotion/demotion
- [x] Due-card queue
- [x] Empty states optimized for ≤30s first deck / ≤2m first review (decks empty state)

## Phase 1.5 — Local profiles & backup

- [x] Local multi-profile registry (no accounts)
- [x] Per-profile SQLite isolation
- [x] Entity `revision` for future document sync
- [x] Versioned `.jbx` export/import
- [x] Native file picker for backup paths (optional polish)

## Phase 2 — Study Quality

- [x] Keyboard shortcuts for review
- [x] Basic stats (local only)
- [x] Learning consistency (study streaks + 12‑month heatmap from review activity; ADR 013)
- [x] Deck import/export (file-based) — superseded by profile `.jbx` backup
- [x] Accessibility pass (RTL + LTR)
- [x] Leitner interval presets (recommended + packages) per deck / profile
- [x] Hold-to-confirm delete for decks, cards, and profiles

## Phase 2.5 — Rich text cards

- [x] TipTap editor for card front/back (design-system toolbar)
- [x] Versioned JSON document storage (`document_v1`) in SQLite + IPC
- [x] Read-only rich rendering in review
- [x] Media attachments inside documents (local files + metadata; ADR 010)
- [x] Ink / handwriting layers (vector JSON + canvas renderer; ADR 010)

## Phase 2.6 — Smart Language Assist

- [x] Offline language / script detection + Persian transliteration heuristics
- [x] Optional Google Translate + TTS providers behind ports (never mandatory)
- [x] Local pronunciation audio cache under `media/audio/`
- [x] Card metadata + settings toggles + UI badge / play / regenerate (ADR 011)

## Phase 2.7 — Soft delete & Trash

- [x] Domain soft-delete with `deleted_at` + `deleted_by` (local profile UUID)
- [x] Trash repository APIs: list / restore / permanent delete (cascade restore)
- [x] Trash UI in sidebar; hold-to-confirm delete forever
- [x] `.jbx` v3 includes soft-deleted entities (ADR 012)

## Phase 3 — Polish & Distribution

- [ ] Packaging for Windows / macOS / Linux
- [ ] Onboarding polish
- [ ] Performance pass on large decks
- [ ] Contributor documentation maturity

## Explicitly Deferred / Out of Scope

- Cloud sync
- Accounts / auth
- Analytics
- AI features
- Notes / knowledge graph
