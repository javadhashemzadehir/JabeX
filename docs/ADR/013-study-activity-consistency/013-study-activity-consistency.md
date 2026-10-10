# ADR 013: Study Activity & Learning Consistency

## Status

Accepted

## Context

JabeX already records every Leitner grade in the append-only `reviews` table and shows basic local counts (`get_study_stats`). Users also benefit from a lightweight **consistency** signal — streaks and a year-long activity calendar — without turning the product into a habit tracker.

Constraints:

- Local-first, per-profile SQLite (no cloud, no accounts)
- Reviews remain the source of truth for what was studied
- No user-defined habits, goals, or manual “mark complete” actions

## Decision

### Not a habit tracker

Study activity measures only **Leitner flashcard review consistency**. There are no habit entities, schedules, or custom goals. UI copy frames this as learning consistency, not productivity tracking.

### Reviews as source of truth

Each successful `submit_review` emits an internal application event `ReviewCompletedEvent` (`profile_id`, `deck_id`, `cards_reviewed`, `duration_seconds`). `StudyActivityService` handles the event by upserting a daily `study_activity` row for the user’s **local calendar date**.

Daily rows are a **projection** for fast streaks/heatmap reads. Historical profiles with reviews but no activity rows are backfilled once from `reviews` before first read/write.

### Calculated streaks (not stored)

`current_streak`, `longest_streak`, and period totals are computed in the domain from `study_activity` dates. They are never persisted as counters, so they cannot drift from the daily log.

### Heatmap intensity

Buckets by cards reviewed that day: `0` none, `1–20` low, `21–99` medium, `100+` high.

### Schema

Migration **V8__study_activity.sql** adds `study_activity` (`id`, `profile_id`, `activity_date`, `cards_reviewed`, `minutes_spent`, `created_at`) with indexes on `profile_id`, `activity_date`, and a unique `(profile_id, activity_date)`.

`minutes_spent` stores accumulated review duration in **seconds** so short reviews sum accurately; IPC exposes whole minutes.

`.jbx` **v4** may include optional `data/study_activity.json` (older backups remain importable; missing file → empty, then backfill from reviews).

## Consequences

### Positive

- Motivation UI without product-scope creep into habits
- Offline, per-profile, no telemetry
- Streaks stay correct as long as daily rows match review days

### Trade-offs

- Daily projection can diverge if someone edits the DB by hand (acceptable; reviews can re-backfill only when the table is empty)
- Sub-minute precision is kept in SQLite seconds but JSON backup round-trips whole minutes

## Alternatives considered

| Alternative | Why not |
|-------------|---------|
| Generic habit entities | Out of product scope; confuses Leitner focus |
| Store streak counters | Drift risk; recalculation from days is cheap locally |
| Frontend-only session complete event | Mid-session exits would lose activity; per-review is reliable |
