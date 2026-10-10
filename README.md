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
| Frontend | Next.js (App Router) + TypeScript + Tailwind |
| UI system | [shadcn/ui](https://ui.shadcn.com/docs/components) |
| Desktop | Tauri 2 + Rust |
| i18n | `fa` (default, RTL) · `en` (LTR) |
| Themes | Light · Dark · System |

## Monorepo

```
apps/desktop/frontend   # Next.js UI
apps/desktop/src-tauri  # Tauri / Rust host
packages/ui             # Shared UI (placeholder)
packages/types          # Shared types
packages/config         # Shared tooling config
docs/                   # Product & architecture docs
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

- [Product](./docs/PRODUCT.md)
- [Architecture](./docs/ARCHITECTURE.md)
- [Database](./docs/DATABASE.md)
- [Development](./docs/DEVELOPMENT.md)
- [Git workflow](./docs/GIT_WORKFLOW.md)
- [Roadmap](./docs/ROADMAP.md)
- [Contributing](./CONTRIBUTING.md)
- [Security](./SECURITY.md)
- [Changelog](./CHANGELOG.md)

## License

[MIT](./LICENSE)
