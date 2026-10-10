use std::path::PathBuf;

use tauri::State;

use crate::application::backup_service::{ImportMode, ImportResult, JbxManifest};
use crate::domain::errors::JabeXError;
use crate::infrastructure::filesystem::media::{MediaKind, StoredMedia};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMediaInput {
    pub kind: String,
    pub bytes: Vec<u8>,
    pub extension: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredMediaDto {
    pub sha256: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub duplicated: bool,
}

impl From<StoredMedia> for StoredMediaDto {
    fn from(value: StoredMedia) -> Self {
        Self {
            sha256: value.sha256,
            relative_path: value.relative_path,
            size_bytes: value.size_bytes,
            duplicated: value.duplicated,
        }
    }
}

fn parse_kind(kind: &str) -> Result<MediaKind, JabeXError> {
    match kind {
        "image" => Ok(MediaKind::Image),
        "audio" => Ok(MediaKind::Audio),
        "drawing" => Ok(MediaKind::Drawing),
        other => Err(JabeXError::Validation(format!(
            "unknown media kind: {other}"
        ))),
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBackupInput {
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBackupInput {
    pub path: PathBuf,
    /// `create_profile` (default) or `replace_active`
    pub mode: Option<String>,
}

#[tauri::command]
pub async fn export_backup(
    state: State<'_, AppState>,
    input: Option<ExportBackupInput>,
) -> Result<PathBuf, JabeXError> {
    tracing::debug!(command = "export_backup", "command invoked");
    let output = input.and_then(|i| i.output_path);
    state.export_backup(output)
}

/// Deprecated alias — prefer `export_backup` (.jbx).
#[tauri::command]
pub async fn create_backup(state: State<'_, AppState>) -> Result<PathBuf, JabeXError> {
    tracing::debug!(
        command = "create_backup",
        "command invoked (alias of export_backup)"
    );
    state.export_backup(None)
}

#[tauri::command]
pub async fn validate_backup(path: PathBuf) -> Result<JbxManifest, JabeXError> {
    tracing::debug!(command = "validate_backup", "command invoked");
    crate::application::BackupService::validate_jbx(&path)
}

#[tauri::command]
pub async fn import_backup(
    state: State<'_, AppState>,
    input: ImportBackupInput,
) -> Result<ImportResult, JabeXError> {
    tracing::debug!(command = "import_backup", "command invoked");
    let mode = match input.mode.as_deref() {
        None | Some("create_profile") => ImportMode::CreateProfile,
        Some("replace_active") => ImportMode::ReplaceActive,
        Some(other) => {
            return Err(JabeXError::Validation(format!(
                "unknown import mode: {other}"
            )))
        }
    };
    state.import_backup(&input.path, mode)
}

#[tauri::command]
pub async fn store_media(
    state: State<'_, AppState>,
    input: StoreMediaInput,
) -> Result<StoredMediaDto, JabeXError> {
    tracing::debug!(command = "store_media", kind = %input.kind, "command invoked");
    let kind = parse_kind(&input.kind)?;
    state.with_session(|session| {
        let stored = session
            .media
            .store_bytes(kind, &input.bytes, input.extension.as_deref())?;
        Ok(stored.into())
    })
}
