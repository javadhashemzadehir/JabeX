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
- [ ] Native file picker for backup paths (optional polish)

## Phase 2 — Study Quality

- [x] Keyboard shortcuts for review
- [ ] Basic stats (local only)
- [ ] Deck import/export (file-based) — superseded in part by `.jbx`
- [ ] Accessibility pass (RTL + LTR)

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
