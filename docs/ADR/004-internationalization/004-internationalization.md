# ADR 004: Internationalization

## Status

Accepted

## Context

JabeX targets Persian users first while remaining accessible to English speakers. Layout direction, typography, and copy must switch cleanly without duplicated UI trees.

## Decision

Implement first-class bilingual support:

| Locale | Direction | Role |
|--------|-----------|------|
| `fa` | RTL | Default |
| `en` | LTR | Secondary |

Implementation approach:

- Translation catalogs in `locales/fa` and `locales/en`
- Dynamic `lang` and `dir` on the document root
- All user-facing strings go through the i18n layer
- Locale preference persisted locally (no account)

## Consequences

### Positive

- Persian-first UX without blocking English adoption
- Single component tree for both directions
- Clear contribution path for new locales later

### Negative

- Every UI string needs catalog entries
- Some CSS/layout bugs appear only in one direction and must be dual-tested

## Alternatives Considered

- **Persian only**: excludes English contributors/users
- **English default**: fights product positioning
- **Separate RTL/LTR apps**: unnecessary maintenance cost
