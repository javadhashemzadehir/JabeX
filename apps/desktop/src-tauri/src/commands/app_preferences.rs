use tauri::{AppHandle, State};

use crate::desktop::tray::sync_tray_with_preferences;
use crate::domain::app_preferences::AppPreferences;
use crate::domain::errors::JabeXError;
use crate::state::AppState;

#[tauri::command]
pub async fn get_app_preferences(state: State<'_, AppState>) -> Result<AppPreferences, JabeXError> {
    tracing::debug!(command = "get_app_preferences", "command invoked");
    state.app_preferences.get()
}

#[tauri::command]
pub async fn set_app_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: AppPreferences,
) -> Result<AppPreferences, JabeXError> {
    tracing::debug!(command = "set_app_preferences", "command invoked");
    let saved = state.app_preferences.set(preferences)?;
    // Tray rebuild stays in the desktop shell; command only orchestrates.
    sync_tray_with_preferences(&app, &saved)
        .map_err(|e| JabeXError::Internal(format!("failed to sync system tray: {e}")))?;
    Ok(saved)
}
