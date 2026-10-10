//! Study activity derived from Leitner reviews — not a habit tracker.
//!
//! Daily rows are a projection of completed reviews for consistency (streaks,
//! heatmap). Streaks and aggregates are calculated in memory; they are never
//! persisted as separate counters.

mod activity;
mod study_statistics;
mod study_streak;

pub use activity::*;
pub use study_statistics::*;
pub use study_streak::*;
