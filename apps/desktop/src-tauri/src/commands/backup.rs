use std::path::PathBuf;

use tauri::State;

use crate::application::backup_service::{ImportMode, ImportResult, JbxManifest};
use crate::domain::errors::JabeXError;
use crate::state::AppState;

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
