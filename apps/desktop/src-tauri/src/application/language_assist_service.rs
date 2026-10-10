use std::sync::Arc;

use uuid::Uuid;

use crate::application::settings_service::SettingsService;
use crate::domain::card::Card;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::language_assist::{
    LanguageAssistOverrides, LanguageAssistProviderKind, LanguageAssistSettings,
    LanguageAssistStatus, LanguageDetectionService, PronunciationAudioStorage, SpeechProvider,
    TranslationProvider, LANGUAGE_ASSIST_SETTING_KEY,
};
use crate::domain::language_detection::{study_lemma, HeuristicLanguageDetector};
use crate::domain::repositories::{CardRepository, DeckRepository};
use crate::infrastructure::providers::{
    GoogleSpeechProvider, GoogleTranslationProvider, NoopSpeechProvider, NoopTranslationProvider,
};
use crate::infrastructure::repositories::{SqliteCardRepository, SqliteDeckRepository};
use crate::shared::time::now_utc;

/// Orchestrates offline detection + optional network translation/TTS.
///
/// Effective network features resolve as: profile master → deck override → card override.
pub struct LanguageAssistService<C, D, S>
where
    C: CardRepository,
    D: DeckRepository,
    S: PronunciationAudioStorage,
{
    cards: C,
    decks: D,
    settings: SettingsService,
    audio: S,
    detector: HeuristicLanguageDetector,
}

impl
    LanguageAssistService<
        SqliteCardRepository,
        SqliteDeckRepository,
        crate::infrastructure::filesystem::LocalMediaStorage,
    >
{
    pub fn from_session(
        db: crate::infrastructure::database::Database,
        media: crate::infrastructure::filesystem::LocalMediaStorage,
    ) -> Self {
        Self {
            cards: SqliteCardRepository::new(db.clone()),
            decks: SqliteDeckRepository::new(db.clone()),
            settings: SettingsService::new(db),
            audio: media,
            detector: HeuristicLanguageDetector,
        }
    }
}

