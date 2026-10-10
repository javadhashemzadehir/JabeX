# ADR 005: Rust Backend in Tauri

## Status

Accepted

## Context

JabeX needs offline domain logic, scheduling, persistence, media, and backups. The UI stack (Next.js/React) must not own business rules or storage.

## Decision

Implement the backend in **Rust** inside the Tauri host:

- Domain + application layers are pure and independently testable
- Tauri commands are thin IPC adapters
- No React/Next dependencies in backend crates

## Consequences

### Positive

- Strong type safety and performance for scheduling/persistence
- Clear process boundary from the webview
- Offline-first by construction

### Negative

- Contributors need a Rust toolchain for desktop work
- IPC contracts must be kept in sync with TypeScript types

## Alternatives Considered

- Business logic in TypeScript only: weaker isolation and harder native FS/DB control
- Separate daemon process: unnecessary complexity for a single desktop app
