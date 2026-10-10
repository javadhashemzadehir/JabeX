use std::fs;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use tracing::info;
use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::profile::Profile;
use crate::infrastructure::database::{
    migrate_legacy_layout, open_profile_database, AppPaths, Database,
};
use crate::infrastructure::repositories::helpers::{parse_time, parse_uuid};
use crate::shared::time::now_utc;

const ACTIVE_PROFILE_KEY: &str = "active_profile_id";

/// App-level SQLite registry for local profiles (not study data).
#[derive(Clone)]
pub struct ProfileRegistry {
    conn: Arc<Mutex<Connection>>,
    paths: AppPaths,
}

impl ProfileRegistry {
    pub fn open(paths: AppPaths) -> JabeXResult<Self> {
        let conn = Connection::open(&paths.app_database)
            .map_err(|e| JabeXError::Database(e.to_string()))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )
        .map_err(|e| JabeXError::Database(e.to_string()))?;

        let registry = Self {
            conn: Arc::new(Mutex::new(conn)),
            paths,
        };
        registry.migrate()?;
        Ok(registry)
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    fn migrate(&self) -> JabeXResult<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY NOT NULL,
                display_name TEXT NOT NULL,
                avatar TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_active_at TEXT NOT NULL,
                deleted_at TEXT
            );

            CREATE TABLE IF NOT EXISTS app_meta (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| JabeXError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn create_profile(
        &self,
        display_name: String,
        avatar: Option<String>,
    ) -> JabeXResult<Profile> {
        let name = display_name.trim().to_string();
        if name.is_empty() {
            return Err(JabeXError::Validation("display name is required".into()));
        }
        let profile = Profile::new(name, avatar);
        self.insert_profile(&profile)?;
        // Ensure isolated storage exists even before first study write.
        let _ = self.paths.profile_paths(profile.id)?;
        info!(profile_id = %profile.id, "local profile created");
        Ok(profile)
    }

    fn insert_profile(&self, profile: &Profile) -> JabeXResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO profiles (
                id, display_name, avatar, created_at, updated_at, last_active_at, deleted_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                profile.id.to_string(),
                profile.display_name,
                profile.avatar,
                profile.created_at.to_rfc3339(),
                profile.updated_at.to_rfc3339(),
                profile.last_active_at.to_rfc3339(),
                profile.deleted_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn list_profiles(&self) -> JabeXResult<Vec<Profile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, display_name, avatar, created_at, updated_at, last_active_at, deleted_at
             FROM profiles WHERE deleted_at IS NULL
             ORDER BY last_active_at DESC",
        )?;
        let rows = stmt
            .query_map([], map_profile)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn find_profile(&self, id: Uuid) -> JabeXResult<Option<Profile>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, display_name, avatar, created_at, updated_at, last_active_at, deleted_at
             FROM profiles WHERE id = ?1 AND deleted_at IS NULL",
            params![id.to_string()],
            map_profile,
        )
        .optional()
        .map_err(JabeXError::from)
    }

    pub fn update_profile(
        &self,
        id: Uuid,
        display_name: Option<String>,
        avatar: Option<Option<String>>,
    ) -> JabeXResult<Profile> {
        let mut profile = self.find_profile(id)?.ok_or(JabeXError::NotFound {
            entity: "profile",
            id,
        })?;
        if let Some(name) = display_name {
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(JabeXError::Validation("display name is required".into()));
            }
            profile.display_name = name;
        }
        if let Some(avatar) = avatar {
            profile.avatar = avatar;
        }
        profile.updated_at = now_utc();
        self.persist_profile(&profile)?;
        Ok(profile)
    }

    pub fn soft_delete_profile(&self, id: Uuid) -> JabeXResult<()> {
        let active = self.active_profile_id()?;
        if active == Some(id) {
            return Err(JabeXError::Validation(
                "cannot delete the active profile; switch first".into(),
            ));
        }
        let mut profile = self.find_profile(id)?.ok_or(JabeXError::NotFound {
            entity: "profile",
            id,
        })?;
        profile.soft_delete(now_utc());
        self.persist_profile(&profile)?;
        Ok(())
    }

    fn persist_profile(&self, profile: &Profile) -> JabeXResult<()> {
        let conn = self.conn.lock();
        let changed = conn.execute(
            "UPDATE profiles SET
                display_name = ?2,
                avatar = ?3,
                updated_at = ?4,
                last_active_at = ?5,
                deleted_at = ?6
             WHERE id = ?1",
            params![
                profile.id.to_string(),
                profile.display_name,
                profile.avatar,
                profile.updated_at.to_rfc3339(),
                profile.last_active_at.to_rfc3339(),
                profile.deleted_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        if changed == 0 {
            return Err(JabeXError::NotFound {
                entity: "profile",
                id: profile.id,
            });
        }
        Ok(())
    }

    pub fn active_profile_id(&self) -> JabeXResult<Option<Uuid>> {
        let conn = self.conn.lock();
        let value: Option<String> = conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = ?1",
                params![ACTIVE_PROFILE_KEY],
                |row| row.get(0),
            )
            .optional()?;
        value.map(|s| parse_uuid(&s).map_err(JabeXError::from)).transpose()
    }

    pub fn set_active_profile(&self, id: Uuid) -> JabeXResult<Profile> {
        let mut profile = self.find_profile(id)?.ok_or(JabeXError::NotFound {
            entity: "profile",
            id,
        })?;
        let now = now_utc();
        profile.touch_active(now);
        self.persist_profile(&profile)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![ACTIVE_PROFILE_KEY, id.to_string()],
        )?;
        Ok(profile)
    }

    pub fn get_active_profile(&self) -> JabeXResult<Profile> {
        let id = self
            .active_profile_id()?
            .ok_or_else(|| JabeXError::Internal("no active profile".into()))?;
        self.find_profile(id)?.ok_or(JabeXError::NotFound {
            entity: "profile",
            id,
        })
    }
}

