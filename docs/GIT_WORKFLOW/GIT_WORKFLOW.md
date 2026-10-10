# Git Workflow

JabeX uses an improved **Git Flow** model suitable for a solo maintainer today and open-source contributors later.

Related files:

- [CONTRIBUTING.md](../../CONTRIBUTING.md) — how to contribute
- [CHANGELOG.md](../../CHANGELOG.md) — Keep a Changelog
- [SECURITY.md](../../SECURITY.md) — vulnerability reporting
- [PACKAGING.md](../PACKAGING/PACKAGING.md) — local installers and release artifacts
- `.github/workflows/` — CI, release, security, stale automation

---

## Branch strategy

```
feature/* ──┐
bugfix/*  ──┼──► develop ──► release/* ──► main (tags: vX.Y.Z)
hotfix/*  ──┘         ▲                      │
                      └──────────────────────┘
                         (hotfix also merges back)
```

| Branch | Purpose | Lifetime |
| ------ | ------- | -------- |
| `main` | Production-ready releases only | Permanent |
| `develop` | Integration branch for the next release | Permanent |
| `feature/*` | New functionality | Short-lived |
| `bugfix/*` | Non-urgent fixes against `develop` | Short-lived |
| `release/*` | Release hardening, version bumps, changelog | Short-lived |
| `hotfix/*` | Emergency fixes for production | Short-lived |

### Naming rules

| Type | Pattern | Examples |
| ---- | ------- | -------- |
| Feature | `feature/<short-kebab-name>` | `feature/card-editor`, `feature/theme-system`, `feature/rust-scheduler` |
| Bugfix | `bugfix/<short-kebab-name>` | `bugfix/database-migration-error` |
| Hotfix | `hotfix/<short-kebab-name>` | `hotfix/crash-on-empty-deck` |
| Release | `release/v<MAJOR.MINOR.PATCH>` | `release/v0.1.0`, `release/v0.2.1` |

Rules:

- Use lowercase kebab-case after the prefix
- Keep names short and descriptive (≤ 4 words preferred)
- Do not use vague names (`feature/update`, `bugfix/fix`)

---

## Branch protection & merge rules

### `main`

- Protected branch
- **No direct commits**
- Merges only from `release/*` (planned releases) or `hotfix/*` (emergencies)
- Requires green CI
- Prefer squash or merge commits as configured by maintainers; never force-push

### `develop`

- Protected branch
- **No direct commits**
- Accepts Pull Requests from `feature/*`, `bugfix/*`, `hotfix/*`, and completed `release/*` back-merges
- Requires green CI
- Code review recommended (required once multiple maintainers exist)

### `feature/*`

| | |
| --- | --- |
| Created from | `develop` |
| Merged into | `develop` via PR |
| Deleted after merge | Yes |

### `bugfix/*`

| | |
| --- | --- |
| Created from | `develop` |
| Merged into | `develop` via PR |
| Deleted after merge | Yes |

### `hotfix/*`

| | |
| --- | --- |
| Created from | `main` |
| Merged into | `main` **and** `develop` (or the active `release/*` if one exists) |
| Tag after `main` merge | Patch bump (`v0.2.1`) |
| Deleted after merge | Yes |

### `release/*`

| | |
| --- | --- |
| Created from | `develop` |
| Merged into | `main`, then back into `develop` |
| Allowed commits | Version bumps, changelog, last-mile fixes, docs |
| Tag after `main` merge | Matching SemVer tag (`v0.1.0`) |

---

## Conventional Commits

Format:

```text
type(scope): message
```

### Allowed types