impl<C, D, S> LanguageAssistService<C, D, S>
where
    C: CardRepository,
    D: DeckRepository,
    S: PronunciationAudioStorage,
{
    pub fn load_settings(&self) -> JabeXResult<LanguageAssistSettings> {
        match self.settings.get(LANGUAGE_ASSIST_SETTING_KEY)? {
            None => Ok(LanguageAssistSettings::default()),
            Some(entry) => Ok(serde_json::from_value(entry.value).unwrap_or_default()),
        }
    }

    pub fn save_settings(
        &self,
        settings: LanguageAssistSettings,
    ) -> JabeXResult<LanguageAssistSettings> {
        let value =
            serde_json::to_value(&settings).map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.settings
            .set(LANGUAGE_ASSIST_SETTING_KEY.into(), value)?;
        Ok(settings)
    }

    pub fn set_card_overrides(
        &self,
        card_id: Uuid,
        overrides: LanguageAssistOverrides,
    ) -> JabeXResult<Card> {
        let mut card = self
            .cards
            .find(card_id)?
            .ok_or(JabeXError::NotFound {
                entity: "card",
                id: card_id,
            })?;
        card.language_assist_overrides = overrides;
        card.updated_at = now_utc();
        card.bump_revision();
        self.cards.update(&card)?;
        Ok(card)
    }

    /// Mark pending then run detection / optional providers. Errors are stored as `Failed`.
    pub fn process_card(&self, card_id: Uuid, force: bool) -> JabeXResult<Card> {
        let mut card = self
            .cards
            .find(card_id)?
            .ok_or(JabeXError::NotFound {
                entity: "card",
                id: card_id,
            })?;

        let profile = self.load_settings()?;
        let deck_overrides = self
            .decks
            .find(card.deck_id)?
            .map(|d| d.settings.language_assist)
            .unwrap_or_default();

        card.language_assist_status = LanguageAssistStatus::Pending;
        card.updated_at = now_utc();
        let _ = self.cards.update(&card);

        match self.run_pipeline(&mut card, &profile, &deck_overrides, force) {
            Ok(()) => {
                if card.language_assist_status == LanguageAssistStatus::Pending {
                    card.language_assist_status = LanguageAssistStatus::Ready;
                }
                card.updated_at = now_utc();
                self.cards.update(&card)?;
                Ok(card)
            }
            Err(err) => {
                tracing::warn!(%card_id, error = %err, "language assist failed");
                card.language_assist_status = LanguageAssistStatus::Failed;
                card.updated_at = now_utc();
                let _ = self.cards.update(&card);
                Err(err)
            }
        }
    }

    fn run_pipeline(
        &self,
        card: &mut Card,
        profile: &LanguageAssistSettings,
        deck: &LanguageAssistOverrides,
        force: bool,
    ) -> JabeXResult<()> {
        let want_translation = card
            .language_assist_overrides
            .resolve_translation(profile, deck);
        let want_pronunciation = card
            .language_assist_overrides
            .resolve_pronunciation(profile, deck);

        let front = card.front_content.plain_text();
        let sample = front
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or(front.as_str())
            .to_string();

        if sample.is_empty() {
            card.language_assist_status = LanguageAssistStatus::Skipped;
            return Ok(());
        }

        // Always run local detection (offline).
        let mut detection = self.detector.detect(&sample);
        card.detected_language = Some(detection.language.clone());
        card.transliteration = detection.transliteration.clone();

        let (mut lemma, mut lemma_lang) = study_lemma(&detection, &sample);

        let (translator, speech): (Arc<dyn TranslationProvider>, Arc<dyn SpeechProvider>) =
            match profile.provider {
                LanguageAssistProviderKind::Google => (
                    Arc::new(GoogleTranslationProvider::new()),
                    Arc::new(GoogleSpeechProvider::new()),
                ),
                LanguageAssistProviderKind::None => (
                    Arc::new(NoopTranslationProvider),
                    Arc::new(NoopSpeechProvider),
                ),
            };

        // Optionally resolve unknown Persian transliterations via reverse translate.
        if want_translation {
            if let Some(meta) = card.transliteration.as_mut() {
                if meta.is_transliteration && meta.resolved_word.is_none() {
                    match translator.translate(&sample, Some("fa"), "en") {
                        Ok(result) => {
                            let resolved = result.translated_text.trim().to_string();
                            if !resolved.is_empty()
                                && resolved.to_lowercase() != sample.to_lowercase()
                            {
                                meta.resolved_word = Some(resolved.clone());
                                meta.resolved_language = Some(
                                    result
                                        .detected_source_language
                                        .unwrap_or_else(|| "en".into()),
                                );
                                meta.confidence = meta.confidence.max(0.55);
                                lemma = resolved;
                                lemma_lang = meta
                                    .resolved_language
                                    .clone()
                                    .unwrap_or_else(|| "en".into());
                                detection.transliteration = Some(meta.clone());
                            }
                        }
                        Err(err) => {
                            tracing::debug!(error = %err, "transliteration resolve skipped");
                        }
                    }
                }
            }
        }

        if want_translation {
            let target = profile.target_language.as_str();
            let should_translate = force
                || card.translation.is_none()
                || card.translation.as_deref().unwrap_or("").is_empty();

            if should_translate && lemma_lang != target && lemma_lang != "und" {
                match translator.translate(&lemma, Some(&lemma_lang), target) {
                    Ok(result) => {
                        card.translation = Some(result.translated_text);
                        if let Some(detected) = result.detected_source_language {
                            if card.detected_language.as_deref() == Some("und")
                                || card.detected_language.is_none()
                            {
                                card.detected_language = Some(detected);
                            }
                        }
                    }
                    Err(err) => {
                        tracing::debug!(error = %err, "translation skipped");
                    }
                }
            }
        } else if force {
            card.translation = None;
        }

        if want_pronunciation {
            let need_audio = force
                || card
                    .pronunciation_audio_path
                    .as_ref()
                    .map_or(true, |p| !self.audio.pronunciation_exists(p));

            if need_audio && lemma_lang != "und" && !lemma.is_empty() {
                if let Some(old) = card.pronunciation_audio_path.take() {
                    let _ = self.audio.remove_pronunciation(&old);
                }
                match speech.synthesize(&lemma, &lemma_lang) {
                    Ok(audio) => {
                        let path = self.audio.store_pronunciation(
                            card.id,
                            &audio.bytes,
                            &audio.extension,
                        )?;
                        card.pronunciation_audio_path = Some(path);
                    }
                    Err(err) => {
                        tracing::debug!(error = %err, "pronunciation skipped");
                    }
                }
            }
        } else if force || !want_pronunciation {
            if let Some(old) = card.pronunciation_audio_path.take() {
                let _ = self.audio.remove_pronunciation(&old);
            }
        }

        let has_output = card.detected_language.is_some()
            || card.translation.is_some()
            || card.pronunciation_audio_path.is_some()
            || card.transliteration.is_some();
        card.language_assist_status = if has_output {
            LanguageAssistStatus::Ready
        } else {
            LanguageAssistStatus::Skipped
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::Deck;
    use crate::domain::language_assist::AssistToggle;
    use crate::infrastructure::database::open_database_at;
    use crate::infrastructure::filesystem::LocalMediaStorage;
    use tempfile::tempdir;

    #[test]
    fn local_detection_without_network() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let media = LocalMediaStorage::new(db.paths().clone());
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let service = LanguageAssistService::from_session(db, media);

        let deck = Deck::new("Lang".into(), None);
        decks.create(&deck).expect("deck");
        let card = Card::new(deck.id, "کامپیوتر", "رایانه");
        cards.create(&card).expect("card");

        let processed = service.process_card(card.id, false).expect("process");
        assert_eq!(processed.detected_language.as_deref(), Some("fa"));
        let meta = processed.transliteration.expect("transliteration");
        assert_eq!(meta.resolved_word.as_deref(), Some("computer"));
        assert_eq!(processed.language_assist_status, LanguageAssistStatus::Ready);
        assert!(processed.translation.is_none());
        assert!(processed.pronunciation_audio_path.is_none());
    }

    #[test]
    fn settings_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let media = LocalMediaStorage::new(db.paths().clone());
        let service = LanguageAssistService::from_session(db, media);

        let mut settings = LanguageAssistSettings::default();
        settings.auto_translation = true;
        settings.provider = LanguageAssistProviderKind::Google;
        settings.target_language = "fa".into();
        let saved = service.save_settings(settings.clone()).expect("save");
        assert!(saved.auto_translation);
        let loaded = service.load_settings().expect("load");
        assert_eq!(loaded.provider, LanguageAssistProviderKind::Google);
        assert!(loaded.auto_translation);
    }

    #[test]
    fn card_overrides_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let db = open_database_at(dir.path()).expect("db");
        let media = LocalMediaStorage::new(db.paths().clone());
        let decks = SqliteDeckRepository::new(db.clone());
        let cards = SqliteCardRepository::new(db.clone());
        let service = LanguageAssistService::from_session(db, media);

        let deck = Deck::new("Lang".into(), None);
        decks.create(&deck).expect("deck");
        let card = Card::new(deck.id, "hello", "سلام");
        cards.create(&card).expect("card");

        let updated = service
            .set_card_overrides(
                card.id,
                LanguageAssistOverrides {
                    translation: AssistToggle::Inherit,
                    pronunciation: AssistToggle::Disabled,
                },
            )
            .expect("overrides");
        assert_eq!(
            updated.language_assist_overrides.pronunciation,
            AssistToggle::Disabled
        );
    }
}
