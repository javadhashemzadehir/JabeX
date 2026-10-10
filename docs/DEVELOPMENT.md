# Development Guide

## Prerequisites

- Node.js 22+ (LTS recommended)
- pnpm 9+
- Rust stable + Cargo (required for Tauri builds)
- Platform deps for Tauri: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Repository Setup

```bash
pnpm install
cp .env.example .env
```

## Common Commands

```bash
# Frontend only (browser / Next dev)
pnpm --filter @jabex/desktop-frontend dev

# Typecheck / lint / test (workspace)
pnpm lint
pnpm typecheck
pnpm test

# Tauri desktop (requires Rust toolchain)
pnpm tauri:dev
pnpm tauri:build
# equivalent:
pnpm --filter @jabex/desktop tauri:dev
```

## Workspace Layout

- Desktop host: `apps/desktop` (Tauri CLI entry — must see `src-tauri/`)
- App UI: `apps/desktop/frontend`
- Tauri/Rust: `apps/desktop/src-tauri`
- Shared packages: `packages/*`

## Frontend Notes

- App Router + `src/` directory
- Path aliases: `@/*`, `@components/*`, `@features/*`, `@lib/*`
- Default locale: `fa` (RTL)
- Themes via `next-themes`
- UI system: **shadcn/ui** (New York) in `src/components/ui`
  - Add components: `pnpm --filter @jabex/desktop-frontend exec shadcn@latest add <name>`
  - Compatible block sources: [Shadcn Studio](https://shadcnstudio.com/components), [Shadcnblocks](https://www.shadcnblocks.com/), [ReUI](https://reui.io/components)
  - Do not introduce non-shadcn UI kits (FarsiUI, MUI, Chakra, …)
- Fonts: local only (no Google Fonts). Persian (`fa`) uses Iran Yekan Bakh FaNum; English (`en`) uses Comfortaa. Files live in `apps/desktop/frontend/src/fonts`.
- RTL: `dir` on `<html>` + Radix `DirectionProvider`

## Tauri Notes

- Run CLI from `apps/desktop` (workspace package `@jabex/desktop`) so `src-tauri/tauri.conf.json` is discoverable; do not invoke `tauri` from `frontend/` alone
- Dev: Next.js dev server URL in `tauri.conf.json`
- Prod: Next static export consumed by Tauri
- Keep capabilities minimal
- Backend docs: `docs/BACKEND.md`
- Run Rust tests: `cd apps/desktop/src-tauri && cargo test`
- Windows requires MSVC Build Tools (`link.exe`) for `cargo` / Tauri builds

## Git & contributions

- Branching, commits, and releases: [GIT_WORKFLOW.md](./GIT_WORKFLOW.md)
- Contribution steps: [CONTRIBUTING.md](../CONTRIBUTING.md)

## Coding Standards

Follow `.cursor/rules/` and keep changes aligned with ADRs. Do not add analytics, auth, or cloud dependencies.

## Troubleshooting

| Issue | Action |
|-------|--------|
| `cargo` / `rustc` not found | Install Rust via rustup |
| Tauri build fails on Windows | Install WebView2 + MSVC build tools |
| RTL looks wrong | Ensure `dir` on `<html>` matches locale |
| Theme flash | Confirm ThemeProvider + `suppressHydrationWarning` on `<html>` |
