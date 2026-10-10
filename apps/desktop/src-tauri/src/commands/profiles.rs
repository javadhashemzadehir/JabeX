use tauri::State;
use uuid::Uuid;

use crate::domain::errors::JabeXError;
use crate::domain::profile::Profile;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileInput {
    pub display_name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileInput {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub avatar: Option<Option<String>>,
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, JabeXError> {
    tracing::debug!(command = "list_profiles", "command invoked");
    state.profiles.list()
}

#[tauri::command]
pub async fn get_active_profile(state: State<'_, AppState>) -> Result<Profile, JabeXError> {
    tracing::debug!(command = "get_active_profile", "command invoked");
    state.profiles.get_active()
}

#[tauri::command]
pub async fn create_profile(
    state: State<'_, AppState>,
    input: CreateProfileInput,
) -> Result<Profile, JabeXError> {
    tracing::debug!(command = "create_profile", "command invoked");
    state.profiles.create(input.display_name, input.avatar)
}

#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    input: UpdateProfileInput,
) -> Result<Profile, JabeXError> {
    tracing::debug!(command = "update_profile", id = %input.id, "command invoked");
    state
        .profiles
        .update(input.id, input.display_name, input.avatar)
}

#[tauri::command]
pub async fn delete_profile(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_profile", %id, "command invoked");
    state.profiles.delete(id)
}

#[tauri::command]
pub async fn switch_profile(state: State<'_, AppState>, id: Uuid) -> Result<Profile, JabeXError> {
    tracing::debug!(command = "switch_profile", %id, "command invoked");
    state.switch_profile(id)
}

#[tauri::command]
pub async fn is_initial_profile_setup_pending(
    state: State<'_, AppState>,
) -> Result<bool, JabeXError> {
    tracing::debug!(
        command = "is_initial_profile_setup_pending",
        "command invoked"
    );
    state.profiles.is_initial_profile_setup_pending()
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInitialProfileSetupInput {
    pub display_name: String,
}

#[tauri::command]
pub async fn complete_initial_profile_setup(
    state: State<'_, AppState>,
    input: CompleteInitialProfileSetupInput,
) -> Result<Profile, JabeXError> {
    tracing::debug!(
        command = "complete_initial_profile_setup",
        "command invoked"
    );
    state
        .profiles
        .complete_initial_profile_setup(input.display_name)
}
