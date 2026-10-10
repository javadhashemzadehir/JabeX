use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::domain::attachment::Attachment;
use crate::domain::card::Card;
use crate::domain::deck::Deck;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::ink::InkDocumentRef;
use crate::domain::profile::Profile;
use crate::domain::repositories::{
    AttachmentRepository, CardRepository, DeckRepository, InkDocumentRepository, ReviewRepository,
    StudyActivityRepository,
};
use crate::domain::review::Review;
use crate::domain::study_activity::StudyActivity;
use crate::infrastructure::database::{open_profile_database, AppPaths, Database};
use crate::infrastructure::repositories::{
    SqliteAttachmentRepository, SqliteCardRepository, SqliteDeckRepository,
    SqliteInkDocumentRepository, SqliteReviewRepository, SqliteStudyActivityRepository,
};
use crate::shared::time::now_utc;

/// Current `.jbx` document schema version.
pub const JBX_FORMAT: &str = "jabex-backup";
/// v2 adds attachment/ink metadata + mediaVersion checksums (v1 still importable).
/// v3 includes soft-deleted decks/cards/settings (preserves `deletedAt` / `deletedBy`).
/// v4 optionally includes `data/study_activity.json` (learning consistency days).
pub const JBX_VERSION: u32 = 4;
pub const MEDIA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JbxManifest {
    pub format: String,
    pub version: u32,
    pub created_at: String,
    pub profile_id: Uuid,
    pub profile_display_name: String,
    pub entity_counts: JbxEntityCounts,
    #[serde(default = "default_media_version")]
    pub media_version: u32,
    #[serde(default)]
    pub media_checksums: Vec<MediaChecksum>,
}

