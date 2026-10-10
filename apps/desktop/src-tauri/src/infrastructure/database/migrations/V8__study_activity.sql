-- Daily Leitner study activity (projection of reviews for streaks / heatmap).
-- Not a habit tracker: rows are written only when cards are reviewed.
-- Streaks are calculated in application/domain code and are never stored here.

CREATE TABLE IF NOT EXISTS study_activity (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    activity_date TEXT NOT NULL,
    cards_reviewed INTEGER NOT NULL DEFAULT 0,
    -- Accumulated review duration in whole seconds (API exposes minutes).
    minutes_spent INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_study_activity_profile_id
    ON study_activity(profile_id);

CREATE INDEX IF NOT EXISTS idx_study_activity_activity_date
    ON study_activity(activity_date);

CREATE UNIQUE INDEX IF NOT EXISTS idx_study_activity_profile_date
    ON study_activity(profile_id, activity_date);
