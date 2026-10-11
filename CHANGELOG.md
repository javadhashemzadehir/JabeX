# Changelog

All notable changes to JabeX are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-10-11

### Added

- Offline pronunciation and translation pipeline for language assist

### Fixed

- TTS voice select max-width so the settings control fits the UI better

## [0.1.2] - 2026-10-11

### Fixed

- Guided tour stays on the current step when you press Next or Skip. Leaving the decks step no longer restarts the walkthrough.

## [0.1.1] - 2026-10-10

### Changed

- First-run experience uses a React Joyride product tour (deck → card → review → settings); release builds no longer offer an in-app sample deck. Mock study data remains `pnpm db:seed` in development only.

## [0.1.0] - 2026-10-10

### Added

- Private, offline-first digital Leitner Box (desktop)
- Next.js frontend with Persian-first RTL UX and English LTR
- Tauri 2 shell with local SQLite persistence and clean-architecture Rust backend
- Deck and card CRUD with TipTap rich-text documents (`document_v1` / `document_v2`)
- Media attachments and ink/handwriting layers (local-only)
- Leitner review sessions with box promotion/demotion and due-card queue
- Local multi-profile registry with per-profile SQLite isolation
- Versioned `.jbx` profile backup import/export
- Keyboard shortcuts, local study stats, streaks, and 12-month heatmap
- Leitner interval presets per deck/profile
- Soft delete with Trash (list / restore / permanent delete)
- Offline language assist (detection, optional translate/TTS behind ports)
- Global search command palette (FTS5 + fuzzy ranking; local history only)
- Accessibility pass (RTL/LTR), hold-to-confirm deletes, theme system
- Phase 3 polish: packaging docs + bundle metadata, first-run onboarding (sample deck), large-deck card list pagination/virtualization, contributor docs index
- Windows distributions: NSIS installer, MSI installer, and portable ZIP (`pnpm release:assemble` → `release/` + `SHA256SUMS.txt`)
- Portable Windows pack (`pnpm portable:pack` / `portable:build`) with WebView2 requirements docs
- Pre-migration SQLite file backup on profile DB upgrade (`database.sqlite.pre-migrate-*.bak`)

### Security

- No accounts, analytics, or required network for core study flows
- Least-privilege Tauri capabilities; user data stays on device
- No auto-updater in this release (user-downloaded builds only)

[Unreleased]: https://github.com/javadhashemzadehir/JabeX/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/javadhashemzadehir/JabeX/releases/tag/v0.1.3
[0.1.2]: https://github.com/javadhashemzadehir/JabeX/releases/tag/v0.1.2
[0.1.1]: https://github.com/javadhashemzadehir/JabeX/releases/tag/v0.1.1
[0.1.0]: https://github.com/javadhashemzadehir/JabeX/releases/tag/v0.1.0
