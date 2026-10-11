# Repository Metadata Cleanup

> Internal maintenance record. Documents the one-time history rewrite performed to clean repository metadata.

## Summary

A repository-wide metadata cleanup was performed to strip third-party tooling attribution from commit history and remove obsolete local tooling configuration from the tree. No source code, package versions, or product behavior were changed.

## Why

The repository history contained:

- `Co-authored-by` trailers referencing an external code-generation tool on the majority of commits.
- A tracked `.cursor/rules/` directory holding local AI-agent rule files.
- Documentation references to that local tooling configuration.

These were removed so the project history reflects only the project's own contributors and configuration.

## What changed

### Working tree

- Deleted the tracked `.cursor/rules/` directory (10 rule files).
- Removed references to the local tooling configuration from:
  - `README.md`
  - `docs/ARCHITECTURE/ARCHITECTURE.md`
  - `docs/DEVELOPMENT/DEVELOPMENT.md`
  - `docs/ROADMAP/ROADMAP.md`
- Added an ignore block to `.gitignore` for `.cursor/`, `.cursorrules`, and `.cursorignore` so the tooling configuration cannot be re-introduced.

### Git history

- Rewrote all commits to strip `Co-authored-by` trailers that referenced the external tool.
- Removed one body sentence from a single commit that referenced the tooling configuration.
- Removed the `.cursor/` path from every commit in history (not just the working tree).
- Commit subjects, trees (apart from `.cursor/` removal), authors, and tags were preserved.
- Commit count is unchanged (38 commits on `main`); only commit hashes changed because messages and trees changed.

## How (commands used)

Tool: [`git-filter-repo`](https://github.com/newren/git-filter-repo) (run as a standalone script).

```bash
# 1. Back up the repository first (see Backup procedure below)
git clone --mirror . ../jabex-backup-<timestamp>.git

# 2. Stash in-progress work so the working tree is clean
git stash push -u -m "jabex-pre-cleanup-wip"

# 3. Rewrite history: strip trailers/body line and remove .cursor/ from all commits
#    cursor-replace.txt contains regex rules, each prefixed with `regex:`,
#    e.g.  regex:(?im)^co-authored-by:[^\n]*cursor[^\n]*\n==>
python git-filter-repo --force \
  --replace-message cursor-replace.txt \
  --path .cursor --invert-paths

# 4. Re-add the origin remote (git-filter-repo removes it as a safety measure)
git remote add origin <remote-url>

# 5. Restore in-progress work
git stash pop
```

## Backup procedure

Before the rewrite, a full mirror backup was created and kept on disk:

- `D:\Projects\JabeX\jabex-backup-<timestamp>.git` — bare mirror with all refs (branches, tags, stash at time of backup).
- A binary patch of in-progress work was also saved at `D:\Projects\JabeX\wip-backup.patch`.

Do not delete the mirror backup until the force-push has been verified on the remote and at least one collaborator has confirmed a clean clone.

## Verification

The following checks were run after the rewrite; all returned no results:

```bash
git log --all --grep="cursor" -i                 # no commit message matches
git log --all --format='%an <%ae>' | grep -i cursor   # no author matches
git log --all --format='%(trailers)' | grep -i cursor # no trailer matches
git ls-files | grep -i cursor                    # no tracked filename matches
git log --all --oneline -- .cursor                # no history for the removed path
```

Remaining occurrences of the substring "cursor" in the tree are intentional and unrelated to any tooling: CSS `cursor` utilities, a date-loop variable named `cursor` in `study_statistics.rs`, and the `@tiptap/extension-dropcursor` / `gapcursor` dependencies in `pnpm-lock.yaml`.

## Remaining actions

History rewrite changes every commit hash, so the published remote must be force-updated:

```bash
git push --force --all origin
git push --force --tags origin
```

Before force-pushing:

1. Confirm the mirror backup is intact and restorable.
2. Notify anyone with a clone or fork — they will need to re-clone or reset their local branches.
3. After force-pushing, verify the remote contributor graph and tag list.

> Note: force-pushing to `main` is normally prohibited by this project's Git Workflow. This cleanup is an explicit, one-time exception.
