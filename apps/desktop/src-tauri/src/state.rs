use std::sync::Arc;

use parking_lot::RwLock;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::application::{
    AppPreferencesService, AttachmentService, BackupService, CardService, DeckService, InkService,
    LanguageAssistService, MediaLibraryService, ProfileService, ReviewService, SearchService,
    SettingsService, StatsService, StudyActivityService, TrashService,
};
use crate::domain::errors::JabeXResult;
use crate::domain::profile::Profile;
use crate::domain::search::SearchEntityType;
use crate::infrastructure::database::{open_profile_database, AppPaths, Database};
use crate::infrastructure::filesystem::LocalMediaStorage;
use crate::infrastructure::registry::ProfileRegistry;
use crate::infrastructure::repositories::{
    SqliteAttachmentRepository, SqliteCardRepository, SqliteDeckRepository,
    SqliteInkDocumentRepository, SqliteRecentSearchRepository, SqliteReviewRepository,
    SqliteSearchHistoryRepository, SqliteSearchIndexRepository, SqliteStudyActivityRepository,
    SqliteTrashRepository,
};

/// Emitted when a background language-assist job finishes (success or failure).
pub const LANGUAGE_ASSIST_UPDATED_EVENT: &str = "language-assist://updated";

/// Active profile session: isolated DB + services.
pub struct ProfileSession {
    pub profile_id: Uuid,
    pub db: Database,
    pub media: LocalMediaStorage,
    pub decks: DeckService<SqliteDeckRepository>,
    pub cards: CardService<SqliteCardRepository, SqliteDeckRepository>,
    pub reviews: ReviewService<SqliteCardRepository, SqliteReviewRepository, SqliteDeckRepository>,
    pub attachments: AttachmentService<SqliteAttachmentRepository, LocalMediaStorage>,
    pub media_library: MediaLibraryService<SqliteAttachmentRepository, LocalMediaStorage>,
    pub ink: InkService<SqliteInkDocumentRepository, LocalMediaStorage>,
    pub trash: TrashService<
        SqliteTrashRepository,
        SqliteDeckRepository,
        SqliteCardRepository,
        SqliteAttachmentRepository,
        SqliteInkDocumentRepository,
        LocalMediaStorage,
    >,
    pub stats: StatsService,
    pub study_activity: StudyActivityService<SqliteStudyActivityRepository, SqliteReviewRepository>,
    pub search: SearchService<
        SqliteSearchIndexRepository,
        SqliteRecentSearchRepository,
        SqliteSearchHistoryRepository,
        SqliteCardRepository,
        SqliteDeckRepository,
        SqliteAttachmentRepository,
    >,
}

impl ProfileSession {
    pub fn open(app_paths: &AppPaths, profile_id: Uuid) -> JabeXResult<Self> {
        let db = open_profile_database(app_paths, profile_id)?;
        Ok(Self::from_db(db))
    }

    fn from_db(db: Database) -> Self {
        let profile_id = db.paths().profile_id;
        let media = LocalMediaStorage::new(db.paths().clone());
        let settings = SettingsService::new(db.clone());
        Self {
            profile_id,
            decks: DeckService::new(SqliteDeckRepository::new(db.clone())),
            cards: CardService::new(
                SqliteCardRepository::new(db.clone()),
                SqliteDeckRepository::new(db.clone()),
            ),
            reviews: ReviewService::new(
                SqliteCardRepository::new(db.clone()),
                SqliteReviewRepository::new(db.clone()),
                SqliteDeckRepository::new(db.clone()),
                settings.clone(),
            ),
            attachments: AttachmentService::new(
                SqliteAttachmentRepository::new(db.clone()),
                media.clone(),
            ),
            media_library: MediaLibraryService::new(
                SqliteAttachmentRepository::new(db.clone()),
                media.clone(),
            ),
            ink: InkService::new(SqliteInkDocumentRepository::new(db.clone()), media.clone()),
            trash: TrashService::new(
                SqliteTrashRepository::new(db.clone()),
                SqliteDeckRepository::new(db.clone()),
                SqliteCardRepository::new(db.clone()),
                SqliteAttachmentRepository::new(db.clone()),
                SqliteInkDocumentRepository::new(db.clone()),
                media.clone(),
            ),
            stats: StatsService::new(db.clone()),
            study_activity: StudyActivityService::new(
                SqliteStudyActivityRepository::new(db.clone()),
                SqliteReviewRepository::new(db.clone()),
                profile_id,
            ),
            search: SearchService::new(
                SqliteSearchIndexRepository::new(db.clone()),
                SqliteRecentSearchRepository::new(db.clone()),
                SqliteSearchHistoryRepository::new(db.clone()),
                SqliteCardRepository::new(db.clone()),
                SqliteDeckRepository::new(db.clone()),
                SqliteAttachmentRepository::new(db.clone()),
                profile_id,
            ),
            media,
            db,
        }
    }

    /// Best-effort incremental index update after card mutations.
    pub fn reindex_card(&self, card_id: uuid::Uuid) -> JabeXResult<()> {
        let card = match self.cards.get_card(card_id) {
            Ok(card) => card,
            Err(_) => {
                self.search
                    .remove_entity(SearchEntityType::Card, &card_id.to_string())?;
                return Ok(());
            }
        };
        let deck_name = self
            .decks
            .get_deck(card.deck_id)
            .map(|d| d.name)
            .unwrap_or_default();
        let attachments = self
            .attachments
            .list_attachments(crate::domain::attachment::AttachmentOwner::Card, card.id)?;
        self.search.index_card(&card, &deck_name, &attachments)
    }

