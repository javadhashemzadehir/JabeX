# JabeX Product

## Vision

JabeX (جعبه ایکس) is a **private, offline-first digital Leitner Box**. It helps people memorize with spaced flashcards — nothing else.

## Positioning

| We are | We are not |
|--------|------------|
| A Leitner / SRS flashcard app | A notes app |
| Offline-first & privacy-first | A cloud SaaS |
| Persian-first, bilingual | English-only tooling |
| Simple and beautiful | A knowledge graph / wiki |

## Primary Jobs To Be Done

1. Create a deck quickly
2. Add flashcards (front / back)
3. Review due cards with Leitner box progression
4. Study fully offline, with data staying local
5. See learning consistency (streaks / calendar) from real review activity

## Learning consistency (not habits)

JabeX shows how steadily you review flashcards — current streak, best streak, and a year-long heatmap. This is **not** a habit tracker: there are no user-defined habits, goals, or “mark complete” buttons. Every signal comes from completed Leitner reviews on this device (see [ADR 013](../ADR/013-study-activity-consistency/013-study-activity-consistency.md)).

## Success Metrics (Product)

- First deck created in **≤ 30 seconds**
- First review completed in **≤ 2 minutes**
- Zero required account or network for core flows

## Languages & UX

- Default: **Persian (`fa`)**, RTL
- Secondary: **English (`en`)**, LTR
- Themes: Light, Dark, System

## Explicit Non-Goals

- Note-taking systems
- Knowledge graphs / backlinks
- Page builders / canvases
- Generic habit trackers / user-defined habits or goals
- AI tutoring or generation (unless user later opts into a clearly scoped, offline-friendly feature via ADR)
- Analytics / telemetry
- Mandatory authentication

## Open Source

JabeX is built with an open-source mindset: transparent architecture, reproducible development setup, and contributions guided by `CONTRIBUTING.md`.
