use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::entity::{SearchEntityType, SearchFilter};
use super::highlight::HighlightRange;

/// Denormalized index document written to `search_index`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexDocument {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub entity_type: SearchEntityType,
    pub entity_id: String,
    pub title: String,
    pub content: String,
    pub normalized_content: String,
    pub metadata: Value,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One hit returned to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entity_type: SearchEntityType,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub breadcrumb: Option<String>,
    pub highlights: Vec<HighlightRange>,
    pub score: f64,
    pub href: Option<String>,
    pub metadata: Value,
    pub text_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub filter: SearchFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentSearch {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub query: String,
    pub filters: Vec<SearchFilter>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryEntry {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub query: String,
    pub entity_type: Option<SearchEntityType>,
    pub entity_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Aggregated local query frequency (from `search_history`, never uploaded).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopularSearch {
    pub query: String,
    pub hit_count: u64,
    pub last_searched_at: DateTime<Utc>,
}

/// Default caps (local-only; not telemetry).
pub const RECENT_SEARCHES_MAX: usize = 20;
pub const SEARCH_HISTORY_MAX: usize = 200;
pub const POPULAR_SEARCHES_MAX: usize = 10;
pub const DEFAULT_SEARCH_LIMIT: usize = 40;
