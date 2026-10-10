# Release checklist

Use this before tagging a production Windows (and multi-OS) release.

Related: [PACKAGING.md](./PACKAGING/PACKAGING.md) · [GIT_WORKFLOW.md](./GIT_WORKFLOW/GIT_WORKFLOW.md) · [CHANGELOG.md](../CHANGELOG.md)

## Version consistency

Confirm the **same** SemVer in all of:

- [ ] root `package.json`
- [ ] `apps/desktop/package.json`
- [ ] `apps/desktop/frontend/package.json`
- [ ] `packages/types/package.json`
- [ ] `apps/desktop/src-tauri/Cargo.toml`
- [ ] `apps/desktop/src-tauri/tauri.conf.json`
- [ ] `CHANGELOG.md` section `## [X.Y.Z] - YYYY-MM-DD`
- [ ] Annotated git tag `vX.Y.Z` (points at the release commit)

## Windows requirements (document for users)

- [ ] README / release notes state: **Windows 10/11 64-bit (x64)**
- [ ] README / release notes state: **Microsoft Edge WebView2 Runtime** required
- [ ] No telemetry, accounts, cloud dependency, or auto-updater in the build

## Before release

- [ ] `CHANGELOG.md` updated (`[Unreleased]` notes moved into the version section)
- [ ] SQLite migrations smoke-tested on an existing profile (upgrade path)
- [ ] Pre-migration backup created on upgrade (`database.sqlite.pre-migrate-*.bak`)
- [ ] Failed-migration recovery path understood (restore `.bak`; see [DATABASE.md](./DATABASE/DATABASE.md))
- [ ] Existing profiles remain intact after upgrade
- [ ] `.jbx` backup export + import tested (create-profile mode)
- [ ] Fresh install tested (empty app data → create profile → first deck → card → review)
- [ ] Upgrade install tested (install over previous version; profiles still open)
- [ ] No secrets, `.env`, or user DB files in the commit/tag

## Build (local Windows)

- [ ] `pnpm typecheck` passes
- [ ] `cd apps/desktop/src-tauri && cargo test` passes (includes migration backup tests)
- [ ] `pnpm tauri:build` succeeds
- [ ] `pnpm release:assemble` produces `release/` artifacts + `SHA256SUMS.txt`

Expected under `apps/desktop/src-tauri/target/release/bundle/`:

- [ ] `nsis/JabeX_<version>_x64-setup.exe`
- [ ] `msi/JabeX_<version>_x64_en-US.msi`

Expected under `release/` (gitignored; attach to GitHub Release):

```
release/
├── JabeX_<version>_x64-setup.exe
├── JabeX_<version>_x64_en-US.msi
├── JabeX-<version>-portable.zip
└── SHA256SUMS.txt
```

## Installer validation

- [ ] NSIS install (current-user; no admin; Start Menu shortcut)
- [ ] Launch after install; first-run profile / migrations OK
- [ ] Uninstall removes Start Menu entry and application files (user data under AppData remains)
- [ ] Reinstall **same** version succeeds
- [ ] Upgrade path: install newer build over previous; existing profiles/decks still listed
- [ ] MSI install / launch smoke (optional if NSIS is the primary channel)

## Portable validation

- [ ] Unzip on a clean user / machine (or isolated `JABEX_DATA_DIR`)
- [ ] `JabeX.exe` runs **without** administrator rights
- [ ] First launch creates SQLite under `%APPDATA%\JabeX\JabeX\data\` (or `JABEX_DATA_DIR`)
- [ ] Profile creation works
- [ ] Card creation works
- [ ] Review session works
- [ ] Backup export/import (`.jbx`) works
- [ ] Portable folder itself has no new `*.sqlite` / `profiles/` / `backups/` writes

Isolated data check:

```powershell
$data = Join-Path $env:TEMP "jabex-portable-smoke"
New-Item -ItemType Directory -Force -Path $data | Out-Null
$env:JABEX_DATA_DIR = $data
# Start release portable JabeX.exe, exercise flows, quit.
# Expect: $data\app.sqlite (+ profiles) exist; zip extract folder unchanged.
```

## GitHub Release

- [ ] Annotated tag pushed: `v<version>` (example `v0.1.0`) on the hardened release commit
- [ ] `release.yml` finished green for Windows (and macOS/Linux if shipping those)
- [ ] Release assets include `.exe`, `.msi`, portable `.zip`, and `SHA256SUMS.txt` (CI may omit portable zip — attach local `release/` assets if needed)
- [ ] Release notes list Windows 10/11 x64 + WebView2 requirements
- [ ] Release notes / CHANGELOG link look correct

## Explicitly deferred (do not add for v0.1.0)

- [ ] Windows Authenticode code signing (not configured)
- [ ] Apple notarization (not configured)
- [ ] Tauri updater / auto-update (not configured — local-first; user downloads new builds)
- [ ] Telemetry, accounts, or cloud sync
