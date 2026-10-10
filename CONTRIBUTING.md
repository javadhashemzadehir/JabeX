# Contributing to JabeX

Thanks for helping build a private, offline-first Leitner Box.

Full Git / branch / release details: **[docs/GIT_WORKFLOW/GIT_WORKFLOW.md](./docs/GIT_WORKFLOW/GIT_WORKFLOW.md)**

## Principles

- Flashcards only — no notes, graphs, or page builders
- Privacy-first — no analytics/telemetry
- Offline-first — core flows must work without a network
- Persian-first UX with English support

## Getting started

1. Browse the docs index: [`docs/README.md`](./docs/README.md)
2. Read `docs/PRODUCT/PRODUCT.md`, `docs/ARCHITECTURE/ARCHITECTURE.md`, and ADRs
3. Follow `docs/DEVELOPMENT/DEVELOPMENT.md` for local setup (including OS-specific Tauri prereqs)
4. Read `docs/GIT_WORKFLOW/GIT_WORKFLOW.md` for branching and commits
5. Prefer small pull requests with a clear purpose
6. Follow the [Code of Conduct](./CODE_OF_CONDUCT.md)

## First contribution

A good first path:

1. Branch from `develop` (`feature/<short-name>` or `bugfix/<short-name>`)
2. Make a focused change (docs typo, small UI polish, or a labeled good-first-issue)
3. Run `pnpm lint && pnpm typecheck && pnpm test` (and Rust checks if you touched `src-tauri`)
4. Open a PR into **`develop`** with a Conventional Commit title
5. Wait for CI (frontend + Rust) to pass

Where to change code:

| Goal | Start in |
| ---- | -------- |
| UI / i18n | `apps/desktop/frontend/src/` |
| IPC / persistence / scheduler | `apps/desktop/src-tauri/src/` |
| Shared contracts | `packages/types/` |
| Docs / ADRs | `docs/` |

## Contributor workflow

1. **Fork** the repository (external contributors)
2. **Clone** your fork and install dependencies:

   ```bash
   git clone https://github.com/<you>/jabex.git
   cd jabex
   pnpm install
   cp .env.example .env
   ```

3. **Create a branch from `develop`:**

   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/my-change
   ```

   Use `bugfix/<name>` for non-urgent fixes. Hotfixes branch from `main` — see the workflow guide.

4. **Commit using Conventional Commits:**

   ```text
   type(scope): message
   ```

   Examples:

   ```text
   feat(cards): add card creation flow
   fix(database): repair migration rollback
   docs(readme): update installation guide
   ```

   Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`

5. **Open a Pull Request** into **`develop`** (not `main`)
6. **Pass CI** — frontend lint/typecheck/build and Rust fmt/clippy/test
7. Address review feedback; a maintainer **merges** when ready

### Development commands

```bash
pnpm install
pnpm dev                 # frontend only (browser)
pnpm tauri:dev           # full desktop app (requires Rust)
pnpm lint
pnpm typecheck
pnpm test
pnpm build
pnpm db:seed             # optional local mock study data
pnpm db:unseed
```

Rust:

```bash
cd apps/desktop/src-tauri
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Testing overview: [docs/TESTING/TESTING.md](./docs/TESTING/TESTING.md).
Packaging / installers: [docs/PACKAGING/PACKAGING.md](./docs/PACKAGING/PACKAGING.md).

## Code guidelines

- TypeScript strict mode
- shadcn/ui for UI primitives (`components/ui`); add via `npx shadcn@latest add`
- User-facing strings belong in `locales/{fa,en}`
- Business logic stays out of presentational components
- Rust domain logic for persistence/scheduling

## Pull requests

- Title uses Conventional Commit format
- Fill out `.github/PULL_REQUEST_TEMPLATE.md`
- Update docs/ADRs when changing architecture
- Ensure CI passes
- Target `develop` for features and bugfixes

## Commit signing (recommended)

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

Register the public key as a signing key on GitHub.

## Out of scope contributions

Please open an issue first before proposing:

- Cloud sync / auth
- AI features
- Analytics
- Non-flashcard product surfaces

## Security

Report vulnerabilities privately — see [SECURITY.md](./SECURITY.md).
