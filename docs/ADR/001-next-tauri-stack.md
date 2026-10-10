# ADR 001: Next.js + Tauri Stack

## Status

Accepted

## Context

JabeX needs a desktop-first, offline-capable flashcard app with a modern web UI, Persian RTL support, and strong privacy guarantees. We need a stack that:

- Ships as a native desktop app
- Keeps data local
- Enables a high-quality React UI
- Avoids Electron’s heavier footprint when possible

## Decision

Use **Next.js (App Router, TypeScript)** for the UI and **Tauri (Rust)** as the desktop runtime and local backend.

- Development: Tauri loads the Next.js dev server
- Production: Next.js static export served by Tauri’s webview
- Business-critical persistence and OS integration live in Rust

## Consequences

### Positive

- Smaller binaries and better security posture than Electron
- Familiar React/Next ecosystem for UI
- Clear IPC boundary for local-first architecture

### Negative

- Requires Rust toolchain for desktop builds
- Next.js must remain static-export compatible for production webview use
- Some Next server-only features are unavailable in the desktop build

## Alternatives Considered

- **Electron + React**: heavier, larger attack surface
- **Tauri + plain Vite React**: viable, but Next App Router + ecosystem chosen for structure and RSC where useful in web tooling
- **Pure native UI**: slower iteration for bilingual design system work
