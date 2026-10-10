# Testing

How we verify JabeX without shipping telemetry or cloud test harnesses.

## What to run locally

From the repo root:

```bash
pnpm lint
pnpm typecheck
pnpm test
```

Rust (from `apps/desktop/src-tauri`):

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Desktop smoke (after Rust toolchain + platform deps):

```bash
pnpm tauri:dev
```

Installer smoke (Windows example):

```bash
pnpm tauri:build
```

See [PACKAGING.md](../PACKAGING/PACKAGING.md).

## Frontend

| Area | Tooling | Location |
| ---- | ------- | -------- |
| Unit / component | Vitest + Testing Library | `apps/desktop/frontend` (`*.test.ts(x)`) |
| Types | `tsc --noEmit` | `pnpm typecheck` |
| Lint | ESLint | `pnpm lint` |

Prefer tests for pure helpers (rich text, shortcuts, search highlight) and critical UI flows. Avoid snapshot-heavy suites.

## Rust

| Layer | Prefer |
| ----- | ------ |
| Domain | Pure unit tests (scheduler, rich text, search normalize/rank) |
| Application | Use-case tests with SQLite temp dirs where needed |
| Infrastructure | Repository tests against `open_database_at(tempdir)` |

Domain must stay free of Tauri/SQLite imports — keep tests aligned with that boundary.

## CI

GitHub Actions (`.github/workflows/ci.yml`) runs frontend quality checks and Rust fmt/clippy/test on pull requests. Release installers run only on `v*` tags (`.github/workflows/release.yml`).

## Manual QA focus

- First-run profile naming → first deck → first review (≤ 2 minutes path)
- Soft delete / Trash restore
- `.jbx` export/import on a throwaway profile
- Large deck: open cards list and review without multi-second freezes
