use std::path::PathBuf;

use tauri::State;
use uuid::Uuid;

use crate::domain::attachment::{Attachment, AttachmentOwner};
use crate::domain::errors::JabeXError;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttachmentInput {
    pub owner_type: String,
    pub owner_id: Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Option<Vec<u8>>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAttachmentsInput {
    pub owner_type: String,
    pub owner_id: Uuid,
}

fn parse_owner(value: &str) -> Result<AttachmentOwner, JabeXError> {
    AttachmentOwner::parse(value).map_err(JabeXError::Validation)
}

#[tauri::command]
pub async fn create_attachment(
    state: State<'_, AppState>,
    input: CreateAttachmentInput,
) -> Result<Attachment, JabeXError> {
    tracing::debug!(command = "create_attachment", "command invoked");
    let owner = parse_owner(&input.owner_type)?;
    state.with_session(|session| {
        let attachment = if let Some(path) = input.source_path.as_ref() {
            session.attachments.create_from_path(
                owner,
                input.owner_id,
                path,
                Some(input.file_name.clone()),
                input.mime_type.clone(),
            )?
        } else if let Some(bytes) = input.bytes.as_ref() {
            session.attachments.create_from_bytes(
                owner,
                input.owner_id,
                input.file_name.clone(),
                input.mime_type.clone(),
                bytes,
            )?
        } else {
            return Err(JabeXError::Validation(
                "create_attachment requires bytes or sourcePath".into(),
            ));
        };
        let _ = session.search.index_attachment(&attachment);
        if owner == AttachmentOwner::Card {
            let _ = session.reindex_card(input.owner_id);
        }
        Ok(attachment)
    })
}

#[tauri::command]
pub async fn list_attachments(
    state: State<'_, AppState>,
    input: ListAttachmentsInput,
) -> Result<Vec<Attachment>, JabeXError> {
    tracing::debug!(command = "list_attachments", "command invoked");
    let owner = parse_owner(&input.owner_type)?;
    state.with_session(|session| session.attachments.list_attachments(owner, input.owner_id))
}

#[tauri::command]
pub async fn delete_attachment(state: State<'_, AppState>, id: Uuid) -> Result<(), JabeXError> {
    tracing::debug!(command = "delete_attachment", "command invoked");
    let deleted_by = state.active_profile_id();
    state.with_session(|session| {
        let _ = session.search.remove_entity(
            crate::domain::search::SearchEntityType::Attachment,
            &id.to_string(),
        );
        session.trash.soft_delete_item(
            crate::domain::trash::TrashEntityKind::Attachment,
            id,
            Some(deleted_by),
        )
    })
}

#[tauri::command]
pub async fn restore_attachment(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<Attachment, JabeXError> {
    tracing::debug!(command = "restore_attachment", "command invoked");
    state.with_session(|session| session.attachments.restore_attachment(id))
}

#[tauri::command]
pub async fn read_attachment_bytes(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<Vec<u8>, JabeXError> {
    tracing::debug!(command = "read_attachment_bytes", "command invoked");
    state.with_session(|session| session.attachments.read_bytes(id))
}
