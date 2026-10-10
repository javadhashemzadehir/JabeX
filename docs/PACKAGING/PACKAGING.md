# Packaging

How to build Windows installers for JabeX and how CI publishes GitHub Releases.

Related:

- [DEVELOPMENT.md](../DEVELOPMENT/DEVELOPMENT.md) — local setup
- [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md) — pre-tag verification
- [GIT_WORKFLOW.md](../GIT_WORKFLOW/GIT_WORKFLOW.md) — tagging and release process
- [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — multi-OS release job

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

Smoke-check on a clean Windows user/profile when possible. **Unsigned** builds may trigger SmartScreen (“Windows protected your PC”) — expected until Authenticode signing is added.

## CI release (tag-triggered)

Pushing an annotated tag matching `v*` (for example `v0.1.0` or `v0.1.0-beta.1`) runs `release.yml`:

| Matrix job | Runner | Bundle args |
| ---------- | ------ | ----------- |
| windows | `windows-latest` | `--bundles nsis,msi` → `.exe` + `.msi` |
| macos-dmg | `macos-latest` | universal Darwin + `--bundles dmg` |
| linux-appimage | `ubuntu-22.04` | `--bundles appimage` |

The workflow publishes a GitHub Release with the installers attached. Tags containing `-` are marked prerelease.

Version sync before tagging: root `package.json`, `apps/desktop/package.json`, `apps/desktop/frontend/package.json`, `apps/desktop/src-tauri/Cargo.toml`, `tauri.conf.json`, and `CHANGELOG.md` (see GIT_WORKFLOW and [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md)).

## Code signing (deferred)

Phase 3 ships **unsigned** installers first so the pipeline can be proven end-to-end.

Not yet configured:

- Windows Authenticode / SmartScreen reputation
- Apple Developer ID signing + notarization
- Tauri updater signing keys

When added, secrets belong in GitHub Actions (never in the repo). Prefer documenting the exact secret names and notarization steps in this file when signing lands.

## Privacy / architecture (do not break)

Release packaging must keep JabeX:

- Local-first (SQLite + media on device)
- No required accounts or cloud sync
- No telemetry
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
