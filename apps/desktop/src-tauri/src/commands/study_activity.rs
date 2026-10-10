use tauri::State;

use crate::domain::errors::JabeXError;
use crate::domain::study_activity::{StudyActivity, StudyConsistencyStats};
use crate::state::AppState;

#[tauri::command]
pub async fn get_study_consistency_stats(
    state: State<'_, AppState>,
) -> Result<StudyConsistencyStats, JabeXError> {
    tracing::debug!(command = "get_study_consistency_stats", "command invoked");
    state.with_session(|session| session.study_activity.get_consistency_stats())
}

#[tauri::command]
pub async fn list_study_activity(
    state: State<'_, AppState>,
) -> Result<Vec<StudyActivity>, JabeXError> {
    tracing::debug!(command = "list_study_activity", "command invoked");
    state.with_session(|session| session.study_activity.list_activity())
}
