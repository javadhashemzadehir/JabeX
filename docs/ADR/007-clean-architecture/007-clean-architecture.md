# ADR 007: Clean Architecture for Backend

## Status

Accepted

## Context

The backend will grow (import/export, alternate schedulers, richer media). We need boundaries that keep domain logic free of SQLite/Tauri details.

## Decision

Organize Rust code as:

| Layer | Responsibility |
|-------|----------------|
| `domain` | Entities, errors, scheduler trait, repository ports |
| `application` | Use-cases / services |
| `infrastructure` | SQLite, filesystem, repository adapters |
| `commands` | Tauri IPC (validate → service → result) |
| `shared` | Cross-cutting helpers (UUIDv7, time) |

Dependency rule: inward only. Domain never imports rusqlite/Tauri.

## Consequences

### Positive

- Domain tests run without a UI or full app boot
- Scheduler algorithms can be swapped (`LeitnerScheduler` → `FsrsScheduler`)
- Storage engines can evolve behind ports

### Negative

- More files/modules than a flat CRUD layout
- Requires discipline to keep commands thin

## Alternatives Considered

- Transaction-script modules per feature: faster initially, harder to evolve
- Hexagonal with many crates: overkill for current monorepo stage
