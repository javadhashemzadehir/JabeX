use tauri::State;
use uuid::Uuid;

use crate::domain::errors::JabeXError;
use crate::domain::trash::{TrashEntityKind, TrashItem};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItemInput {
    pub entity_kind: TrashEntityKind,
    pub id: Uuid,
}

#[tauri::command]
pub async fn list_deleted_items(
    state: State<'_, AppState>,
) -> Result<Vec<TrashItem>, JabeXError> {
    tracing::debug!(command = "list_deleted_items", "command invoked");
    state.with_session(|session| session.trash.list_deleted_items())
}

#[tauri::command]
pub async fn restore_item(
    state: State<'_, AppState>,
    input: TrashItemInput,
) -> Result<(), JabeXError> {
    tracing::debug!(
        command = "restore_item",
        entity = input.entity_kind.as_str(),
        id = %input.id,
        "command invoked"
    );
    state.with_session(|session| {
        session
            .trash
            .restore_item(input.entity_kind, input.id)?;
        match input.entity_kind {
            TrashEntityKind::Card => {
                let _ = session.reindex_card(input.id);
            }
            TrashEntityKind::Deck => {
                let _ = session.reindex_deck(input.id);
                if let Ok(cards) = session.cards.list_cards(input.id) {
                    for card in cards {
                        let _ = session.reindex_card(card.id);
                    }
                }
            }
            TrashEntityKind::Attachment => {
                if let Ok(att) = session.attachments.get_attachment(input.id) {
                    let _ = session.search.index_attachment(&att);
                }
            }
            TrashEntityKind::InkDocument => {}
        }
        Ok(())
    })
}

#[tauri::command]
pub async fn permanently_delete_item(
    state: State<'_, AppState>,
    input: TrashItemInput,
) -> Result<(), JabeXError> {
    tracing::debug!(
        command = "permanently_delete_item",
        entity = input.entity_kind.as_str(),
        id = %input.id,
        "command invoked"
    );
    state.with_session(|session| {
        session
            .trash
            .permanently_delete_item(input.entity_kind, input.id)
    })
}
