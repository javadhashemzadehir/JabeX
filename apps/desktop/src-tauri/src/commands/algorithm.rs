use tauri::State;
use uuid::Uuid;

use crate::domain::errors::JabeXError;
use crate::domain::leitner::{
    list_presets, LeitnerConfig, LeitnerPresetInfo, ProfileLeitnerSettings,
};
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveAlgorithmDto {
    pub deck_id: Option<Uuid>,
    pub config: LeitnerConfig,
    pub profile: ProfileLeitnerSettings,
}

#[tauri::command]
pub async fn list_leitner_presets() -> Result<Vec<LeitnerPresetInfo>, JabeXError> {
    Ok(list_presets())
}

#[tauri::command]
pub async fn get_profile_algorithm(
    state: State<'_, AppState>,
) -> Result<ProfileLeitnerSettings, JabeXError> {
    tracing::debug!(command = "get_profile_algorithm", "command invoked");
    state.with_session(|session| session.reviews.load_profile_algorithm())
}

#[tauri::command]
pub async fn set_profile_algorithm(
    state: State<'_, AppState>,
    settings: ProfileLeitnerSettings,
) -> Result<ProfileLeitnerSettings, JabeXError> {
    tracing::debug!(command = "set_profile_algorithm", "command invoked");
    state.with_session(|session| session.reviews.save_profile_algorithm(settings))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyProfileAlgorithmInput {
    /// When true, decks inherit the profile default. When false, copy current profile settings onto each deck.
    pub inherit: bool,
}

#[tauri::command]
pub async fn apply_profile_algorithm_to_all_decks(
    state: State<'_, AppState>,
    input: ApplyProfileAlgorithmInput,
) -> Result<u32, JabeXError> {
    tracing::debug!(
        command = "apply_profile_algorithm_to_all_decks",
        inherit = input.inherit,
        "command invoked"
    );
    state.with_session(|session| {
        session
            .reviews
            .apply_profile_algorithm_to_all_decks(input.inherit)
    })
}

#[tauri::command]
pub async fn get_effective_algorithm(
    state: State<'_, AppState>,
    deck_id: Option<Uuid>,
) -> Result<EffectiveAlgorithmDto, JabeXError> {
    tracing::debug!(command = "get_effective_algorithm", "command invoked");
    state.with_session(|session| {
        let profile = session.reviews.load_profile_algorithm()?;
        let config = match deck_id {
            Some(id) => session.reviews.effective_config_for_deck(id)?,
            None => profile.resolve()?,
        };
        Ok(EffectiveAlgorithmDto {
            deck_id,
            config,
            profile,
        })
    })
}
