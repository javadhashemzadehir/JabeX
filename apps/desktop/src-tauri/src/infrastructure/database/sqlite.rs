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
    pub fn resolve() -> JabeXResult<Self> {
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
            &self.images_dir,
            &self.audio_dir,
            &self.drawings_dir,
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

    fn migrate(&mut self) -> JabeXResult<()> {
        info!("running database migrations");
        let mut conn = self.conn.lock();
        embedded::migrations::runner()
            .run(&mut *conn)
            .map_err(|e| {
                error!(error = %e, "database migration failed");
                JabeXError::Migration(e.to_string())
            })?;
        info!("database migrations complete");
        Ok(())
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

    #[test]
    fn opens_and_migrates_fresh_database() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");
        db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='cards'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1);
            let revision_col: i64 = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('cards') WHERE name='revision'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(revision_col, 1);
            Ok(())
        })
        .expect("query");
    }

    #[test]
    fn enables_foreign_keys() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("open db");
        db.with_conn(|conn| {
            let enabled: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert_eq!(enabled, 1);
            Ok(())
        })
        .expect("pragma");
    }

    #[test]
    fn app_paths_create_layout() {
        let dir = tempdir().expect("tempdir");
        let paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        assert!(paths.profiles_dir.exists());
        assert!(paths.backups_dir.exists());
    }
}
