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

Install the repository `commit-msg` hook into your local Git hooks directory. The hook runs before a commit is finalized and **automatically removes** AI assistant attribution trailers. It does not block normal commits and does not modify source files.

| Path | Role |
| ---- | ---- |
| `scripts/git/commit-msg` | Canonical hook template (tracked) |
| `.git/hooks/commit-msg` | Active local hook (not tracked) |
| `.githooks/` | Ignored local override directory — do not commit |

Do **not** set `core.hooksPath` to `.githooks` in this repository. Use the default Git hooks path (`.git/hooks`).

### One-time setup (new clones)

```bash
cp scripts/git/commit-msg .git/hooks/commit-msg
chmod +x .git/hooks/commit-msg
```

On Windows (Git Bash):

```bash
cp scripts/git/commit-msg .git/hooks/commit-msg
```

## Related

- [Git Workflow](../GIT_WORKFLOW/GIT_WORKFLOW.md) — branch model, Conventional Commits, PR rules
- [Repository Metadata Cleanup](./GIT_HISTORY_CLEANUP.md) — one-time history metadata cleanup record
