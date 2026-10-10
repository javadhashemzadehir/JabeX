# ADR 009: TipTap JSON Documents for Card Faces

## Status

Accepted (extended by [ADR 014](../014-advanced-card-editor/014-advanced-card-editor.md) for `document_v2` nodes and editor modes)

## Context

Card front/back were plain strings. Product direction requires layered card content (rich text first, then media and ink) without HTML-only storage or cloud editors.

## Decision

- Store each card face as a **versioned TipTap-compatible JSON document**:

  ```json
  { "version": 1, "type": "doc", "content": [/* TipTap nodes */] }
  ```

- Persist as TEXT in existing `cards.front_content` / `cards.back_content` columns (no destructive schema change).
- Domain type: `RichTextDocument` (Rust + `packages/types`).
- Frontend editor: TipTap + StarterKit, styled with shadcn/design tokens.
- Display during review via TipTap in read-only mode (no `dangerouslySetInnerHTML` of arbitrary HTML).
- Legacy plain-text rows and older `.jbx` string faces are accepted on read/import and normalized.

Current write version is **`2`** (spoilers, math, tables, highlighted code). See ADR 014. Readers still accept `version: 1`.

## Consequences

### Positive

- Aligns with future media/ink layers and document sync (`revision` already present)
- Schema-versioned documents (`document_v1` → `document_v2` → later migrations)
- XSS surface stays bounded to a fixed extension set

### Negative

- IPC payload size grows vs plain strings
- List UIs should prefer plain-text previews (not one editor per row)

## Alternatives Considered

- **HTML in SQLite**: rejected — harder to migrate, weaker XSS story
- **Separate content table**: deferred until media/ink need it
- **Markdown only**: weaker WYSIWYG UX for Persian-first flashcards
