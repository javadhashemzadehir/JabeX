use tauri::State;

use crate::domain::errors::JabeXError;
use crate::domain::stats::StudyStats;
use crate::state::AppState;

#[tauri::command]
pub async fn get_study_stats(state: State<'_, AppState>) -> Result<StudyStats, JabeXError> {
    tracing::debug!(command = "get_study_stats", "command invoked");
    state.with_session(|session| session.stats.get_study_stats())
}
