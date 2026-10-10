use std::path::{Path, PathBuf};
use std::sync::Arc;

use directories::ProjectDirs;
use parking_lot::Mutex;
use rusqlite::Connection;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/infrastructure/database/migrations");
}

/// Top-level on-disk layout for the JabeX app (registry + profiles + backups).
///
/// ```text
/// {app_data}/JabeX/
/// ├── app.sqlite
/// ├── profiles/{uuid}/database.sqlite
/// └── backups/*.jbx
/// ```
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub app_database: PathBuf,
    pub profiles_dir: PathBuf,
    pub backups_dir: PathBuf,
}

impl AppPaths {
    /// Resolve the writable app-data root.
    ///
    /// Priority:
    /// 1. `JABEX_DATA_DIR` (absolute or relative) — used by portable smoke tests / tooling
    /// 2. OS project data dir (`directories::ProjectDirs`) — never the install/portable folder
    ///
    /// Profiles, SQLite, media, and backups always live under this root so a portable
    /// zip can sit on a read-only or USB path without writing next to `JabeX.exe`.
    pub fn resolve() -> JabeXResult<Self> {
        if let Ok(override_dir) = std::env::var("JABEX_DATA_DIR") {
            let trimmed = override_dir.trim();
            if !trimmed.is_empty() {
                return Self::from_root(PathBuf::from(trimmed));
            }
        }
        let project = ProjectDirs::from("app", "JabeX", "JabeX").ok_or_else(|| {
            JabeXError::Filesystem("unable to resolve application data directory".into())
        })?;
        Self::from_root(project.data_dir().to_path_buf())
    }

    pub fn from_root(root: PathBuf) -> JabeXResult<Self> {
        let paths = Self {
            app_database: root.join("app.sqlite"),
            profiles_dir: root.join("profiles"),
            backups_dir: root.join("backups"),
            root,
        };
        paths.ensure_directories()?;
        Ok(paths)
    }

    fn ensure_directories(&self) -> JabeXResult<()> {
        for dir in [&self.root, &self.profiles_dir, &self.backups_dir] {
            std::fs::create_dir_all(dir).map_err(|e| {
                JabeXError::Filesystem(format!("failed to create {}: {e}", dir.display()))
            })?;
        }
        Ok(())
    }

    pub fn profile_root(&self, profile_id: Uuid) -> PathBuf {
        self.profiles_dir.join(profile_id.to_string())
    }

    pub fn profile_paths(&self, profile_id: Uuid) -> JabeXResult<DatabasePaths> {
        DatabasePaths::for_profile(self.profile_root(profile_id), profile_id)
    }
}

/// Per-profile SQLite + media locations.
#[derive(Debug, Clone)]
pub struct DatabasePaths {
    pub profile_id: Uuid,
    pub root: PathBuf,
    pub database_file: PathBuf,
    pub media_root: PathBuf,
    pub attachments_dir: PathBuf,
    pub ink_dir: PathBuf,
    /// Legacy folders (may exist on upgraded installs; not created for new profiles).
    pub images_dir: PathBuf,
    pub audio_dir: PathBuf,
    pub drawings_dir: PathBuf,
}

impl DatabasePaths {
    pub fn for_profile(root: PathBuf, profile_id: Uuid) -> JabeXResult<Self> {
        let media_root = root.join("media");
        let paths = Self {
            profile_id,
            database_file: root.join("database.sqlite"),
            attachments_dir: media_root.join("attachments"),
            ink_dir: media_root.join("ink"),
            images_dir: media_root.join("images"),
            audio_dir: media_root.join("audio"),
            drawings_dir: media_root.join("drawings"),
            media_root,
            root,
        };
        paths.ensure_directories()?;
        Ok(paths)
    }

    /// Test helper: treat `root` as a profile directory with a nil profile id.
    pub fn from_root(root: PathBuf) -> JabeXResult<Self> {
        Self::for_profile(root, Uuid::nil())
    }

