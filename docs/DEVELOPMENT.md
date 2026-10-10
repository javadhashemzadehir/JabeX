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

## System tray (Windows)

JabeX can stay running in the notification area (near the clock) instead of quitting when the window is closed.

### How tray mode works

- With **Enable minimize to tray** on, a single tray icon (app branding) is installed once (`jabex-tray`).
- **Minimize** (taskbar) keeps normal Windows minimize behaviour.
- **Close (X)** hides the window when **Close button hides to tray** is on; the process keeps running.
- **Left-click** the tray icon toggles the main window (show/focus ↔ hide).
- **Right-click** opens: Open JabeX, Switch Profile, Create Backup, Restore Backup, Settings, Exit JabeX.
- **Switch Profile** is available from the system tray: it restores/focuses the window (if hidden) and opens the existing header profile switcher. Profile list/switch still uses the normal frontend + profile IPC — no duplicate logic in Rust.

### Restoring the window

1. Left-click the tray icon, or  
2. Right-click → **Open JabeX**, or  
3. Use a tray menu item that opens Settings / Backup / Switch Profile (the window is shown first).

### Completely exiting

Use tray menu **Exit JabeX**. That calls a full process exit. Closing the window alone does **not** quit while close-to-tray is enabled.

### Configuration

Settings → Application (stored in `app.sqlite`, key `app_preferences`):

| Option | Effect |
|--------|--------|
| Enable minimize to tray | Show/hide the tray icon; required for hide-on-close / start minimized |
| Close button hides to tray | X hides instead of quitting |
| Start minimized | Launch hidden in the tray |
| Launch on system startup | Windows: register/unregister current-user Run key |

Disabling the tray removes the icon and restores normal close → quit behaviour.

## Troubleshooting

| Issue | Action |
|-------|--------|
| `cargo` / `rustc` not found | Install Rust via rustup |
| Tauri build fails on Windows | Install WebView2 + MSVC build tools |
| RTL looks wrong | Ensure `dir` on `<html>` matches locale |
| Duplicate tray icons | Restart the app; tray rebuild always removes `jabex-tray` first |
| Close quits unexpectedly | Enable tray + “Close button hides to tray” in Settings |
| Theme flash | Confirm ThemeProvider + `suppressHydrationWarning` on `<html>` |
