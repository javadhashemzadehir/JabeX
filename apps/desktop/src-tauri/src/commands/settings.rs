use serde_json::Value;
use tauri::State;

use crate::domain::errors::JabeXError;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDto {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSettingInput {
    pub key: String,
    pub value: Value,
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<SettingDto>, JabeXError> {
    tracing::debug!(command = "get_setting", %key, "command invoked");
    state.with_session(|session| {
        Ok(session.settings.get(&key)?.map(|entry| SettingDto {
            key: entry.key,
            value: entry.value,
        }))
    })
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    input: SetSettingInput,
) -> Result<SettingDto, JabeXError> {
    tracing::debug!(command = "set_setting", key = %input.key, "command invoked");
    state.with_session(|session| {
        let entry = session.settings.set(input.key, input.value)?;
        Ok(SettingDto {
            key: entry.key,
            value: entry.value,
        })
    })
}
