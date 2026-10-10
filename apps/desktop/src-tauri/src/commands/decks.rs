use tauri::State;
use uuid::Uuid;

use crate::domain::deck::{Deck, DeckSettings};
use crate::domain::errors::JabeXError;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeckInput {
    pub name: String,
    pub description: Option<String>,
    pub settings: Option<DeckSettings>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameDeckInput {
    pub id: Uuid,
    pub name: String,
}

#[tauri::command]
pub async fn create_deck(
    state: State<'_, AppState>,
    input: CreateDeckInput,
) -> Result<Deck, JabeXError> {
    tracing::debug!(command = "create_deck", "command invoked");
    state.with_session(|session| {
        session
            .decks
            .create_deck(input.name, input.description, input.settings)
    })
}

#[tauri::command]
pub async fn list_decks(state: State<'_, AppState>) -> Result<Vec<Deck>, JabeXError> {
    tracing::debug!(command = "list_decks", "command invoked");
    state.with_session(|session| session.decks.list_decks())
}

#[tauri::command]
pub async fn get_deck(state: State<'_, AppState>, id: Uuid) -> Result<Deck, JabeXError> {
    tracing::debug!(command = "get_deck", %id, "command invoked");
    state.with_session(|session| session.decks.get_deck(id))
}

#[tauri::command]
pub async fn rename_deck(
    state: State<'_, AppState>,
    input: RenameDeckInput,
) -> Result<Deck, JabeXError> {
    tracing::debug!(command = "rename_deck", id = %input.id, "command invoked");
    state.with_session(|session| session.decks.rename_deck(input.id, input.name))
}

#[tauri::command]
pub async fn delete_deck(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_deck", %id, "command invoked");
    state.with_session(|session| session.decks.delete_deck(id))
}