    fn ensure_directories(&self) -> JabeXResult<()> {
        for dir in [
            &self.root,
            &self.media_root,
            &self.attachments_dir,
            &self.ink_dir,
            &self.audio_dir,
        ] {
            std::fs::create_dir_all(dir).map_err(|e| {
                JabeXError::Filesystem(format!("failed to create {}: {e}", dir.display()))
            })?;
        }
        Ok(())
    }
}

/// Thread-safe SQLite handle for one profile database.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    paths: DatabasePaths,
}

impl Database {
    pub fn open(paths: DatabasePaths) -> JabeXResult<Self> {
        info!(path = %paths.database_file.display(), "opening profile sqlite database");
        let conn = Connection::open(&paths.database_file).map_err(|e| {
            error!(error = %e, "failed to open sqlite database");
            JabeXError::Database(e.to_string())
        })?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )
        .map_err(|e| JabeXError::Database(e.to_string()))?;

        let mut db = Self {
            conn: Arc::new(Mutex::new(conn)),
            paths,
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn paths(&self) -> &DatabasePaths {
        &self.paths
    }

    pub fn with_conn<T, F>(&self, f: F) -> JabeXResult<T>
    where
        F: FnOnce(&Connection) -> JabeXResult<T>,
    {
        let conn = self.conn.lock();
        f(&conn)
    }

    pub fn with_conn_mut<T, F>(&self, f: F) -> JabeXResult<T>
    where
        F: FnOnce(&mut Connection) -> JabeXResult<T>,
    {
        let mut conn = self.conn.lock();
        f(&mut conn)
    }

    /// Run `f` inside a SQLite transaction; commit on `Ok`, roll back on `Err` or panic.
    pub fn with_transaction<T, F>(&self, f: F) -> JabeXResult<T>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> JabeXResult<T>,
    {
        let mut conn = self.conn.lock();
        let tx = conn
            .transaction()
            .map_err(|e| JabeXError::Database(e.to_string()))?;
        let value = f(&tx)?;
        tx.commit()
            .map_err(|e| JabeXError::Database(e.to_string()))?;
        Ok(value)
    }

    fn migrate(&mut self) -> JabeXResult<()> {
        info!("running database migrations");

        // Upgrade safety: copy the on-disk DB before applying pending migrations so a
        // failed/partial migration never leaves the user without a recoverable file.
        // Fresh databases (no applied migrations yet) skip the backup.
        let backup_path = {
            let conn = self.conn.lock();
            let applied = Self::applied_migration_version(&conn)?;
            let latest = Self::embedded_latest_version();
            if applied > 0 && applied < latest {
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                drop(conn);
                Some(Self::create_pre_migration_backup(
                    &self.paths.database_file,
                )?)
            } else {
                None
            }
        };

        if let Some(ref path) = backup_path {
            info!(backup = %path.display(), "created pre-migration database backup");
        }

        let mut conn = self.conn.lock();
        embedded::migrations::runner()
            .run(&mut *conn)
            .map_err(|e| {
                error!(error = %e, "database migration failed");
                let hint = backup_path
                    .as_ref()
                    .map(|p| format!("; pre-migration backup kept at {}", p.display()))
                    .unwrap_or_default();
                JabeXError::Migration(format!("{e}{hint}"))
            })?;
        info!("database migrations complete");
        Ok(())
    }

    fn embedded_latest_version() -> u32 {
        embedded::migrations::runner()
            .get_migrations()
            .iter()
            .map(|m| m.version())
            .max()
            .unwrap_or(0)
    }

    fn applied_migration_version(conn: &Connection) -> JabeXResult<u32> {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='refinery_schema_history'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| JabeXError::Database(e.to_string()))?;
        if exists == 0 {
            return Ok(0);
        }
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history",
                [],
                |row| row.get(0),
            )
            .map_err(|e| JabeXError::Database(e.to_string()))?;
        Ok(version as u32)
    }

    /// Snapshot `database.sqlite` beside the live file before an upgrade migration.
    fn create_pre_migration_backup(database_file: &Path) -> JabeXResult<PathBuf> {
        if !database_file.exists() {
            return Err(JabeXError::Filesystem(format!(
                "cannot backup missing database: {}",
                database_file.display()
            )));
        }
        let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let file_name = format!(
            "{}.pre-migrate-{stamp}.bak",
            database_file
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| "database.sqlite".into())
        );
        let backup_path = database_file.with_file_name(file_name);
        std::fs::copy(database_file, &backup_path).map_err(|e| {
            JabeXError::Filesystem(format!(
                "failed to create pre-migration backup {}: {e}",
                backup_path.display()
            ))
        })?;
        Ok(backup_path)
    }
}

