# Git Commit Policy

## Principle

Automated assistant tools may be used locally while developing JabeX.

Commit history must contain **only human / project contributors**.

AI assistant attribution trailers are **prohibited** in this repository.

## Prohibited in commit messages

Do not include assistant attribution such as:

- `Co-authored-by` trailers that name an AI coding assistant
- Assistant product emails or agent identities in commit metadata
- Body text that attributes authorship of the commit to an AI assistant

Legitimate human `Co-authored-by` trailers (real people collaborating on a change) remain allowed and must be preserved.

## Enforcement

This repository uses a version-controlled `commit-msg` hook:

| Path | Role |
| ---- | ---- |
| `.githooks/commit-msg` | Canonical hook (tracked in Git) |
| `core.hooksPath` | Set to `.githooks` for this repository |

The hook runs before a commit is finalized and **automatically removes** AI assistant attribution trailers from the commit message. It does not block normal commits and does not modify source files.

### One-time setup (new clones)

After cloning, ensure the repository uses the tracked hooks directory:

```bash
git config core.hooksPath .githooks
```

On Unix-like systems, ensure the hook is executable if your clone does not preserve the mode:

```bash
chmod +x .githooks/commit-msg
```

## Related

- [Git Workflow](../GIT_WORKFLOW/GIT_WORKFLOW.md) — branch model, Conventional Commits, PR rules
- [Repository Metadata Cleanup](./GIT_HISTORY_CLEANUP.md) — one-time history metadata cleanup record
