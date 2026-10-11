use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::JabeXError;
use crate::domain::language_assist::{LanguageAssistOverrides, LanguageAssistSettings};
use crate::domain::speech::TtsVoiceInfo;
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
    // Await blocking job so the UI gets updated metadata; network work stays off
    // the async IPC worker (reqwest::blocking must not run on tokio-rt-worker).
    let card = state.run_language_assist_blocking(card_id, true).await?;
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
    // Apply on a blocking thread so disabling pronunciation clears cached audio
    // without dropping a nested Tokio runtime on the async IPC worker.
    let card = state
        .set_card_language_assist_blocking(input.card_id, input.overrides)
        .await?;
    let _ = app;
    Ok(card)
}

#[tauri::command]
pub async fn list_tts_voices(state: State<'_, AppState>) -> Result<Vec<TtsVoiceInfo>, JabeXError> {
    tracing::debug!(command = "list_tts_voices", "command invoked");
    let app_root = state.app_paths.root.clone();
    let (db, media) =
        state.with_session(|session| Ok((session.db.clone(), session.media.clone())))?;
    tokio::task::spawn_blocking(move || {
        crate::application::LanguageAssistService::from_session(db, media, app_root)
            .list_tts_voices()
    })
    .await
    .map_err(|err| JabeXError::Internal(format!("list voices task failed: {err}")))?
}

#[tauri::command]
pub async fn get_speech_voices_dir(state: State<'_, AppState>) -> Result<String, JabeXError> {
    Ok(state.speech_voices_dir().to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn cancel_piper_generation(state: State<'_, AppState>) -> Result<(), JabeXError> {
    state.cancel_piper_generation();
    Ok(())
}
