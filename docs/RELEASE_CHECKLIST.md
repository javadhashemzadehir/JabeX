# Release checklist

Use this before tagging a production Windows (and multi-OS) release.

Related: [PACKAGING.md](./PACKAGING/PACKAGING.md) · [GIT_WORKFLOW.md](./GIT_WORKFLOW/GIT_WORKFLOW.md) · [CHANGELOG.md](../CHANGELOG.md)

## Before release

- [ ] Version bumped in sync:
  - root `package.json`
  - `apps/desktop/package.json`
  - `apps/desktop/frontend/package.json`
  - `apps/desktop/src-tauri/Cargo.toml`
  - `apps/desktop/src-tauri/tauri.conf.json`
- [ ] `CHANGELOG.md` updated (`[Unreleased]` → version section)
- [ ] SQLite migrations smoke-tested on an existing profile (upgrade path)
- [ ] `.jbx` backup export + import tested (create-profile mode)
- [ ] Fresh install tested (empty app data → create profile → first deck)
- [ ] Upgrade install tested (install over previous version; profiles still open)
- [ ] No secrets, `.env`, or user DB files in the commit/tag

## Build (local Windows)

- [ ] `pnpm typecheck` passes
- [ ] `cd apps/desktop/src-tauri && cargo check` passes
- [ ] `pnpm tauri:build` succeeds

Expected artifacts under `apps/desktop/src-tauri/target/release/bundle/`:

- [ ] `nsis/JabeX_<version>_x64-setup.exe`
- [ ] `msi/JabeX_<version>_x64_en-US.msi`

## Installer smoke

- [ ] EXE (NSIS) installs correctly (current-user mode; Start Menu shortcut)
- [ ] MSI installs correctly
- [ ] App launches; local SQLite profiles load (or first-run setup appears)
- [ ] Uninstall removes Start Menu entry and application files
- [ ] Shortcuts open the app
- [ ] Profile / migration behaviour OK after upgrade (existing decks still listed)
- [ ] Optional: create deck, add card, export `.jbx`, re-import

## GitHub Release

- [ ] Annotated tag pushed: `v<version>` (example `v0.1.0`)
- [ ] `release.yml` finished green for Windows (and macOS/Linux if shipping those)
- [ ] Release assets include `.exe` and `.msi`
- [ ] Release notes / CHANGELOG link look correct

## Explicitly deferred

- [ ] Windows Authenticode code signing (not configured)
- [ ] Apple notarization (not configured)
- [ ] Tauri updater signing keys (not configured)
