use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::domain::card::{Card, CardListPage};
use crate::domain::errors::JabeXError;
use crate::domain::rich_text::RichTextDocument;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCardInput {
    pub deck_id: Uuid,
    pub front_content: RichTextDocument,
    pub back_content: RichTextDocument,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCardInput {
    pub id: Uuid,
    pub front_content: RichTextDocument,
    pub back_content: RichTextDocument,
}

#[tauri::command]
pub async fn create_card(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateCardInput,
) -> Result<Card, JabeXError> {
    tracing::debug!(command = "create_card", deck_id = %input.deck_id, "command invoked");
    let card = state.with_session(|session| {
        session
            .cards
            .create_card(input.deck_id, input.front_content, input.back_content)
    })?;
    let _ = state.with_session(|session| session.reindex_card(card.id));
    // Never block card creation on detection / optional network work.
    state.schedule_language_assist(&app, card.id, false);
    Ok(card)
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
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<CardListPage, JabeXError> {
    tracing::debug!(command = "list_cards", %deck_id, "command invoked");
    state.with_session(|session| session.cards.list_card_summaries(deck_id, limit, offset))
}

#[tauri::command]
pub async fn update_card(
    app: AppHandle,
    state: State<'_, AppState>,
    input: UpdateCardInput,
) -> Result<Card, JabeXError> {
    tracing::debug!(command = "update_card", id = %input.id, "command invoked");
    let card = state.with_session(|session| {
        session
            .cards
            .update_card_content(input.id, input.front_content, input.back_content)
    })?;
    let _ = state.with_session(|session| session.reindex_card(card.id));
    state.schedule_language_assist(&app, card.id, false);
    Ok(card)
}

#[tauri::command]
pub async fn delete_card(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_card", %id, "command invoked");
    let deleted_by = state.active_profile_id();
    state.with_session(|session| {
        session.trash.soft_delete_item(
            crate::domain::trash::TrashEntityKind::Card,
            id,
            Some(deleted_by),
        )?;
        session
            .search
            .remove_entity(crate::domain::search::SearchEntityType::Card, &id.to_string())
    })
}