    pub fn reindex_deck(&self, deck_id: uuid::Uuid) -> JabeXResult<()> {
        let deck = match self.decks.get_deck(deck_id) {
            Ok(deck) => deck,
            Err(_) => {
                self.search
                    .remove_entity(SearchEntityType::Deck, &deck_id.to_string())?;
                return Ok(());
            }
        };
        let count = self.cards.count_cards(deck_id)?;
        self.search.index_deck(&deck, count)
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
            let next = ProfileSession::open(&self.app_paths, active_id)?;
            *self.session.write() = next;
        } else if matches!(
            mode,
            crate::application::backup_service::ImportMode::CreateProfile
        ) {
            let _ = self.switch_profile(result.profile_id);
        }
        Ok(result)
    }

    /// Clone session handles and process language assist off the IPC path.
    /// Network failures are logged; card creation/update already succeeded.
    pub fn schedule_language_assist(&self, app: &AppHandle, card_id: Uuid, force: bool) {
        let (db, media, profile_id) = {
            let session = self.session.read();
            (
                session.db.clone(),
                session.media.clone(),
                session.profile_id,
            )
        };
        let app_root = self.app_paths.root.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                let service = LanguageAssistService::from_session(db, media, app_root);
                service.process_card(card_id, force)
            })
            .await;

            let status = match result {
                Ok(Ok(card)) => card.language_assist_status,
                Ok(Err(err)) => {
                    tracing::warn!(%card_id, error = %err, "language assist job failed");
                    crate::domain::language_assist::LanguageAssistStatus::Failed
                }
                Err(err) => {
                    tracing::warn!(%card_id, error = %err, "language assist join failed");
                    crate::domain::language_assist::LanguageAssistStatus::Failed
                }
            };

            let _ = app.emit(
                LANGUAGE_ASSIST_UPDATED_EVENT,
                serde_json::json!({
                    "cardId": card_id,
                    "profileId": profile_id,
                    "status": status,
                }),
            );
        });
    }

    pub fn run_language_assist(
        &self,
        card_id: Uuid,
        force: bool,
    ) -> JabeXResult<crate::domain::card::Card> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        let service = LanguageAssistService::from_session(db, media, self.app_paths.root.clone());
        service.process_card(card_id, force)
    }

    /// Run language assist off the async IPC worker.
    ///
    /// Blocking TTS (SAPI/Piper) and legacy Google HTTP must not run on a
    /// `tokio-rt-worker`. Always call this from async commands instead of
    /// [`Self::run_language_assist`].
    pub async fn run_language_assist_blocking(
        &self,
        card_id: Uuid,
        force: bool,
    ) -> JabeXResult<crate::domain::card::Card> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        let app_root = self.app_paths.root.clone();
        tokio::task::spawn_blocking(move || {
            LanguageAssistService::from_session(db, media, app_root).process_card(card_id, force)
        })
        .await
        .map_err(|err| {
            crate::domain::errors::JabeXError::Internal(format!(
                "language assist task failed: {err}"
            ))
        })?
    }

    /// Persist per-card overrides then re-run assist on a blocking thread.
    pub async fn set_card_language_assist_blocking(
        &self,
        card_id: Uuid,
        overrides: crate::domain::language_assist::LanguageAssistOverrides,
    ) -> JabeXResult<crate::domain::card::Card> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        let app_root = self.app_paths.root.clone();
        tokio::task::spawn_blocking(move || {
            let service = LanguageAssistService::from_session(db, media, app_root);
            let _ = service.set_card_overrides(card_id, overrides)?;
            service.process_card(card_id, true)
        })
        .await
        .map_err(|err| {
            crate::domain::errors::JabeXError::Internal(format!(
                "language assist task failed: {err}"
            ))
        })?
    }

    pub fn language_assist_settings(
        &self,
    ) -> JabeXResult<crate::domain::language_assist::LanguageAssistSettings> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        LanguageAssistService::from_session(db, media, self.app_paths.root.clone()).load_settings()
    }

    pub fn set_language_assist_settings(
        &self,
        settings: crate::domain::language_assist::LanguageAssistSettings,
    ) -> JabeXResult<crate::domain::language_assist::LanguageAssistSettings> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        LanguageAssistService::from_session(db, media, self.app_paths.root.clone())
            .save_settings(settings)
    }

    pub fn set_card_language_assist_overrides(
        &self,
        card_id: Uuid,
        overrides: crate::domain::language_assist::LanguageAssistOverrides,
    ) -> JabeXResult<crate::domain::card::Card> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        LanguageAssistService::from_session(db, media, self.app_paths.root.clone())
            .set_card_overrides(card_id, overrides)
    }

    pub fn list_tts_voices(&self) -> JabeXResult<Vec<crate::domain::speech::TtsVoiceInfo>> {
        let (db, media) = {
            let session = self.session.read();
            (session.db.clone(), session.media.clone())
        };
        LanguageAssistService::from_session(db, media, self.app_paths.root.clone())
            .list_tts_voices()
    }

    pub fn speech_voices_dir(&self) -> std::path::PathBuf {
        self.app_paths.root.join("data").join("voices")
    }

    pub fn cancel_piper_generation(&self) {
        let piper =
            crate::infrastructure::providers::PiperProvider::from_app_root(&self.app_paths.root);
        piper.cancel_current();
    }

    pub fn read_pronunciation_bytes(&self, card_id: Uuid) -> JabeXResult<Vec<u8>> {
        self.with_session(|session| {
            let card = session.cards.get_card(card_id)?;
            let Some(relative) = card.pronunciation_audio_path.as_ref() else {
                return Err(crate::domain::errors::JabeXError::NotFound {
                    entity: "pronunciation",
                    id: card_id,
                });
            };
            use crate::domain::media_ports::AttachmentStorage;
            session.media.read_bytes(relative)
        })
    }
}

pub type SharedState = Arc<AppState>;