fn default_media_version() -> u32 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JbxEntityCounts {
    pub decks: usize,
    pub cards: usize,
    pub reviews: usize,
    pub settings: usize,
    #[serde(default)]
    pub attachments: usize,
    #[serde(default)]
    pub ink_documents: usize,
    #[serde(default)]
    pub study_activity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaChecksum {
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JbxSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub revision: u32,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub deleted_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Create a new local profile and load backup data into it.
    CreateProfile,
    /// Replace the active profile's study data (destructive).
    ReplaceActive,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub profile_id: Uuid,
    pub mode: String,
    pub decks: usize,
    pub cards: usize,
    pub reviews: usize,
    pub attachments: usize,
    pub ink_documents: usize,
}

/// Versioned `.jbx` export/import (normalized JSON documents inside a ZIP).
pub struct BackupService {
    app_paths: AppPaths,
}

impl BackupService {
    pub fn new(app_paths: AppPaths) -> Self {
        Self { app_paths }
    }

    pub fn export_jbx(
        &self,
        db: &Database,
        profile: &Profile,
        output: Option<PathBuf>,
    ) -> JabeXResult<PathBuf> {
        // Include soft-deleted study entities so Trash survives backup roundtrips.
        let decks = SqliteDeckRepository::new(db.clone()).list_all(true)?;
        let cards = SqliteCardRepository::new(db.clone()).list_all(true)?;
        let reviews = SqliteReviewRepository::new(db.clone()).list_all()?;
        let settings = load_settings(db)?;
        let attachments = SqliteAttachmentRepository::new(db.clone()).list_all(true)?;
        let ink_documents = SqliteInkDocumentRepository::new(db.clone()).list_all(true)?;
        let study_activity =
            SqliteStudyActivityRepository::new(db.clone()).list_by_profile(profile.id)?;

        let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
        let file_name = format!(
            "jabex-{}-{stamp}.jbx",
            sanitize_filename(&profile.display_name)
        );
        let output = output.unwrap_or_else(|| self.app_paths.backups_dir.join(file_name));

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| JabeXError::Backup(format!("failed to create backup dir: {e}")))?;
        }

        let file = File::create(&output)
            .map_err(|e| JabeXError::Backup(format!("failed to create .jbx file: {e}")))?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        let media_checksums = collect_media_checksums(&db.paths().root, &db.paths().media_root)?;

        let manifest = JbxManifest {
            format: JBX_FORMAT.into(),
            version: JBX_VERSION,
            created_at: now_utc().to_rfc3339(),
            profile_id: profile.id,
            profile_display_name: profile.display_name.clone(),
            entity_counts: JbxEntityCounts {
                decks: decks.len(),
                cards: cards.len(),
                reviews: reviews.len(),
                settings: settings.len(),
                attachments: attachments.len(),
                ink_documents: ink_documents.len(),
                study_activity: study_activity.len(),
            },
            media_version: MEDIA_VERSION,
            media_checksums,
        };

        write_json(&mut zip, "manifest.json", &manifest, options)?;
        write_json(&mut zip, "profile.json", profile, options)?;
        write_json(&mut zip, "data/decks.json", &decks, options)?;
        write_json(&mut zip, "data/cards.json", &cards, options)?;
        write_json(&mut zip, "data/reviews.json", &reviews, options)?;
        write_json(&mut zip, "data/settings.json", &settings, options)?;
        write_json(&mut zip, "data/attachments.json", &attachments, options)?;
        write_json(&mut zip, "data/ink_documents.json", &ink_documents, options)?;
        write_json(
            &mut zip,
            "data/study_activity.json",
            &study_activity,
            options,
        )?;

        let media_root = &db.paths().media_root;
        if media_root.exists() {
            add_dir_to_zip(&mut zip, db.paths().root.as_path(), media_root, options)?;
        }

        zip.finish()
            .map_err(|e| JabeXError::Backup(format!("failed to finalize .jbx: {e}")))?;

        info!(path = %output.display(), "jbx backup exported");
        Ok(output)
    }

    pub fn validate_jbx(path: &Path) -> JabeXResult<JbxManifest> {
        let file = File::open(path)
            .map_err(|e| JabeXError::Backup(format!("failed to open backup: {e}")))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| JabeXError::Backup(format!("invalid .jbx archive: {e}")))?;
        let manifest = read_json_from_zip::<JbxManifest>(&mut archive, "manifest.json")?;
        validate_manifest(&manifest)?;
        for required in [
            "profile.json",
            "data/decks.json",
            "data/cards.json",
            "data/reviews.json",
            "data/settings.json",
        ] {
            archive.by_name(required).map_err(|_| {
                JabeXError::Backup(format!("backup missing required file: {required}"))
            })?;
        }
        Ok(manifest)
    }

    pub fn import_jbx(
        &self,
        path: &Path,
        mode: ImportMode,
        active_profile_id: Uuid,
        create_profile: impl FnOnce(String) -> JabeXResult<Profile>,
    ) -> JabeXResult<ImportResult> {
        let _manifest = Self::validate_jbx(path)?;
        let file = File::open(path)
            .map_err(|e| JabeXError::Backup(format!("failed to open backup: {e}")))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| JabeXError::Backup(format!("invalid .jbx archive: {e}")))?;

        let profile_meta = read_json_from_zip::<Profile>(&mut archive, "profile.json")?;
        let decks = read_json_from_zip::<Vec<Deck>>(&mut archive, "data/decks.json")?;
        let cards = read_json_from_zip::<Vec<Card>>(&mut archive, "data/cards.json")?;
        let reviews = read_json_from_zip::<Vec<Review>>(&mut archive, "data/reviews.json")?;
        let settings = read_json_from_zip::<Vec<JbxSetting>>(&mut archive, "data/settings.json")?;
        let attachments =
            read_json_from_zip_optional::<Vec<Attachment>>(&mut archive, "data/attachments.json")?
                .unwrap_or_default();
        let ink_documents = read_json_from_zip_optional::<Vec<InkDocumentRef>>(
            &mut archive,
            "data/ink_documents.json",
        )?
        .unwrap_or_default();
        let mut study_activity = read_json_from_zip_optional::<Vec<StudyActivity>>(
            &mut archive,
            "data/study_activity.json",
        )?
        .unwrap_or_default();

        let target_profile = match mode {
            ImportMode::CreateProfile => {
                let name = format!("{} (import)", profile_meta.display_name);
                create_profile(name)?
            }
            ImportMode::ReplaceActive => Profile {
                id: active_profile_id,
                ..profile_meta.clone()
            },
        };

        let db = open_profile_database(&self.app_paths, target_profile.id)?;
        db.with_transaction(|tx| {
            if matches!(mode, ImportMode::ReplaceActive) {
                wipe_study_data_on(tx)?;
            }
            for deck in &decks {
                SqliteDeckRepository::create_on(tx, deck)?;
            }
            for card in &cards {
                SqliteCardRepository::create_on(tx, card)?;
            }
            for review in &reviews {
                SqliteReviewRepository::create_on(tx, review)?;
            }
            restore_settings_on(tx, &settings)?;
            for attachment in &attachments {
                SqliteAttachmentRepository::create_on(tx, attachment)?;
            }
            for ink in &ink_documents {
                SqliteInkDocumentRepository::create_on(tx, ink)?;
            }
            Ok(())
        })?;

        // Remap study activity to the target profile id (DB is already profile-scoped).
        for activity in &mut study_activity {
            activity.profile_id = target_profile.id;
            SqliteStudyActivityRepository::new(db.clone()).create(activity)?;
        }

        extract_media(&mut archive, &db.paths().root)?;
        verify_imported_media(&db.paths().root, &_manifest.media_checksums)?;

        info!(
            profile_id = %target_profile.id,
            decks = decks.len(),
            cards = cards.len(),
            attachments = attachments.len(),
            ink = ink_documents.len(),
            study_activity = study_activity.len(),
            "jbx backup imported"
        );

        Ok(ImportResult {
            profile_id: target_profile.id,
            mode: match mode {
                ImportMode::CreateProfile => "create_profile".into(),
                ImportMode::ReplaceActive => "replace_active".into(),
            },
            decks: decks.len(),
            cards: cards.len(),
            reviews: reviews.len(),
            attachments: attachments.len(),
            ink_documents: ink_documents.len(),
        })
    }
}

