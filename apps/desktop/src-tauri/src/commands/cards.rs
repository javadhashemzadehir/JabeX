use tauri::State;
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::errors::JabeXError;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCardInput {
    pub deck_id: Uuid,
    pub front_content: String,
    pub back_content: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCardInput {
    pub id: Uuid,
    pub front_content: String,
    pub back_content: String,
}

#[tauri::command]
pub async fn create_card(
    state: State<'_, AppState>,
    input: CreateCardInput,
) -> Result<Card, JabeXError> {
    tracing::debug!(command = "create_card", deck_id = %input.deck_id, "command invoked");
    state.with_session(|session| {
        session
            .cards
            .create_card(input.deck_id, input.front_content, input.back_content)
    })
}

#[tauri::command]
pub async fn get_card(state: State<'_, AppState>, id: Uuid) -> Result<Card, JabeXError> {
    tracing::debug!(command = "get_card", %id, "command invoked");
    state.with_session(|session| session.cards.get_card(id))
}

#[tauri::command]
pub async fn list_cards(
    state: State<'_, AppState>,
    deck_id: Uuid,
) -> Result<Vec<Card>, JabeXError> {
    tracing::debug!(command = "list_cards", %deck_id, "command invoked");
    state.with_session(|session| session.cards.list_cards(deck_id))
}

#[tauri::command]
pub async fn update_card(
    state: State<'_, AppState>,
    input: UpdateCardInput,
) -> Result<Card, JabeXError> {
    tracing::debug!(command = "update_card", id = %input.id, "command invoked");
    state.with_session(|session| {
        session
            .cards
            .update_card_content(input.id, input.front_content, input.back_content)
    })
}

#[tauri::command]
pub async fn delete_card(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_card", %id, "command invoked");
    state.with_session(|session| session.cards.delete_card(id))
}
