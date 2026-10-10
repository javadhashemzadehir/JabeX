use tauri::State;
use uuid::Uuid;

use crate::domain::errors::JabeXError;
use crate::domain::media_library::{
    MediaIntegrityReport, MediaLibraryFilter, MediaLibraryItem, MediaLibraryStats, MediaPurgeResult,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_media_library(
    state: State<'_, AppState>,
    filter: Option<MediaLibraryFilter>,
) -> Result<Vec<MediaLibraryItem>, JabeXError> {
    tracing::debug!(command = "list_media_library", "command invoked");
    let filter = filter.unwrap_or_default();
    state.with_session(|session| session.media_library.list_library(&filter))
}

#[tauri::command]
pub async fn get_media_library_stats(
    state: State<'_, AppState>,
) -> Result<MediaLibraryStats, JabeXError> {
    tracing::debug!(command = "get_media_library_stats", "command invoked");
    state.with_session(|session| session.media_library.get_stats())
}

#[tauri::command]
pub async fn purge_attachment(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<MediaPurgeResult, JabeXError> {
    tracing::debug!(command = "purge_attachment", "command invoked");
    state.with_session(|session| session.media_library.purge_attachment(id))
}

#[tauri::command]
pub async fn purge_deleted_attachments(
    state: State<'_, AppState>,
) -> Result<MediaPurgeResult, JabeXError> {
    tracing::debug!(command = "purge_deleted_attachments", "command invoked");
    state.with_session(|session| session.media_library.purge_deleted())
}

#[tauri::command]
pub async fn remove_orphan_media_dirs(state: State<'_, AppState>) -> Result<u32, JabeXError> {
    tracing::debug!(command = "remove_orphan_media_dirs", "command invoked");
    state.with_session(|session| session.media_library.remove_orphan_dirs())
}

#[tauri::command]
pub async fn verify_media_library(
    state: State<'_, AppState>,
) -> Result<MediaIntegrityReport, JabeXError> {
    tracing::debug!(command = "verify_media_library", "command invoked");
    state.with_session(|session| session.media_library.verify_integrity())
}
