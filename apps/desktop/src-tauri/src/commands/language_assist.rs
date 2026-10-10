use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::JabeXError;
use crate::domain::language_assist::{LanguageAssistOverrides, LanguageAssistSettings};
use crate::state::AppState;

#[tauri::command]
pub async fn get_language_assist_settings(
    state: State<'_, AppState>,
) -> Result<LanguageAssistSettings, JabeXError> {
    tracing::debug!(command = "get_language_assist_settings", "command invoked");
    state.language_assist_settings()
}

#[tauri::command]
pub async fn set_language_assist_settings(
    state: State<'_, AppState>,
    settings: LanguageAssistSettings,
) -> Result<LanguageAssistSettings, JabeXError> {
    tracing::debug!(command = "set_language_assist_settings", "command invoked");
    state.set_language_assist_settings(settings)
}

#[tauri::command]
pub async fn regenerate_language_assist(
    app: AppHandle,
    state: State<'_, AppState>,
    card_id: Uuid,
) -> Result<Card, JabeXError> {
    tracing::debug!(command = "regenerate_language_assist", %card_id, "command invoked");
    // Synchronous regenerate so the UI gets immediate metadata; audio/translate
    // may still hit the network when enabled — caller chose regenerate explicitly.
    let card = state.run_language_assist(card_id, true)?;
    let _ = app;
    Ok(card)
}

#[tauri::command]
pub async fn read_pronunciation_audio(
    state: State<'_, AppState>,
    card_id: Uuid,
) -> Result<Vec<u8>, JabeXError> {
    tracing::debug!(command = "read_pronunciation_audio", %card_id, "command invoked");
    state.read_pronunciation_bytes(card_id)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCardLanguageAssistInput {
    pub card_id: Uuid,
    pub overrides: LanguageAssistOverrides,
}

/// Update per-card translation / pronunciation toggles, then re-run assist.
#[tauri::command]
pub async fn set_card_language_assist(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SetCardLanguageAssistInput,
) -> Result<Card, JabeXError> {
    tracing::debug!(
        command = "set_card_language_assist",
        card_id = %input.card_id,
        "command invoked"
    );
    let _ = state.set_card_language_assist_overrides(input.card_id, input.overrides)?;
    // Apply immediately so disabling pronunciation clears cached audio.
    let card = state.run_language_assist(input.card_id, true)?;
    let _ = app;
    Ok(card)
}
