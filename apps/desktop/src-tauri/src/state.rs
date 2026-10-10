use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::application::{
    AppPreferencesService, BackupService, CardService, DeckService, ProfileService, ReviewService,
    SettingsService,
};
use crate::domain::errors::JabeXResult;
use crate::domain::profile::Profile;
use crate::domain::scheduler::LeitnerScheduler;
use crate::infrastructure::database::{open_profile_database, AppPaths, Database};
use crate::infrastructure::filesystem::MediaManager;
use crate::infrastructure::registry::ProfileRegistry;
use crate::infrastructure::repositories::{
    SqliteCardRepository, SqliteDeckRepository, SqliteReviewRepository,
};

/// Active profile session: isolated DB + services.
pub struct ProfileSession {
    pub profile_id: Uuid,
    pub db: Database,
    pub media: MediaManager,
    pub decks: DeckService<SqliteDeckRepository>,
    pub cards: CardService<SqliteCardRepository, SqliteDeckRepository>,
    pub reviews: ReviewService<SqliteCardRepository, SqliteReviewRepository, LeitnerScheduler>,
    pub settings: SettingsService,
}

impl ProfileSession {
    pub fn open(app_paths: &AppPaths, profile_id: Uuid) -> JabeXResult<Self> {
        let db = open_profile_database(app_paths, profile_id)?;
        Ok(Self::from_db(db))
    }

    fn from_db(db: Database) -> Self {
        let profile_id = db.paths().profile_id;
        let media = MediaManager::new(db.paths().clone());
        Self {
            profile_id,
            media,
            decks: DeckService::new(SqliteDeckRepository::new(db.clone())),
            cards: CardService::new(
                SqliteCardRepository::new(db.clone()),
                SqliteDeckRepository::new(db.clone()),
            ),
            reviews: ReviewService::new(
                SqliteCardRepository::new(db.clone()),
                SqliteReviewRepository::new(db.clone()),
                LeitnerScheduler,
            ),
            settings: SettingsService::new(db.clone()),
            db,
        }
    }
}

/// Shared application state injected into Tauri commands.
pub struct AppState {
    pub app_paths: AppPaths,
    pub profiles: ProfileService,
    pub app_preferences: AppPreferencesService,
    pub backups: BackupService,
    session: RwLock<ProfileSession>,
}

impl AppState {
    pub fn new(app_paths: AppPaths, db: Database, _active_profile_id: Uuid) -> JabeXResult<Self> {
        let registry = ProfileRegistry::open(app_paths.clone())?;
        // Preferences share the registry connection; ProfileService keeps ownership of the clone.
        let app_preferences = AppPreferencesService::new(registry.clone());
        let profiles = ProfileService::new(registry);
        let backups = BackupService::new(app_paths.clone());
        let session = ProfileSession::from_db(db);
        Ok(Self {
            app_paths,
            profiles,
            app_preferences,
            backups,
            session: RwLock::new(session),
        })
    }

    pub fn with_session<T, F>(&self, f: F) -> JabeXResult<T>
    where
        F: FnOnce(&ProfileSession) -> JabeXResult<T>,
    {
        let session = self.session.read();
        f(&session)
    }

    pub fn active_profile_id(&self) -> Uuid {
        self.session.read().profile_id
    }

    pub fn switch_profile(&self, profile_id: Uuid) -> JabeXResult<Profile> {
        let profile = self.profiles.set_active(profile_id)?;
        let next = ProfileSession::open(&self.app_paths, profile_id)?;
        *self.session.write() = next;
        Ok(profile)
    }

    pub fn export_backup(
        &self,
        output: Option<std::path::PathBuf>,
    ) -> JabeXResult<std::path::PathBuf> {
        let profile = self.profiles.get_active()?;
        let session = self.session.read();
        self.backups.export_jbx(&session.db, &profile, output)
    }

    pub fn import_backup(
        &self,
        path: &std::path::Path,
        mode: crate::application::backup_service::ImportMode,
    ) -> JabeXResult<crate::application::backup_service::ImportResult> {
        let active_id = self.active_profile_id();
        let result = self.backups.import_jbx(path, mode, active_id, |name| {
            self.profiles.create(name, None)
        })?;

        if matches!(
            mode,
            crate::application::backup_service::ImportMode::ReplaceActive
        ) {
            // Reload session so in-memory services see replaced DB.
            let next = ProfileSession::open(&self.app_paths, active_id)?;
            *self.session.write() = next;
        } else if matches!(
            mode,
            crate::application::backup_service::ImportMode::CreateProfile
        ) {
            // Optionally switch to imported profile.
            let _ = self.switch_profile(result.profile_id);
        }
        Ok(result)
    }
}

pub type SharedState = Arc<AppState>;