fn map_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
    Ok(Profile {
        id: parse_uuid(&row.get::<_, String>(0)?)?,
        display_name: row.get(1)?,
        avatar: row.get(2)?,
        created_at: parse_time(&row.get::<_, String>(3)?)?,
        updated_at: parse_time(&row.get::<_, String>(4)?)?,
        last_active_at: parse_time(&row.get::<_, String>(5)?)?,
        deleted_at: row
            .get::<_, Option<String>>(6)?
            .map(|s| parse_time(&s))
            .transpose()?,
    })
}

/// Ensure registry + at least one profile + open active profile DB.
/// Also migrates legacy single-database installs.
pub fn ensure_bootstrapped(app_paths: &AppPaths) -> JabeXResult<(AppPaths, Database, Uuid)> {
    let registry = ProfileRegistry::open(app_paths.clone())?;

    let profiles = registry.list_profiles()?;
    let active_id = if profiles.is_empty() {
        let profile = registry.create_profile("Default".into(), None)?;
        migrate_legacy_layout(app_paths, profile.id)?;
        registry.set_active_profile(profile.id)?;
        profile.id
    } else if let Some(id) = registry.active_profile_id()? {
        if registry.find_profile(id)?.is_some() {
            id
        } else {
            let fallback = profiles[0].id;
            registry.set_active_profile(fallback)?;
            fallback
        }
    } else {
        let id = profiles[0].id;
        registry.set_active_profile(id)?;
        id
    };

    // Drop empty profile dirs leftovers if needed; always open DB.
    let db = open_profile_database(app_paths, active_id)?;
    Ok((app_paths.clone(), db, active_id))
}

/// Remove a profile's on-disk directory (after soft-delete). Best-effort.
#[allow(dead_code)]
pub fn remove_profile_files(app_paths: &AppPaths, profile_id: Uuid) -> JabeXResult<()> {
    let root = app_paths.profile_root(profile_id);
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|e| {
            JabeXError::Filesystem(format!("failed to remove profile data: {e}"))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn bootstrap_creates_default_profile() {
        let dir = tempdir().expect("tempdir");
        let paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");
        let (app_paths, db, profile_id) = ensure_bootstrapped(&paths).expect("boot");
        assert!(app_paths.app_database.exists());
        assert!(db.paths().database_file.exists());
        assert_ne!(profile_id, Uuid::nil());

        let registry = ProfileRegistry::open(app_paths).expect("registry");
        let listed = registry.list_profiles().expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].display_name, "Default");
    }

    #[test]
    fn migrate_legacy_single_db() {
        let dir = tempdir().expect("tempdir");
        let paths = AppPaths::from_root(dir.path().to_path_buf()).expect("paths");

        let tmp_profile = Uuid::now_v7();
        let tmp_db = open_profile_database(&paths, tmp_profile).expect("tmp");
        drop(tmp_db);
        let old_db = dir.path().join("database.sqlite");
        fs::copy(
            paths.profile_root(tmp_profile).join("database.sqlite"),
            &old_db,
        )
        .expect("copy legacy");
        let _ = fs::remove_dir_all(paths.profile_root(tmp_profile));
        // No app.sqlite yet — bootstrap will create Default and move legacy DB.
        let _ = fs::remove_file(paths.app_database.clone());

        let (app_paths, db, profile_id) = ensure_bootstrapped(&paths).expect("boot");
        assert!(!old_db.exists());
        assert!(db.paths().database_file.exists());
        assert!(app_paths.profile_root(profile_id).exists());
    }
}