fn validate_manifest(manifest: &JbxManifest) -> JabeXResult<()> {
    if manifest.format != JBX_FORMAT {
        return Err(JabeXError::Backup(format!(
            "unsupported backup format: {}",
            manifest.format
        )));
    }
    if manifest.version == 0 || manifest.version > JBX_VERSION {
        return Err(JabeXError::Backup(format!(
            "unsupported backup version: {} (max {})",
            manifest.version, JBX_VERSION
        )));
    }
    Ok(())
}

fn write_json<T: Serialize>(
    zip: &mut ZipWriter<File>,
    name: &str,
    value: &T,
    options: SimpleFileOptions,
) -> JabeXResult<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| JabeXError::Backup(format!("failed to serialize {name}: {e}")))?;
    zip.start_file(name, options)
        .map_err(|e| JabeXError::Backup(e.to_string()))?;
    zip.write_all(&bytes)
        .map_err(|e| JabeXError::Backup(e.to_string()))?;
    Ok(())
}

fn read_json_from_zip<T: for<'de> Deserialize<'de>>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> JabeXResult<T> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| JabeXError::Backup(format!("missing {name}: {e}")))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| JabeXError::Backup(format!("failed to read {name}: {e}")))?;
    serde_json::from_str(&buf)
        .map_err(|e| JabeXError::Backup(format!("invalid JSON in {name}: {e}")))
}

fn read_json_from_zip_optional<T: for<'de> Deserialize<'de>>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> JabeXResult<Option<T>> {
    match archive.by_name(name) {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .map_err(|e| JabeXError::Backup(format!("failed to read {name}: {e}")))?;
            let value = serde_json::from_str(&buf)
                .map_err(|e| JabeXError::Backup(format!("invalid JSON in {name}: {e}")))?;
            Ok(Some(value))
        }
        Err(_) => Ok(None),
    }
}

