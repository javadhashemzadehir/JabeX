# ADR 002: UI Framework Choice (shadcn/ui)

## Status

Accepted (supersedes FarsiUI as primary kit)

## Context

JabeX needs accessible, maintainable React components with full code ownership, Tailwind theming, light/dark/system support, and first-class Persian RTL + English LTR. We previously considered FarsiUI; the product decision is now to standardize on the official shadcn/ui ecosystem.

## Decision

Adopt **[shadcn/ui](https://ui.shadcn.com/docs/components)** as the primary component/design system.

- Style: **New York**
- Components are copied into `apps/desktop/frontend/src/components/ui`
- Install via `npx shadcn@latest add <component>` (or registry JSON when CLI network fails)
- RTL via document `dir` + `@radix-ui/react-direction` `DirectionProvider`
- Themes via CSS variables + `next-themes`

### Allowed secondary sources (shadcn-compatible only)

Use these for variants/blocks **only** when they install as owned source compatible with our `components.json` — never introduce a competing design system:

- [shadcn/ui components](https://ui.shadcn.com/docs/components)
- [Shadcn Studio](https://shadcnstudio.com/components)
- [Shadcnblocks](https://www.shadcnblocks.com/)
- [ReUI](https://reui.io/components)

Supporting stack:

- Tailwind CSS v4 + CSS variables + `tw-animate-css`
- Prefer Tailwind v4 canonical / logical utilities for RTL/LTR (`inset-s-*` / `inset-e-*`, `ms-*` / `me-*`, `border-s-*` / `border-e-*`, `text-start` / `text-end`, `bg-linear-to-*`); avoid deprecated `start-*` / `end-*` inset aliases
- `lucide-react` for icons
- `framer-motion` for purposeful motion
- `sonner` for toasts

## Consequences

### Positive

- Official, widely documented primitives
- Full ownership under `components/ui`
- Large ecosystem of compatible blocks without lock-in
- Strong accessibility via Radix patterns

### Negative

- RTL must be validated explicitly (not “Persian-only” by default)
- Components must be maintained in-repo

## Alternatives Considered

- **FarsiUI**: Persian/RTL-oriented, but not the chosen primary kit going forward
- **MUI / Chakra**: heavier abstractions, weaker fit for owned design tokens
- **Fully custom components**: too slow for foundation phase
