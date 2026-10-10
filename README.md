# JabeX (جعبه ایکس)

**Private, offline-first digital Leitner Box.**

Flashcards only. No notes. No cloud account. Persian-first.

## Goals

- Create your first deck in **≤ 30 seconds**
- Complete your first review in **≤ 2 minutes**
- Keep all study data on your device

## Stack

| Layer | Technology |
|-------|------------|
| Frontend | Next.js (App Router) + TypeScript + Tailwind CSS v4 |
| UI system | [shadcn/ui](https://ui.shadcn.com/docs/components) |
| Desktop | Tauri 2 + Rust |
| i18n | `fa` (default, RTL) · `en` (LTR) |
| Themes | Light · Dark · System |
| Card faces | TipTap JSON (`document_v2`) — spoilers, math, tables, Markdown modes |

## Monorepo

```
apps/desktop/frontend   # Next.js UI
apps/desktop/src-tauri  # Tauri / Rust host
packages/types          # Shared TypeScript contracts
docs/                   # Product & architecture docs (one topic per folder)
.cursor/rules/          # Cursor AI agent rules
```

## Quick start

```bash
pnpm install
cp .env.example .env
pnpm dev
```

Open [http://localhost:3000](http://localhost:3000).

### Desktop (requires Rust)

```bash
# Install Rust: https://rustup.rs/
pnpm tauri:dev
```

## Quality checks

```bash
pnpm lint
pnpm typecheck
pnpm test
```

## Documentation

- **[Docs index](./docs/README.md)** — start here
- [Product](./docs/PRODUCT/PRODUCT.md)
- [Architecture](./docs/ARCHITECTURE/ARCHITECTURE.md)
- [Database](./docs/DATABASE/DATABASE.md)
- [Development](./docs/DEVELOPMENT/DEVELOPMENT.md)
- [Packaging](./docs/PACKAGING/PACKAGING.md)
- [Testing](./docs/TESTING/TESTING.md)
- [Git workflow](./docs/GIT_WORKFLOW/GIT_WORKFLOW.md)
- [Roadmap](./docs/ROADMAP/ROADMAP.md)
- [ADR 014 — Advanced card editor](./docs/ADR/014-advanced-card-editor/014-advanced-card-editor.md)
- [Contributing](./CONTRIBUTING.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Security](./SECURITY.md)
- [Changelog](./CHANGELOG.md)

## License

[MIT](./LICENSE)
