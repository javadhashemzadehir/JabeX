# ADR 014: Advanced Card Editor (Spoilers, Math, Tables, Modes)

## Status

Accepted

## Context

[ADR 009](../009-tiptap-rich-text-documents/009-tiptap-rich-text-documents.md) established TipTap-compatible JSON as the source of truth for card faces (`document_v1` in `front_content` / `back_content`). Learners need a study-oriented editor closer to Notion + Telegram spoilers + Anki:

- Hide answers until revealed (Telegram-style spoilers)
- LaTeX mathematics
- Tables (including ChatGPT / Markdown paste)
- Code blocks with language highlighting
- Markdown ↔ JSON interoperability
- Edit / Preview / JSON / Markdown modes
- Safe clipboard handling without storing raw HTML

Constraints: offline-first, no HTML as storage, no new cloud deps, Persian RTL + LTR math/code, future `.jbx` / sync readiness.

## Decision

### Storage

- Keep **ADR 009 columns** (`front_content` / `back_content` TEXT). No separate `content_format` / `content_version` SQL columns.
- Bump document schema to **`version: 2`** (`document_v2`).
- Readers accept `version` 1–2 and legacy plain strings; writers emit version `2`.
- Optional export envelope (helpers + docs; `.jbx` already embeds face JSON):

  ```json
  {
    "format": "jabex-json",
    "version": 2,
    "content": { "version": 2, "type": "doc", "content": [] }
  }
  ```

### Editor stack

| Concern | Choice |
|---------|--------|
| Schema / WYSIWYG | TipTap 3 + ProseMirror JSON |
| Math | KaTeX (`trust: false`), custom `mathInline` / `mathDisplay` nodes |
| Spoilers | Custom mark `spoiler` + block `spoilerBlock` |
| Tables | `@tiptap/extension-table*` (insert, row/col, merge/split, resize) |
| Code | `CodeBlockLowlight` + `lowlight` |
| Markdown | Custom JSON ↔ MD walker (spoilers `\|\|…\|\|`, `:::spoiler`, `$` / `$$`) |
| Paste | Schema-bound HTML first; Markdown heuristic; JSON choice dialog |

UI modes on each face: **Edit | Preview | JSON | Markdown**. Media binaries stay in the attachment layer ([ADR 010](../010-rich-content-storage/010-rich-content-storage.md)) — not base64/HTML `src` inside TipTap JSON.

### Security

- Never persist pasted HTML as the source of truth.
- TipTap schema strips unknown tags/attributes on HTML paste.
- KaTeX renders with `trust: false`.
- Imported JSON validated (Zod / Rust version gate) before apply.

### Migration

| Path | Behavior |
|------|----------|
| Existing v1 rows | Readable as-is; next write upgrades `version` to 2 |
| New nodes in old clients | N/A for single desktop app; version gate rejects `> 2` |
| SQL | No ALTER required (same TEXT columns) |

## Consequences

### Positive

- Structured, portable faces for study, backup, and future Git sync
- Spoilers / math / tables improve Leitner answer UX without vendor lock-in
- Modes help power users (Markdown, raw JSON) without abandoning WYSIWYG

### Negative

- Larger IPC payloads and richer paste edge cases
- Custom spoiler/math nodes require shared extensions in edit + review
- Full TipTap Pro table UI is intentionally out of scope

## Alternatives Considered

- **HTML in SQLite**: rejected (ADR 009)
- **Markdown-only storage**: weaker Persian WYSIWYG; kept as a mode/export only
- **Separate SQL version columns**: redundant with JSON `version`; deferred
- **MathJax / CDN KaTeX**: heavier / network risk; local KaTeX chosen
