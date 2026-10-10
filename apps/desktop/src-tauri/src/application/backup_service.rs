use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::domain::card::Card;
use crate::domain::deck::Deck;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::profile::Profile;
use crate::domain::repositories::{CardRepository, DeckRepository, ReviewRepository};
use crate::domain::review::Review;
use crate::infrastructure::database::{open_profile_database, AppPaths, Database};
use crate::infrastructure::repositories::{
    SqliteCardRepository, SqliteDeckRepository, SqliteReviewRepository,
};
use crate::shared::time::now_utc;

/// Current `.jbx` document schema version.
pub const JBX_FORMAT: &str = "jabex-backup";
pub const JBX_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JbxManifest {
    pub format: String,
    pub version: u32,
    pub created_at: String,
    pub profile_id: Uuid,
    pub profile_display_name: String,
    pub entity_counts: JbxEntityCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JbxEntityCounts {
    pub decks: usize,
    pub cards: usize,
    pub reviews: usize,
    pub settings: usize,
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
        let decks = SqliteDeckRepository::new(db.clone()).list()?;
        let mut cards = Vec::new();
        for deck in &decks {
            cards.extend(SqliteCardRepository::new(db.clone()).list_by_deck(deck.id)?);
        }
        let reviews = SqliteReviewRepository::new(db.clone()).list_all()?;
        let settings = load_settings(db)?;

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
            },
        };

        write_json(&mut zip, "manifest.json", &manifest, options)?;
        write_json(&mut zip, "profile.json", profile, options)?;
        write_json(&mut zip, "data/decks.json", &decks, options)?;
        write_json(&mut zip, "data/cards.json", &cards, options)?;
        write_json(&mut zip, "data/reviews.json", &reviews, options)?;
        write_json(&mut zip, "data/settings.json", &settings, options)?;

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
        // Ensure required payload files exist.
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

        let target_profile = match mode {
            ImportMode::CreateProfile => {
                let name = format!("{} (import)", profile_meta.display_name);
                create_profile(name)?
            }
            ImportMode::ReplaceActive => {
                // Caller must confirm; we wipe study tables of the active profile.
                Profile {
                    id: active_profile_id,
                    ..profile_meta.clone()
                }
            }
        };

        let db = open_profile_database(&self.app_paths, target_profile.id)?;
        if matches!(mode, ImportMode::ReplaceActive) {
            wipe_study_data(&db)?;
        }

        let deck_repo = SqliteDeckRepository::new(db.clone());
        let card_repo = SqliteCardRepository::new(db.clone());
        let review_repo = SqliteReviewRepository::new(db.clone());

        for deck in &decks {
            deck_repo.create(deck)?;
        }
        for card in &cards {
            card_repo.create(card)?;
        }
        for review in &reviews {
            review_repo.create(review)?;
        }
        restore_settings(&db, &settings)?;
        extract_media(&mut archive, &db.paths().root)?;

        info!(
            profile_id = %target_profile.id,
            decks = decks.len(),
            cards = cards.len(),
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

fn load_settings(db: &Database) -> JabeXResult<Vec<JbxSetting>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, value, revision, created_at, updated_at, deleted_at
             FROM settings WHERE deleted_at IS NULL",
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
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

fn restore_settings(db: &Database, settings: &[JbxSetting]) -> JabeXResult<()> {
    db.with_conn(|conn| {
        for setting in settings {
            let encoded = serde_json::to_string(&setting.value)
                .map_err(|e| JabeXError::Internal(e.to_string()))?;
            conn.execute(
                "INSERT INTO settings (key, value, revision, created_at, updated_at, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(key) DO UPDATE SET
                    value = excluded.value,
                    revision = excluded.revision,
                    updated_at = excluded.updated_at,
                    deleted_at = excluded.deleted_at",
                rusqlite::params![
                    setting.key,
                    encoded,
                    setting.revision,
                    setting.created_at,
                    setting.updated_at,
                    setting.deleted_at,
                ],
            )?;
        }
        Ok(())
    })
}

fn wipe_study_data(db: &Database) -> JabeXResult<()> {
    db.with_conn(|conn| {
        conn.execute_batch(
            "DELETE FROM reviews;
             DELETE FROM cards;
             DELETE FROM decks;
             DELETE FROM attachments;
             DELETE FROM drawings;
             DELETE FROM settings;",
        )?;
        Ok(())
    })
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

fn extract_media(archive: &mut ZipArchive<File>, profile_root: &Path) -> JabeXResult<()> {
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| JabeXError::Backup(e.to_string()))?;
        let name = file.name().to_string();
        if !name.starts_with("media/") || name.ends_with('/') {
            continue;
        }
        let out_path = profile_root.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| JabeXError::Backup(e.to_string()))?;
        }
        let mut out = File::create(&out_path).map_err(|e| JabeXError::Backup(e.to_string()))?;
        std::io::copy(&mut file, &mut out).map_err(|e| JabeXError::Backup(e.to_string()))?;
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
    use crate::domain::deck::Deck;
    use crate::infrastructure::database::{open_database_at, AppPaths};
    use crate::infrastructure::registry::ProfileRegistry;
    use tempfile::tempdir;

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
        let card = Card::new(deck.id, "front".into(), "back".into());
        SqliteCardRepository::new(db.clone())
            .create(&card)
            .expect("card");

        let backups = BackupService::new(app_paths.clone());
        let path = backups.export_jbx(&db, &profile, None).expect("export");
        assert!(path.extension().and_then(|e| e.to_str()) == Some("jbx"));
        let manifest = BackupService::validate_jbx(&path).expect("validate");
        assert_eq!(manifest.entity_counts.decks, 1);
        assert_eq!(manifest.entity_counts.cards, 1);

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
    fn rejects_bad_manifest_version() {
        let dir = tempdir().expect("tempdir");
        let bad = JbxManifest {
            format: JBX_FORMAT.into(),
            version: 99,
            created_at: now_utc().to_rfc3339(),
            profile_id: Uuid::nil(),
            profile_display_name: "x".into(),
            entity_counts: JbxEntityCounts::default(),
        };
        assert!(validate_manifest(&bad).is_err());

        // touch unused helper so open_database_at stays linked in test cfg if needed
        let _ = open_database_at(dir.path());
    }
}
