# Packaging

How to build Windows installers for JabeX and how CI publishes GitHub Releases.

Related:

- [DEVELOPMENT.md](../DEVELOPMENT/DEVELOPMENT.md) — local setup
- [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md) — pre-tag verification
- [GIT_WORKFLOW.md](../GIT_WORKFLOW/GIT_WORKFLOW.md) — tagging and release process
- [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — multi-OS release job

## End-user Windows requirements

| Requirement | Notes |
| ----------- | ----- |
| OS | **Windows 10 or Windows 11**, **64-bit (x64)** |
| Runtime | **Microsoft Edge WebView2 Runtime** (Evergreen). Usually preinstalled with Windows / Edge; [download](https://developer.microsoft.com/microsoft-edge/webview2/) if the window fails to open |
| Privileges | NSIS uses `currentUser` install mode — **no admin** for a typical install. Portable ZIP also runs without elevation |
| Network | Not required for study. Installers may download WebView2 bootstrapper if missing (`webviewInstallMode: downloadBootstrapper`) |

JabeX remains local-first: **no telemetry**, **no accounts**, **no cloud dependency**, **no auto-updater**.

## Bundle configuration

Source of truth: [`apps/desktop/src-tauri/tauri.conf.json`](../../apps/desktop/src-tauri/tauri.conf.json).

| Field | Purpose |
| ----- | ------- |
| `productName` | Installer / app display name (`JabeX`) |
| `identifier` | Bundle ID (`app.jabex.desktop`) |
| `version` | SemVer (`0.1.0`); keep in sync with Cargo / `package.json` |
| `bundle.active` | Must be `true` for installer generation |
| `bundle.targets` | Windows defaults: `["nsis", "msi"]` |
| `bundle.publisher` | Installer publisher (`JabeX Contributors`) |
| `bundle.category` | Store / OS category (`Education`) |
| `bundle.shortDescription` / `longDescription` | Installer metadata |
| `bundle.copyright` | Copyright line |
| `bundle.windows.nsis.installMode` | `currentUser` (no admin; Start Menu + uninstall) |
| `bundle.icon` | PNG / ICNS / ICO under `src-tauri/icons/` (`icon.ico` for Windows) |

Frontend production assets come from Next static export (`frontendDist`: `../frontend/out`).

NSIS provides the Start Menu shortcut and Add/Remove Programs uninstall entry. MSI is built with WiX (en-US).

macOS / Linux CI jobs pass `--bundles dmg` / `--bundles appimage` and override the default Windows targets.

## Local Windows build

### Prerequisites

- Node.js 22+, pnpm 9+
- Rust stable (rustup)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** (`link.exe`)
- WebView2 (usually preinstalled on Windows 10/11)
- For **MSI**: Windows optional feature **VBScript** enabled (Settings → Apps → Optional features). Errors like `failed to run light.exe` usually mean this is missing.

Full OS matrix: [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

### Commands

From the **repo root**:

```bash
pnpm install
pnpm typecheck
cd apps/desktop/src-tauri && cargo check && cd ../../..
pnpm tauri:build
```

Equivalents:

```bash
pnpm --filter @jabex/desktop tauri:build
# or, from apps/desktop:
pnpm tauri build
```

`pnpm tauri:build` runs the frontend production build (`beforeBuildCommand`), then Tauri release bundling for `nsis` + `msi`.

### Artifact locations

Under `apps/desktop/src-tauri/target/release/bundle/`:

```
msi/
  JabeX_0.1.0_x64_en-US.msi
nsis/
  JabeX_0.1.0_x64-setup.exe
```

| Artifact | Path pattern |
| -------- | ------------ |
| NSIS setup EXE | `nsis/JabeX_<version>_x64-setup.exe` |
| MSI | `msi/JabeX_<version>_x64_en-US.msi` |

### Public release folder

After a successful `pnpm tauri:build`, assemble checksummed distribution files:

```bash
pnpm release:assemble
```

```
release/                          # gitignored
├── JabeX_0.1.0_x64-setup.exe
├── JabeX_0.1.0_x64_en-US.msi
├── JabeX-0.1.0-portable.zip
└── SHA256SUMS.txt
```

`SHA256SUMS.txt` uses the common `hash  filename` layout (two spaces). Verify on Windows:

```powershell
Get-FileHash .\release\JabeX_0.1.0_x64-setup.exe -Algorithm SHA256
Get-Content .\release\SHA256SUMS.txt
```

Smoke-check on a clean Windows user/profile when possible. **Unsigned** builds may trigger SmartScreen (“Windows protected your PC”) — expected until Authenticode signing is added.

## Portable Windows zip (no installer)

Use this when you want a standalone folder — not NSIS/MSI. No administrator privileges are required to run the app.

### Data location (important)

Portable mode **does not write SQLite, profiles, media, or backups into the portable folder**.

Writable data always goes to the OS app-data root (same as the installed app):

| OS | Default path |
| -- | ------------ |
| Windows | `%APPDATA%\JabeX\JabeX\data\` |

Optional override for tests/tooling: set `JABEX_DATA_DIR` to an absolute directory before launch. Do not point it at the portable folder unless you intentionally want data there.

### Build commands

From the **repo root** (frontend must be built so Tauri can embed `frontend/out`):

```bash
pnpm build
cd apps/desktop/src-tauri
cargo build --release
```

Release binary:

```
apps/desktop/src-tauri/target/release/JabeX.exe
```

Assemble the portable folder (copies the exe + docs; **does not** create installers):

```bash
pnpm portable:pack
```

Or build frontend + release binary + pack in one step:

```bash
pnpm portable:build
```

### Output layout

```
dist/JabeX-0.1.0-portable/
├── JabeX.exe
├── resources/
├── WebView2 requirements.txt
└── README.txt
```

Prefer `pnpm release:assemble` to produce `release/JabeX-0.1.0-portable.zip` with checksums. `dist/` and `release/` are gitignored.

### Portable smoke checks

- [ ] `JabeX.exe` launches without elevation
- [ ] First run creates/migrates SQLite under `%APPDATA%\JabeX\JabeX\data\` (or `JABEX_DATA_DIR`)
- [ ] Profiles load; create deck / card works
- [ ] Backup export/import (`.jbx`) works
- [ ] After a short session, the portable folder has **no** new `*.sqlite` / `profiles/` / `backups/` writes (only the shipped files)

Isolated data check:

```powershell
$data = Join-Path $env:TEMP "jabex-portable-smoke"
New-Item -ItemType Directory -Force -Path $data | Out-Null
$env:JABEX_DATA_DIR = $data
# Start dist\JabeX-0.1.0-portable\JabeX.exe, use the app briefly, then quit.
# Expect: $data\app.sqlite exists; portable folder unchanged aside from OS junk.
```

## CI release (tag-triggered)

Pushing an annotated tag matching `v*` (for example `v0.1.1` or `v0.1.1-beta.1`) runs `release.yml`:

| Job | Runner | Bundle args |
| --- | ------ | ----------- |
| windows | `windows-latest` | `--bundles nsis,msi` → `.exe` + `.msi` |

macOS / Linux packaging is deferred until `tauri.conf.json` bundle targets exist for those platforms. The workflow publishes a GitHub Release with the Windows installers attached. Tags containing `-` are marked prerelease.

Version sync before tagging: root `package.json`, `apps/desktop/package.json`, `apps/desktop/frontend/package.json`, `apps/desktop/src-tauri/Cargo.toml`, `tauri.conf.json`, and `CHANGELOG.md` (see GIT_WORKFLOW and [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md)).

## Code signing (deferred)

Phase 3 ships **unsigned** installers first so the pipeline can be proven end-to-end.

Not yet configured:

- Windows Authenticode / SmartScreen reputation
- Apple Developer ID signing + notarization
- Tauri updater signing keys

When added, secrets belong in GitHub Actions (never in the repo). Prefer documenting the exact secret names and notarization steps in this file when signing lands.

## Database / upgrade safety

- Migrations run automatically when a profile database opens
- On upgrade (pending migrations + existing schema history), JabeX writes `database.sqlite.pre-migrate-<UTC>.bak` beside the live DB before applying SQL
- A failed migration surfaces an error that includes the backup path; restore the `.bak` manually if needed (see [DATABASE.md](../DATABASE/DATABASE.md))
- Existing profiles stay isolated under `%APPDATA%\JabeX\JabeX\data\profiles\`

## Privacy / architecture (do not break)

Release packaging must keep JabeX:

- Local-first (SQLite + media on device)
- No required accounts or cloud sync
- No telemetry
- No auto-updater in v0.1.0
- Profile isolation and `.jbx` backup/import unchanged

## Troubleshooting

| Symptom | Check |
| ------- | ----- |
| `link.exe` not found | Install Visual Studio Build Tools (C++ workload) |
| Frontend dist missing | Ensure `pnpm build` produces `apps/desktop/frontend/out` |
| `failed to run light.exe` / MSI fails | Enable Windows VBScript optional feature |
| Linux CI fails on WebKit | Confirm `libwebkit2gtk-4.1-dev` and related deps in `release.yml` |
| SQLITE / app data during smoke | Use a throwaway Windows user or clear app data dirs after install tests |
| SmartScreen warning | Expected for unsigned builds; signing is future work |
