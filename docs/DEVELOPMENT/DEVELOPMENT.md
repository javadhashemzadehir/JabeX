# Development Guide

## Prerequisites

- Node.js 22+ (LTS recommended)
- pnpm 9+
- Rust stable + Cargo via [rustup](https://rustup.rs/) (required for Tauri builds)
- Platform deps for Tauri (summary below; full detail: [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/))

### OS-specific Tauri dependencies

| OS | Required |
| -- | -------- |
| **Windows** | [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload (`link.exe`), plus WebView2 (usually preinstalled on Windows 10/11). For **MSI** installers, enable the Windows **VBScript** optional feature |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Linux** | WebKitGTK and related packages (Ubuntu example: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`) — match versions in `.github/workflows/ci.yml` / `release.yml` |

## Repository Setup

```bash
pnpm install
cp .env.example .env
```

## First successful desktop run

1. Install prerequisites for your OS (table above)
2. `pnpm install` and copy `.env.example` → `.env`
3. `pnpm tauri:dev` from the repo root
4. Confirm the window opens, sidebar navigates, and you can create a deck
5. Optional: `pnpm db:seed` for mock study data (quit the app first if SQLite is busy)

Frontend-only (no Rust): `pnpm dev` → [http://localhost:3000](http://localhost:3000). Profile setup and IPC features require the desktop shell.

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

# Local mock study data (active profile SQLite)
pnpm db:seed                 # insert fixtures from scripts/dev/seed-data.json (dev only — never auto-runs in release)
pnpm db:unseed               # remove only those seed rows
pnpm db:unseed -- --purge    # wipe ALL decks/cards/reviews in the profile
# Production / portable EXE starts with an empty profile library.
# In `tauri:dev` only, Home may show a “sample deck (dev)” shortcut; release builds never do.

# Docs screenshots (isolated data only — see below)
pnpm docs:screenshots
```

### Documentation screenshots

`pnpm docs:screenshots` captures the README image set into `docs/screenshots/` from a live `tauri:dev` session.

| Concern | Behavior |
| -------- | -------- |
| Data isolation | Always sets `JABEX_DATA_DIR` to `<repo>/.tmp/docs-capture-data` and **refuses** paths under `%APPDATA%` |
| Demo content | Bootstraps profiles `Learner` + `Study`, then `pnpm db:seed -- --profile Learner` from `scripts/dev/seed-data.json` |
| Window | Targets **1440×900** via CDP `Browser.setWindowBounds` + device metrics |
| Defaults | Most shots: `fa` + RTL + light; `12` = dark; `13` = `en` LTR; `15` = empty `Study` profile |
| Personal DB | Never opened — quit any everyday JabeX window first so the debug port / process tree stay clean |

```bash
# From repo root (Rust + Node + pnpm required)
pnpm docs:screenshots
```

Outputs: `docs/screenshots/01-app-shell.png` … `15-empty-states.png` plus `manifest.json`.  
Safe to delete `.tmp/docs-capture-data` afterward; it is gitignored.

### Windows production installers

```bash
pnpm typecheck
cd apps/desktop/src-tauri && cargo check && cd ../../..
pnpm tauri:build
```

Outputs (version from `tauri.conf.json`, currently `0.1.2`):

```
apps/desktop/src-tauri/target/release/bundle/nsis/JabeX_0.1.2_x64-setup.exe
apps/desktop/src-tauri/target/release/bundle/msi/JabeX_0.1.2_x64_en-US.msi
```

Portable (no installer) folder: `pnpm portable:build` → `dist/JabeX-<version>-portable/` (see [PACKAGING.md](../PACKAGING/PACKAGING.md#portable-windows-zip-no-installer)).

Full steps, CI tagging, and signing status: [PACKAGING.md](../PACKAGING/PACKAGING.md) · [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md).

Seed scripts target the local app-data directory (`directories` ProjectDirs). Quit the running app if you hit `SQLITE_BUSY`. Override the path with `JABEX_DATA_DIR` when needed.

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
- Tailwind CSS v4: use canonical logical utilities (`inset-s-*` / `inset-e-*`, `ms-*` / `me-*`, `ps-*` / `pe-*`, `border-s-*` / `border-e-*`, `text-start` / `text-end`, `bg-linear-to-*`). Do not use deprecated inset aliases (`start-*` / `end-*`) or physical left/right spacing when a logical equivalent exists.

## Tauri Notes

- Run CLI from `apps/desktop` (workspace package `@jabex/desktop`) so `src-tauri/tauri.conf.json` is discoverable; do not invoke `tauri` from `frontend/` alone
- Dev: Next.js dev server URL in `tauri.conf.json`
- Prod: Next static export consumed by Tauri
- Keep capabilities minimal (`dialog:allow-open` / `dialog:allow-save` for `.jbx` pickers only)
- Backend docs: `docs/BACKEND/BACKEND.md`
- Packaging / installers: [PACKAGING.md](../PACKAGING/PACKAGING.md)
- Release checklist: [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md)
- Run Rust tests: `cd apps/desktop/src-tauri && cargo test`
- Windows requires MSVC Build Tools (`link.exe`) for `cargo` / Tauri builds

## Git & contributions

- Branching, commits, and releases: [GIT_WORKFLOW.md](../GIT_WORKFLOW/GIT_WORKFLOW.md)
- Tag-triggered installers: push `v*` → `.github/workflows/release.yml` (Windows `.exe` + `.msi`)
- Contribution steps: [CONTRIBUTING.md](../../CONTRIBUTING.md)

## Coding Standards

Keep changes aligned with ADRs. Do not add analytics, auth, or cloud dependencies.

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