| Type | When to use |
| ---- | ----------- |
| `feat` | New user-facing feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting / whitespace (no logic change) |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or updating tests |
| `chore` | Maintenance (deps tooling that isn't `build`/`ci`) |
| `perf` | Performance improvement |
| `build` | Build system or dependencies affecting the build |
| `ci` | CI/CD configuration |

### Scopes (suggested)

`cards`, `decks`, `review`, `database`, `scheduler`, `ui`, `i18n`, `tauri`, `desktop`, `docs`, `ci`, `release`

### Examples

```text
feat(cards): add card creation flow
fix(database): repair migration rollback
docs(readme): update installation guide
ci(release): attach AppImage artifacts
chore(deps): bump tauri to 2.x
```

### Rules

- Imperative mood (“add”, not “added”)
- Subject ≤ ~72 characters; explain *why* in the body when non-obvious
- One logical change per commit when practical
- Never commit secrets, `.env`, or local `*.db` / user data files
- Breaking changes: add `!` after type/scope and a `BREAKING CHANGE:` footer

```text
feat(api)!: rename deck IPC command

BREAKING CHANGE: `create_deck` is now `deck_create`.
```

---

## Versioning (SemVer)

Format: `MAJOR.MINOR.PATCH`

| Version | Meaning |
| ------- | ------- |
| `0.1.0` | Initial beta |
| `0.2.0` | New features (minor) |
| `0.2.1` | Bug fixes (patch) |
| `1.0.0` | First stable release |

Pre-releases (optional): `v0.3.0-beta.1`, `v1.0.0-rc.1`

Keep these in sync when cutting a release:

- root `package.json` `version`
- `apps/desktop/frontend/package.json` `version`
- `apps/desktop/src-tauri/Cargo.toml` `version`
- `apps/desktop/src-tauri/tauri.conf.json` `version`
- `CHANGELOG.md`

---

## Release process

### Planned release

1. Ensure `develop` is green and feature-complete for the milestone
2. Create `release/vX.Y.Z` from `develop`
3. On the release branch:
   - Bump versions
   - Update `CHANGELOG.md` (move Unreleased → version section)
   - Fix only release blockers
4. Open PR: `release/vX.Y.Z` → `main`
5. After merge to `main`, tag:

   ```bash
   git checkout main
   git pull
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

6. Merge `main` (or the release branch) back into `develop`
7. GitHub Actions `release.yml` builds installers and publishes a GitHub Release for tag `v*`

### Hotfix release

1. Branch `hotfix/<name>` from `main`
2. Fix, test, PR into `main`
3. Tag patch version on `main` (`v0.2.1`)
4. Merge the hotfix into `develop` immediately
5. Release workflow runs on the new tag

### Artifacts

| Platform | Primary artifact |
| -------- | ---------------- |
| Windows | `.msi` |
| macOS | `.dmg` |
| Linux | `.AppImage` |

Local smoke builds, artifact paths, and signing status: [PACKAGING.md](../PACKAGING/PACKAGING.md).

---

## Contributor workflow

1. Fork the repository (external contributors)
2. Clone and install (`pnpm install`)
3. Checkout `develop` and create a branch:

   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/my-change
   ```

4. Commit with Conventional Commits
5. Push and open a Pull Request **into `develop`** (not `main`)
6. Fill out the PR template; ensure CI is green
7. Address review feedback
8. Maintainer merges; delete the feature branch

Solo maintainers follow the same branch rules even without a fork.

---

## Pull request rules

- **Title** uses Conventional Commit format (`feat(cards): …`)
- Target `develop` for features/bugfixes; target `main` only for `release/*` or `hotfix/*`
- Keep PRs small and reviewable
- Link ADRs when architecture changes
- Do not force-push to `main` or `develop`

---

## Commit signing (recommended)

Signed commits improve trust for open-source and release tags.

```bash
# SSH signing (recommended on GitHub)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

Add the same public key as a **Signing key** in GitHub → Settings → SSH and GPG keys.

---

## Initial repository setup

If starting fresh (already initialized locally):

```bash
git init
git branch -M main

# After the first commit on main:
git checkout -b develop
git push -u origin main
git push -u origin develop
```

Configure default branch to `develop` for everyday PRs; keep `main` as the production branch. Protect both on GitHub:

- Require PR
- Require status checks (`CI`)
- Restrict who can push
- For `main`: optionally restrict merges to release/hotfix patterns via rulesets

---

## Automation overview

| Workflow | Trigger | Purpose |
| -------- | ------- | ------- |
| `ci.yml` | push / pull_request | Frontend lint, typecheck, build; Rust fmt, clippy, test |
| `release.yml` | tag `v*` | Build Tauri installers and publish GitHub Release |
| `security.yml` | schedule / push / PR | Dependency and secret scanning hooks |
| `stale.yml` | schedule | Mark/close inactive issues and PRs |

Also configured:

- Dependabot (`.github/dependabot.yml`)
- Labels (`.github/labels.yml`)
- Auto release-note categories (`.github/release.yml`)
- `CODEOWNERS` for review routing