fn load_settings(db: &Database) -> JabeXResult<Vec<JbxSetting>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, value, revision, created_at, updated_at, deleted_at, deleted_by
             FROM settings",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let value_raw: String = row.get(1)?;
                let value: serde_json::Value =
                    serde_json::from_str(&value_raw).unwrap_or(serde_json::Value::Null);
                Ok(JbxSetting {
                    key: row.get(0)?,
                    value,
                    revision: row.get::<_, i64>(2)? as u32,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    deleted_at: row.get(5)?,
                    deleted_by: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

fn restore_settings_on(conn: &rusqlite::Connection, settings: &[JbxSetting]) -> JabeXResult<()> {
    for setting in settings {
        let encoded = serde_json::to_string(&setting.value)
            .map_err(|e| JabeXError::Internal(e.to_string()))?;
        conn.execute(
            "INSERT INTO settings (key, value, revision, created_at, updated_at, deleted_at, deleted_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                revision = excluded.revision,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                deleted_by = excluded.deleted_by",
            rusqlite::params![
                setting.key,
                encoded,
                setting.revision,
                setting.created_at,
                setting.updated_at,
                setting.deleted_at,
                setting.deleted_by,
            ],
        )?;
    }
    Ok(())
}

fn wipe_study_data_on(conn: &rusqlite::Connection) -> JabeXResult<()> {
    conn.execute_batch(
        "DELETE FROM reviews;
         DELETE FROM cards;
         DELETE FROM decks;
         DELETE FROM attachments;
         DELETE FROM ink_documents;
         DELETE FROM study_activity;
         DELETE FROM settings;",
    )?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    root: &Path,
    dir: &Path,
    options: SimpleFileOptions,
) -> JabeXResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| JabeXError::Backup(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| JabeXError::Backup(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_to_zip(zip, root, &path, options)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| JabeXError::Backup(e.to_string()))?;
            let name = relative.to_string_lossy().replace('\\', "/");
            zip.start_file(&name, options)
                .map_err(|e| JabeXError::Backup(e.to_string()))?;
            let mut input = File::open(&path).map_err(|e| JabeXError::Backup(e.to_string()))?;
            let mut buffer = Vec::new();
            input
                .read_to_end(&mut buffer)
                .map_err(|e| JabeXError::Backup(e.to_string()))?;
            zip.write_all(&buffer)
                .map_err(|e| JabeXError::Backup(e.to_string()))?;
        }
    }
    Ok(())
}

fn collect_media_checksums(root: &Path, media_root: &Path) -> JabeXResult<Vec<MediaChecksum>> {
    let mut out = Vec::new();
    if !media_root.exists() {
        return Ok(out);
    }
    collect_media_checksums_rec(root, media_root, &mut out)?;
    out.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(out)
}

fn collect_media_checksums_rec(
    root: &Path,
    dir: &Path,
    out: &mut Vec<MediaChecksum>,
) -> JabeXResult<()> {
    for entry in fs::read_dir(dir).map_err(|e| JabeXError::Backup(e.to_string()))? {
        let entry = entry.map_err(|e| JabeXError::Backup(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_media_checksums_rec(root, &path, out)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| JabeXError::Backup(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = fs::read(&path).map_err(|e| JabeXError::Backup(e.to_string()))?;
            out.push(MediaChecksum {
                relative_path: relative,
                sha256: hex::encode(Sha256::digest(&bytes)),
                size_bytes: bytes.len() as u64,
            });
        }
    }
    Ok(())
}

fn extract_media(archive: &mut ZipArchive<File>, profile_root: &Path) -> JabeXResult<()> {
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| JabeXError::Backup(e.to_string()))?;
        let name = file.name().replace('\\', "/");
        if !name.starts_with("media/") || name.ends_with('/') {
            continue;
        }
        let safe_relative = sanitize_zip_media_path(&name)?;
        let out_path = profile_root.join(&safe_relative);
        // Defense in depth: resolved path must stay under profile_root/media.
        let media_root = profile_root.join("media");
        if !out_path.starts_with(&media_root) {
            return Err(JabeXError::Backup(format!(
                "refusing path traversal in backup entry: {name}"
            )));
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| JabeXError::Backup(e.to_string()))?;
        }
        let mut out = File::create(&out_path).map_err(|e| JabeXError::Backup(e.to_string()))?;
        std::io::copy(&mut file, &mut out).map_err(|e| JabeXError::Backup(e.to_string()))?;
    }
    Ok(())
}

fn sanitize_zip_media_path(name: &str) -> JabeXResult<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(JabeXError::Backup(format!(
            "absolute paths are not allowed in backup media: {name}"
        )));
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(JabeXError::Backup(format!(
                    "illegal path component in backup media: {name}"
                )));
            }
        }
    }
    if !safe.starts_with("media") {
        return Err(JabeXError::Backup(format!(
            "media entry must start with media/: {name}"
        )));
    }
    Ok(safe)
}