/// Open the active profile database after ensuring the app registry exists.
pub fn open_database() -> JabeXResult<(AppPaths, Database, uuid::Uuid)> {
    let app_paths = AppPaths::resolve()?;
    crate::infrastructure::registry::ensure_bootstrapped(&app_paths)
}

pub fn open_database_at(root: impl AsRef<Path>) -> JabeXResult<Database> {
    let paths = DatabasePaths::from_root(root.as_ref().to_path_buf())?;
    Database::open(paths)
}

pub fn open_profile_database(app_paths: &AppPaths, profile_id: Uuid) -> JabeXResult<Database> {
    let paths = app_paths.profile_paths(profile_id)?;
    Database::open(paths)
}

/// Move a pre-profile single-DB install into `profiles/{id}/`.
pub fn migrate_legacy_layout(app_paths: &AppPaths, profile_id: Uuid) -> JabeXResult<bool> {
    let legacy_db = app_paths.root.join("database.sqlite");
    let legacy_media = app_paths.root.join("media");
    if !legacy_db.exists() {
        return Ok(false);
    }

    let target = app_paths.profile_paths(profile_id)?;
    if target.database_file.exists() {
        warn!("legacy database found but profile database already exists; skipping move");
        return Ok(false);
    }

    info!(profile_id = %profile_id, "migrating legacy single-database layout");
    std::fs::rename(&legacy_db, &target.database_file)
        .map_err(|e| JabeXError::Filesystem(format!("failed to move legacy database: {e}")))?;

    if legacy_media.exists() {
        // Prefer renaming whole media tree; fall back to leaving it if blocked.
        let _ = std::fs::rename(&legacy_media, &target.media_root);
        // Recreate expected media children if rename moved a tree that lacked them.
        target.ensure_directories()?;
    }

    // Old backups lived under media/backups — keep app-level backups_dir.
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |row| row.get(0),
        )?;
        Ok(count == 1)
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
        let count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name=?1"),
            [column],
            |row| row.get(0),
        )?;
        Ok(count == 1)
    }

    #[test]
    fn opens_and_migrates_fresh_database() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");
        db.with_conn(|conn| {
            assert!(table_exists(conn, "cards")?);
            assert!(column_exists(conn, "cards", "revision")?);
            Ok(())
        })
        .expect("query");
    }

    #[test]
    fn fresh_schema_has_expected_tables_and_columns_through_v7() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");
        db.with_conn(|conn| {
            for table in [
                "decks",
                "cards",
                "reviews",
                "attachments",
                "ink_documents",
                "settings",
                "search_index",
                "recent_searches",
                "search_history",
            ] {
                assert!(table_exists(conn, table)?, "missing table {table}");
            }
            assert!(table_exists(conn, "search_index_fts")?);
            assert!(
                !table_exists(conn, "drawings")?,
                "legacy drawings should be gone"
            );

            for column in [
                "revision",
                "deleted_by",
                "detected_language",
                "translation",
                "pronunciation_audio_path",
                "transliteration_json",
                "language_assist_status",
                "language_assist_overrides_json",
            ] {
                assert!(
                    column_exists(conn, "cards", column)?,
                    "missing cards.{column}"
                );
            }
            assert!(column_exists(conn, "decks", "revision")?);
            assert!(column_exists(conn, "decks", "deleted_by")?);
            assert!(column_exists(conn, "attachments", "owner_type")?);
            assert!(column_exists(conn, "ink_documents", "content_hash")?);
            Ok(())
        })
        .expect("schema");
    }

    #[test]
    fn migrations_are_idempotent_on_reopen() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let db1 = open_database_at(&path).expect("open db1");
        let version1: i64 = db1
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT version FROM refinery_schema_history ORDER BY version DESC LIMIT 1",
                    [],
                    |row| row.get(0),
                )?)
            })
            .expect("version1");
        drop(db1);

        let db2 = open_database_at(&path).expect("open db2");
        let version2: i64 = db2
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT version FROM refinery_schema_history ORDER BY version DESC LIMIT 1",
                    [],
                    |row| row.get(0),
                )?)
            })
            .expect("version2");
        assert_eq!(version1, version2);
        assert!(
            version2 >= 10,
            "expected migrations through V10, got {version2}"
        );
    }

    #[test]
    fn pending_migration_creates_pre_migration_backup() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let db = open_database_at(&path).expect("open");
        db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM refinery_schema_history WHERE version = (
                    SELECT MAX(version) FROM refinery_schema_history
                )",
                [],
            )?;
            Ok(())
        })
        .expect("mark pending");
        drop(db);

        let _db2 = open_database_at(&path).expect("reopen migrates");
        let backups: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("pre-migrate") && n.ends_with(".bak"))
            .collect();
        assert_eq!(
            backups.len(),
            1,
            "expected one pre-migration backup, got {backups:?}"
        );
    }

    #[test]
    fn fresh_database_skips_pre_migration_backup() {
        let dir = tempdir().expect("tempdir");
        let _db = open_database_at(dir.path()).expect("open");
        let backups: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("pre-migrate"))
            .collect();
        assert!(
            backups.is_empty(),
            "fresh DB must not create pre-migration backups: {backups:?}"
        );
    }

    #[test]
    fn enables_foreign_keys_and_wal() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");
        db.with_conn(|conn| {
            let enabled: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert_eq!(enabled, 1);
            let journal: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
            assert_eq!(journal.to_ascii_lowercase(), "wal");
            Ok(())
        })
        .expect("pragma");
    }

    #[test]
    fn with_transaction_commits_and_rolls_back() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");

        db.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO settings (key, value, revision, created_at, updated_at, deleted_at, deleted_by)
                 VALUES ('tx.commit', '{}', 1, '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z', NULL, NULL)",
                [],
            )?;
            Ok(())
        })
        .expect("commit");

        let found: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM settings WHERE key = 'tx.commit'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .expect("count");
        assert_eq!(found, 1);

        let err: JabeXResult<()> = db.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO settings (key, value, revision, created_at, updated_at, deleted_at, deleted_by)
                 VALUES ('tx.rollback', '{}', 1, '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z', NULL, NULL)",
                [],
            )?;
            Err(JabeXError::Internal("force rollback".into()))
        });
        assert!(err.is_err());
        let rolled: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM settings WHERE key = 'tx.rollback'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .expect("count");
        assert_eq!(rolled, 0);
    }

    #[test]
    fn app_paths_create_layout() {
        let dir = tempdir().expect("tempdir");
        let paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        assert!(paths.profiles_dir.exists());
        assert!(paths.backups_dir.exists());
    }

    #[test]
    fn app_paths_resolve_honors_jabex_data_dir() {
        let dir = tempdir().expect("tempdir");
        std::env::set_var("JABEX_DATA_DIR", dir.path());
        let paths = AppPaths::resolve().expect("resolve");
        std::env::remove_var("JABEX_DATA_DIR");
        assert_eq!(paths.root, dir.path());
        assert!(paths.app_database.starts_with(dir.path()));
    }
}
