use chrono::{DateTime, Utc};

/// Current UTC timestamp.
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}
