# ADR 003: Local-First Storage

## Status

Accepted (engine choice pending implementation phase)

## Context

Privacy and offline use are core product principles. Users must study without an account or network. Card content is personal and must not be sent to third parties.

## Decision

All core application data is stored **locally on the user’s device**, accessed through the Tauri/Rust layer.

- No cloud database for decks/cards/reviews
- No telemetry pipelines
- Frontend talks to storage only via typed IPC commands
- Preferred persistence engine for MVP: **SQLite** (finalized when Phase 1 begins)

## Consequences

### Positive

- Strong privacy story
- Works offline by default
- Simpler compliance posture
- Clear ownership of data files for future export/backup features

### Negative

- Multi-device sync is out of scope (or must be explicit/opt-in later)
- Backup/restore UX becomes a product responsibility
- Schema migrations must be handled carefully on-device

## Alternatives Considered

- **IndexedDB in the webview only**: weaker for desktop file management and Rust domain logic
- **Remote sync backend**: conflicts with privacy-first positioning
- **Embedded key-value only**: possible, but SQLite is more inspectable for relational study data
