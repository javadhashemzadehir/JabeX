# ADR 006: SQLite Local Storage

## Status

Accepted

## Context

JabeX requires durable local storage for decks, cards, reviews, attachments metadata, drawings metadata, and settings — without cloud databases.

## Decision

Use **SQLite** via `rusqlite` (bundled) with **refinery** migrations.

- Profile study DB: `profiles/{uuid}/database.sqlite` under the app data directory (legacy single-file layouts are migrated on first launch)
- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- Soft delete via `deleted_at` / `deleted_by` (except append-only `reviews`)
- Media blobs stored on disk, not in SQLite

### Persistence style (no ORM)

Keep explicit SQL behind Clean Architecture repository ports:

- **Query modules** — SQL constants per entity (`infrastructure/repositories/queries/`)
- **Named row mappers** — `Row` → domain types by column name (`infrastructure/repositories/mappers/`)
- **Helpers** — UUID/time/enum/JSON conversion + `require_changed`
- **Transactions** — `Database::with_transaction` for multi-step writes (trash cascade, `.jbx` import)

Do **not** introduce SeaORM, Diesel, or sqlx for product persistence. A full ORM adds dependency weight and tends to blur the domain/infrastructure boundary that ADR 007 requires.

## Consequences

### Positive

- Reliable, inspectable, widely understood
- Easy backups (copy DB + media)
- Strong relational integrity for study data
- SQL and mapping stay reviewable and testable without codegen

### Negative

- Schema migrations must be careful on user machines
- Concurrent writers are limited (acceptable for single-user desktop)
- Repositories require more hand-written SQL than an ORM would

## Alternatives Considered

- IndexedDB in the webview: weaker for desktop backup/export story
- Embedded key-value store: poorer fit for relational review queries
- Full ORM (SeaORM / Diesel) or sqlx macros: rejected for local-first desktop scope; layered rusqlite + refinery is sufficient
