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
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeckSettingsInput {
    pub id: Uuid,
    pub settings: DeckSettings,
}

#[tauri::command]
pub async fn create_deck(
    state: State<'_, AppState>,
    input: CreateDeckInput,
) -> Result<Deck, JabeXError> {
    tracing::debug!(command = "create_deck", "command invoked");
    state.with_session(|session| {
        let deck = session
            .decks
            .create_deck(input.name, input.description, input.settings)?;
        session.reindex_deck(deck.id)?;
        Ok(deck)
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
    state.with_session(|session| {
        let deck = session
            .decks
            .rename_deck(input.id, input.name, input.description)?;
        session.reindex_deck(deck.id)?;
        Ok(deck)
    })
}

#[tauri::command]
pub async fn update_deck_settings(
    state: State<'_, AppState>,
    input: UpdateDeckSettingsInput,
) -> Result<Deck, JabeXError> {
    tracing::debug!(command = "update_deck_settings", id = %input.id, "command invoked");
    state.with_session(|session| session.decks.update_deck_settings(input.id, input.settings))
}

#[tauri::command]
pub async fn delete_deck(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_deck", %id, "command invoked");
    let deleted_by = state.active_profile_id();
    state.with_session(|session| {
        // Collect card ids before soft-delete so we can drop index rows.
        let card_ids: Vec<_> = session
            .cards
            .list_cards(id)
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.id)
            .collect();
        session.trash.soft_delete_item(
            crate::domain::trash::TrashEntityKind::Deck,
            id,
            Some(deleted_by),
        )?;
        session.search.remove_entity(
            crate::domain::search::SearchEntityType::Deck,
            &id.to_string(),
        )?;
        for card_id in card_ids {
            let _ = session.search.remove_entity(
                crate::domain::search::SearchEntityType::Card,
                &card_id.to_string(),
            );
        }
        Ok(())
    })
}
