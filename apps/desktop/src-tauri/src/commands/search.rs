use tauri::State;

use crate::domain::errors::JabeXError;
use uuid::Uuid;

use crate::domain::search::{
    PopularSearch, RecentSearch, SearchEntityType, SearchFilter, SearchHistoryEntry, SearchResponse,
};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchGlobalInput {
    pub query: String,
    #[serde(default)]
    pub filter: SearchFilter,
    pub limit: Option<usize>,
    /// When true, persist the query into recent searches (debounced client-side).
    #[serde(default)]
    pub record_recent: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordSearchSelectionInput {
    pub query: String,
    pub entity_type: Option<SearchEntityType>,
    pub entity_id: Option<String>,
}

#[tauri::command]
pub async fn search_global(
    state: State<'_, AppState>,
    input: SearchGlobalInput,
) -> Result<SearchResponse, JabeXError> {
    tracing::debug!(
        command = "search_global",
        query_len = input.query.len(),
        "command invoked"
    );
    let filter = input.filter;
    let response =
        state.with_session(|session| session.search.search(&input.query, filter, input.limit))?;
    if input.record_recent && !input.query.trim().is_empty() {
        let _ = state
            .with_session(|session| session.search.record_recent_search(&input.query, &[filter]));
    }
    Ok(response)
}

#[tauri::command]
pub async fn rebuild_search_index(state: State<'_, AppState>) -> Result<u64, JabeXError> {
    tracing::debug!(command = "rebuild_search_index", "command invoked");
    state.with_session(|session| session.search.rebuild_index())
}

#[tauri::command]
pub async fn list_recent_searches(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<RecentSearch>, JabeXError> {
    state.with_session(|session| session.search.list_recent_searches(limit))
}

#[tauri::command]
pub async fn clear_recent_searches(state: State<'_, AppState>) -> Result<(), JabeXError> {
    state.with_session(|session| session.search.clear_recent_searches())
}

#[tauri::command]
pub async fn delete_recent_search(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), JabeXError> {
    let id = Uuid::parse_str(&id).map_err(|e| JabeXError::Validation(e.to_string()))?;
    state.with_session(|session| session.search.delete_recent_search(id))
}

#[tauri::command]
pub async fn list_popular_searches(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<PopularSearch>, JabeXError> {
    state.with_session(|session| session.search.list_popular_searches(limit))
}

#[tauri::command]
pub async fn record_search_selection(
    state: State<'_, AppState>,
    input: RecordSearchSelectionInput,
) -> Result<SearchHistoryEntry, JabeXError> {
    state.with_session(|session| {
        session.search.record_search_selection(
            &input.query,
            input.entity_type,
            input.entity_id.as_deref(),
        )
    })
}

#[tauri::command]
pub async fn list_search_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SearchHistoryEntry>, JabeXError> {
    state.with_session(|session| session.search.list_search_history(limit))
}

#[tauri::command]
pub async fn clear_search_history(state: State<'_, AppState>) -> Result<(), JabeXError> {
    state.with_session(|session| session.search.clear_search_history())
}