fn verify_imported_media(profile_root: &Path, checksums: &[MediaChecksum]) -> JabeXResult<()> {
    for item in checksums {
        let safe = sanitize_zip_media_path(&item.relative_path)?;
        let path = profile_root.join(&safe);
        if !path.is_file() {
            return Err(JabeXError::Backup(format!(
                "missing media file after import: {}",
                item.relative_path
            )));
        }
        let bytes = fs::read(&path).map_err(|e| JabeXError::Backup(e.to_string()))?;
        let actual = hex::encode(Sha256::digest(&bytes));
        if !actual.eq_ignore_ascii_case(&item.sha256) {
            return Err(JabeXError::Backup(format!(
                "checksum mismatch for {}",
                item.relative_path
            )));
        }
        if bytes.len() as u64 != item.size_bytes {
            return Err(JabeXError::Backup(format!(
                "size mismatch for {}",
                item.relative_path
            )));
        }
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "profile".into()
    } else {
        cleaned.chars().take(40).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::AttachmentService;
    use crate::domain::attachment::AttachmentOwner;
    use crate::domain::deck::Deck;
    use crate::domain::media_ports::AttachmentStorage;
    use crate::infrastructure::database::{open_database_at, open_profile_database, AppPaths};
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use crate::infrastructure::registry::ProfileRegistry;
    use tempfile::tempdir;

    #[test]
    fn export_import_preserves_soft_deleted_entities() {
        let dir = tempdir().expect("tempdir");
        let app_paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let registry = ProfileRegistry::open(app_paths.clone()).expect("registry");
        let profile = registry
            .create_profile("TrashBackup".into(), None)
            .expect("profile");
        let db = open_profile_database(&app_paths, profile.id).expect("db");
        let deck = Deck::new("Gone".into(), None);
        SqliteDeckRepository::new(db.clone())
            .create(&deck)
            .expect("deck");
        let card = Card::new(deck.id, "front", "back");
        SqliteCardRepository::new(db.clone())
            .create(&card)
            .expect("card");
        let deleted_at = now_utc();
        SqliteDeckRepository::new(db.clone())
            .soft_delete(deck.id, deleted_at, Some(profile.id))
            .expect("soft deck");
        SqliteCardRepository::new(db.clone())
            .soft_delete(card.id, deleted_at, Some(profile.id))
            .expect("soft card");

        let backups = BackupService::new(app_paths.clone());
        let path = backups.export_jbx(&db, &profile, None).expect("export");
        let imported = backups
            .import_jbx(&path, ImportMode::CreateProfile, profile.id, |name| {
                registry.create_profile(name, None)
            })
            .expect("import");

        let imported_db = open_profile_database(&app_paths, imported.profile_id).expect("db2");
        let active = SqliteDeckRepository::new(imported_db.clone())
            .list()
            .expect("active");
        assert!(active.is_empty());
        let all = SqliteDeckRepository::new(imported_db.clone())
            .list_all(true)
            .expect("all");
        assert_eq!(all.len(), 1);
        assert!(all[0].is_deleted());
        assert_eq!(all[0].deleted_by, Some(profile.id));
        let cards = SqliteCardRepository::new(imported_db)
            .list_all(true)
            .expect("cards");
        assert_eq!(cards.len(), 1);
        assert!(cards[0].is_deleted());
    }

    #[test]
    fn export_and_import_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let app_paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let registry = ProfileRegistry::open(app_paths.clone()).expect("registry");
        let profile = registry
            .create_profile("Learner".into(), None)
            .expect("profile");
        registry.set_active_profile(profile.id).expect("active");

        let db = open_profile_database(&app_paths, profile.id).expect("db");
        let deck = Deck::new("Vocab".into(), Some("desc".into()));
        SqliteDeckRepository::new(db.clone())
            .create(&deck)
            .expect("deck");
        let card = Card::new(deck.id, "front", "back");
        SqliteCardRepository::new(db.clone())
            .create(&card)
            .expect("card");

        let backups = BackupService::new(app_paths.clone());
        let path = backups.export_jbx(&db, &profile, None).expect("export");
        assert!(path.extension().and_then(|e| e.to_str()) == Some("jbx"));
        let manifest = BackupService::validate_jbx(&path).expect("validate");
        assert_eq!(manifest.entity_counts.decks, 1);
        assert_eq!(manifest.entity_counts.cards, 1);
        assert_eq!(manifest.media_version, MEDIA_VERSION);

        let imported = backups
            .import_jbx(&path, ImportMode::CreateProfile, profile.id, |name| {
                registry.create_profile(name, None)
            })
            .expect("import");
        assert_ne!(imported.profile_id, profile.id);
        assert_eq!(imported.decks, 1);
        assert_eq!(imported.cards, 1);

        let imported_db = open_profile_database(&app_paths, imported.profile_id).expect("db2");
        let decks = SqliteDeckRepository::new(imported_db).list().expect("list");
        assert_eq!(decks[0].name, "Vocab");
    }

    #[test]
    fn export_import_with_attachment_media() {
        let dir = tempdir().expect("tempdir");
        let app_paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let registry = ProfileRegistry::open(app_paths.clone()).expect("registry");
        let profile = registry
            .create_profile("Media".into(), None)
            .expect("profile");
        let db = open_profile_database(&app_paths, profile.id).expect("db");
        let storage = LocalMediaStorage::new(db.paths().clone());
        let attachments =
            AttachmentService::new(SqliteAttachmentRepository::new(db.clone()), storage);
        let owner = Uuid::now_v7();
        let created = attachments
            .create_from_bytes(
                AttachmentOwner::Card,
                owner,
                "pic.png".into(),
                "image/png".into(),
                b"png-data-123",
            )
            .expect("attach");

        let backups = BackupService::new(app_paths.clone());
        let path = backups.export_jbx(&db, &profile, None).expect("export");
        let manifest = BackupService::validate_jbx(&path).expect("validate");
        assert_eq!(manifest.entity_counts.attachments, 1);
        assert!(!manifest.media_checksums.is_empty());

        let imported = backups
            .import_jbx(&path, ImportMode::CreateProfile, profile.id, |name| {
                registry.create_profile(name, None)
            })
            .expect("import");
        assert_eq!(imported.attachments, 1);

        let imported_db = open_profile_database(&app_paths, imported.profile_id).expect("db2");
        let listed = SqliteAttachmentRepository::new(imported_db.clone())
            .list_all(false)
            .expect("list");
        assert_eq!(listed[0].id, created.id);
        let storage2 = LocalMediaStorage::new(imported_db.paths().clone());
        assert!(storage2
            .verify_checksum(&listed[0].relative_path, &listed[0].hash)
            .expect("checksum"));
    }

    #[test]
    fn rejects_path_traversal_media_names() {
        assert!(sanitize_zip_media_path("media/../secret.txt").is_err());
        assert!(sanitize_zip_media_path("/media/x.png").is_err());
        assert!(sanitize_zip_media_path("media/attachments/a/original.png").is_ok());
    }

    #[test]
    fn rejects_bad_manifest_version() {
        let dir = tempdir().expect("tempdir");
        let bad = JbxManifest {
            format: JBX_FORMAT.into(),
            version: 99,
            created_at: now_utc().to_rfc3339(),
            profile_id: Uuid::nil(),
            profile_display_name: "x".into(),
            entity_counts: JbxEntityCounts::default(),
            media_version: MEDIA_VERSION,
            media_checksums: vec![],
        };
        assert!(validate_manifest(&bad).is_err());
        let _ = open_database_at(dir.path());
    }

    fn write_test_jbx(path: &std::path::Path, files: &[(&str, &str)]) {
        let file = File::create(path).expect("create");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, body) in files {
            zip.start_file(*name, options).expect("start");
            zip.write_all(body.as_bytes()).expect("write");
        }
        zip.finish().expect("finish");
    }

    fn sample_manifest_json(format: &str, version: u32) -> String {
        serde_json::json!({
            "format": format,
            "version": version,
            "createdAt": now_utc().to_rfc3339(),
            "profileId": Uuid::nil().to_string(),
            "profileDisplayName": "Test",
            "entityCounts": {
                "decks": 0,
                "cards": 0,
                "reviews": 0,
                "attachments": 0,
                "inkDocuments": 0,
                "settings": 0
            },
            "mediaVersion": MEDIA_VERSION,
            "mediaChecksums": []
        })
        .to_string()
    }

    #[test]
    fn validate_jbx_rejects_corrupt_zip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad.jbx");
        fs::write(&path, b"not-a-zip").expect("write");
        assert!(BackupService::validate_jbx(&path).is_err());
    }

    #[test]
    fn validate_jbx_rejects_missing_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("missing-manifest.jbx");
        write_test_jbx(&path, &[("profile.json", "{}"), ("data/decks.json", "[]")]);
        assert!(BackupService::validate_jbx(&path).is_err());
    }

    #[test]
    fn validate_jbx_rejects_missing_required_data_files() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("missing-data.jbx");
        let manifest = sample_manifest_json(JBX_FORMAT, JBX_VERSION);
        write_test_jbx(
            &path,
            &[
                ("manifest.json", &manifest),
                ("profile.json", "{}"),
                ("data/decks.json", "[]"),
                // missing cards/reviews/settings
            ],
        );
        let err = BackupService::validate_jbx(&path).expect_err("should fail");
        assert!(err.to_string().contains("missing required file"));
    }

    #[test]
    fn validate_jbx_rejects_unsupported_format_and_version() {
        let dir = tempdir().expect("tempdir");
        let bad_format = dir.path().join("bad-format.jbx");
        let manifest = sample_manifest_json("other-format", JBX_VERSION);
        write_test_jbx(
            &bad_format,
            &[
                ("manifest.json", &manifest),
                ("profile.json", "{}"),
                ("data/decks.json", "[]"),
                ("data/cards.json", "[]"),
                ("data/reviews.json", "[]"),
                ("data/settings.json", "[]"),
            ],
        );
        assert!(BackupService::validate_jbx(&bad_format).is_err());

        let bad_version = dir.path().join("bad-version.jbx");
        let manifest = sample_manifest_json(JBX_FORMAT, JBX_VERSION + 10);
        write_test_jbx(
            &bad_version,
            &[
                ("manifest.json", &manifest),
                ("profile.json", "{}"),
                ("data/decks.json", "[]"),
                ("data/cards.json", "[]"),
                ("data/reviews.json", "[]"),
                ("data/settings.json", "[]"),
            ],
        );
        assert!(BackupService::validate_jbx(&bad_version).is_err());
    }

    #[test]
    fn replace_active_import_wipes_and_restores() {
        let dir = tempdir().expect("tempdir");
        let app_paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let registry = ProfileRegistry::open(app_paths.clone()).expect("registry");
        let source = registry
            .create_profile("Source".into(), None)
            .expect("source");
        let target = registry
            .create_profile("Target".into(), None)
            .expect("target");

        let source_db = open_profile_database(&app_paths, source.id).expect("source db");
        let deck = Deck::new("Keep".into(), None);
        SqliteDeckRepository::new(source_db.clone())
            .create(&deck)
            .expect("deck");
        let card = Card::new(deck.id, "front", "back");
        SqliteCardRepository::new(source_db.clone())
            .create(&card)
            .expect("card");

        let target_db = open_profile_database(&app_paths, target.id).expect("target db");
        let stale = Deck::new("Stale".into(), None);
        SqliteDeckRepository::new(target_db.clone())
            .create(&stale)
            .expect("stale");

        let backups = BackupService::new(app_paths.clone());
        let path = backups
            .export_jbx(&source_db, &source, None)
            .expect("export");
        let imported = backups
            .import_jbx(&path, ImportMode::ReplaceActive, target.id, |_| {
                unreachable!("replace should not create profile")
            })
            .expect("import");
        assert_eq!(imported.profile_id, target.id);
        assert_eq!(imported.mode, "replace_active");
        assert_eq!(imported.decks, 1);
        assert_eq!(imported.cards, 1);

        let restored = open_profile_database(&app_paths, target.id).expect("restored");
        let decks = SqliteDeckRepository::new(restored).list().expect("list");
        assert_eq!(decks.len(), 1);
        assert_eq!(decks[0].name, "Keep");
        assert_ne!(decks[0].id, stale.id);
    }

    #[test]
    fn profiles_isolate_media_trees() {
        let dir = tempdir().expect("tempdir");
        let app_paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let registry = ProfileRegistry::open(app_paths.clone()).expect("registry");
        let a = registry.create_profile("A".into(), None).expect("a");
        let b = registry.create_profile("B".into(), None).expect("b");
        let db_a = open_profile_database(&app_paths, a.id).expect("db a");
        let db_b = open_profile_database(&app_paths, b.id).expect("db b");
        assert_ne!(db_a.paths().media_root, db_b.paths().media_root);
        assert!(db_a.paths().attachments_dir.exists());
        assert!(db_b.paths().ink_dir.exists());
    }
}
