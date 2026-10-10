//! Extensible local-first global search (SQLite FTS5 + ranking).

mod entity;
mod extract;
mod fuzzy;
mod highlight;
mod normalize;
mod rank;
mod types;

pub use entity::*;
pub use extract::*;
pub use fuzzy::*;
pub use highlight::*;
pub use normalize::*;
pub use rank::*;
pub use types::*;
