use tauri::State;
use uuid::Uuid;

use crate::domain::errors::JabeXError;
use crate::domain::ink::{InkDocument, InkDocumentRef, InkOwner};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInkDocumentInput {
    pub owner_type: String,
    pub owner_id: Uuid,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInkDocumentsInput {
    pub owner_type: String,
    pub owner_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InkDocumentBundle {
    pub reference: InkDocumentRef,
    pub document: InkDocument,
}

fn parse_owner(value: &str) -> Result<InkOwner, JabeXError> {
    InkOwner::parse(value).map_err(JabeXError::Validation)
}

#[tauri::command]
pub async fn create_ink_document(
    state: State<'_, AppState>,
    input: CreateInkDocumentInput,
) -> Result<InkDocumentBundle, JabeXError> {
    tracing::debug!(command = "create_ink_document", "command invoked");
    let owner = parse_owner(&input.owner_type)?;
    state.with_session(|session| {
        let (reference, document) =
            session
                .ink
                .create_document(owner, input.owner_id, input.width, input.height)?;
        Ok(InkDocumentBundle {
            reference,
            document,
        })
    })
}

#[tauri::command]
pub async fn get_ink_document(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<InkDocumentBundle, JabeXError> {
    tracing::debug!(command = "get_ink_document", "command invoked");
    state.with_session(|session| {
        let (reference, document) = session.ink.get_document(id)?;
        Ok(InkDocumentBundle {
            reference,
            document,
        })
    })
}

#[tauri::command]
pub async fn list_ink_documents(
    state: State<'_, AppState>,
    input: ListInkDocumentsInput,
) -> Result<Vec<InkDocumentRef>, JabeXError> {
    tracing::debug!(command = "list_ink_documents", "command invoked");
    let owner = parse_owner(&input.owner_type)?;
    state.with_session(|session| session.ink.list_by_owner(owner, input.owner_id))
}

#[tauri::command]
pub async fn save_ink_document(
    state: State<'_, AppState>,
    document: InkDocument,
) -> Result<InkDocumentBundle, JabeXError> {
    tracing::debug!(command = "save_ink_document", "command invoked");
    state.with_session(|session| {
        let (reference, document) = session.ink.save_document(document)?;
        Ok(InkDocumentBundle {
            reference,
            document,
        })
    })
}

#[tauri::command]
pub async fn delete_ink_document(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_ink_document", "command invoked");
    let deleted_by = state.active_profile_id();
    state.with_session(|session| {
        session.trash.soft_delete_item(
            crate::domain::trash::TrashEntityKind::InkDocument,
            id,
            Some(deleted_by),
        )
    })
}
