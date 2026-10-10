# ADR 006: SQLite Local Storage

## Status

Accepted

## Context

JabeX requires durable local storage for decks, cards, reviews, attachments metadata, drawings metadata, and settings — without cloud databases.

## Decision

Use **SQLite** via `rusqlite` (bundled) with **refinery** migrations.

- Database file: `JabeX/database.sqlite` under the app data directory
- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- Soft delete via `deleted_at` (except append-only `reviews`)
- Media blobs stored on disk, not in SQLite

## Consequences

### Positive

- Reliable, inspectable, widely understood
- Easy backups (copy DB + media)
- Strong relational integrity for study data

### Negative

- Schema migrations must be careful on user machines
- Concurrent writers are limited (acceptable for single-user desktop)

## Alternatives Considered

- IndexedDB in the webview: weaker for desktop backup/export story
- Embedded key-value store: poorer fit for relational review queries
