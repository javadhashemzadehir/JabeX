# Packaging

How to build installers for JabeX and how CI publishes them.

Related:

- [DEVELOPMENT.md](../DEVELOPMENT/DEVELOPMENT.md) — local setup
- [GIT_WORKFLOW.md](../GIT_WORKFLOW/GIT_WORKFLOW.md) — tagging and release process
- [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — multi-OS release job

## Bundle configuration

Source of truth: [`apps/desktop/src-tauri/tauri.conf.json`](../../apps/desktop/src-tauri/tauri.conf.json).

| Field | Purpose |
| ----- | ------- |
| `productName` | Installer / app display name (`JabeX`) |
| `identifier` | Bundle ID (`app.jabex.desktop`) |
| `version` | SemVer; keep in sync with Cargo/`package.json` |
| `bundle.category` | Store / OS category (`Education`) |
| `bundle.shortDescription` / `longDescription` | Installer metadata |
| `bundle.copyright` | Copyright line |
| `bundle.targets` | `"all"` locally; CI passes `--bundles` per OS |
| `bundle.icon` | PNG / ICNS / ICO under `src-tauri/icons/` |

Frontend production assets come from Next static export (`frontendDist`: `../frontend/out`).

## Local build (smoke test)

Prerequisites: Node 22+, pnpm 9+, Rust stable, and [Tauri platform deps](https://v2.tauri.app/start/prerequisites/). On Windows, MSVC Build Tools (`link.exe`) and WebView2 are required.

From the repo root:

```bash
pnpm install
pnpm tauri:build
```

This runs the frontend production build, then Tauri release bundling.

### Artifact locations

Under `apps/desktop/src-tauri/target/release/bundle/`:

| Platform | Typical path |
| -------- | ------------ |
| Windows | `msi/JabeX_*.msi` (also `nsis/` if NSIS is produced) |
| macOS | `dmg/*.dmg` |
| Linux | `appimage/*.AppImage` |

Smoke-check: install or run the artifact on a clean machine/profile when possible. Unsigned builds may trigger OS warnings (SmartScreen / Gatekeeper).

## CI release (tag-triggered)

Pushing an annotated tag matching `v*` (for example `v0.1.0` or `v0.1.0-beta.1`) runs `release.yml`:

| Matrix job | Runner | Bundle args |
| ---------- | ------ | ----------- |
| windows-msi | `windows-latest` | `--bundles msi` |
| macos-dmg | `macos-latest` | universal Darwin + `--bundles dmg` |
| linux-appimage | `ubuntu-22.04` | `--bundles appimage` |

The workflow publishes a GitHub Release with the installers attached. Tags containing `-` are marked prerelease.

Version sync before tagging: root `package.json`, `apps/desktop/frontend/package.json`, `apps/desktop/src-tauri/Cargo.toml`, `tauri.conf.json`, and `CHANGELOG.md` (see GIT_WORKFLOW).

## Code signing (deferred)

Phase 3 ships **unsigned** installers first so the pipeline can be proven end-to-end.

Not yet configured:

- Windows Authenticode / SmartScreen reputation
- Apple Developer ID signing + notarization
- Tauri updater signing keys

When added, secrets belong in GitHub Actions (never in the repo). Prefer documenting the exact secret names and notarization steps in this file when signing lands.

## Troubleshooting

| Symptom | Check |
| ------- | ----- |
| `link.exe` not found | Install Visual Studio Build Tools (C++ workload) |
| Frontend dist missing | Ensure `pnpm build` produces `apps/desktop/frontend/out` |
| Linux CI fails on WebKit | Confirm `libwebkit2gtk-4.1-dev` and related deps in `release.yml` |
| SQLITE / app data during smoke | Use a throwaway Windows user or clear app data dirs after install tests |
