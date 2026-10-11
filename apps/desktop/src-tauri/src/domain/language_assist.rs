use serde::{Deserialize, Serialize};

use crate::domain::speech::TtsProviderKind;

/// Profile settings key for optional language assist (stored in `settings`).
pub const LANGUAGE_ASSIST_SETTING_KEY: &str = "language_assist";

/// Translation backend. Offline is the default; Google is legacy/opt-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TranslationProviderKind {
    #[default]
    Offline,
    /// Legacy optional network adapter (disabled in UI; kept for stored settings).
    Google,
    None,
}

/// Legacy combined provider enum (ADR 011). Still accepted on deserialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguageAssistProviderKind {
    #[default]
    None,
    Google,
    Offline,
}

/// Per-profile toggles. Network work is never mandatory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageAssistSettings {
    /// When true, translate after local detection using the offline pack/cache.
    pub auto_translation: bool,
    /// When true, synthesize pronunciation audio via the selected TTS provider.
    pub pronunciation_generation: bool,
    /// Preferred TTS backend (`auto` = Piper → SAPI → Browser).
    #[serde(default)]
    pub tts_provider: TtsProviderKind,
    /// Preferred translation backend (default offline).
    #[serde(default)]
    pub translation_provider: TranslationProviderKind,
    /// BCP-47-ish language code for translation target (default Persian).
    pub target_language: String,
    /// Global TTS speed (overridden by language_preferences when set).
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Global pitch hint for SAPI (1.0 = normal).
    #[serde(default = "default_pitch")]
    pub pitch: f32,
    /// Optional preferred voice id (provider-specific).
    #[serde(default)]
    pub preferred_voice: Option<String>,
    /// Legacy field from ADR 011 (`none` | `google` | `offline`).
    #[serde(default)]
    pub provider: LanguageAssistProviderKind,
}

fn default_speed() -> f32 {
    1.0
}

fn default_pitch() -> f32 {
    1.0
}

impl Default for LanguageAssistSettings {
    fn default() -> Self {
        Self {
            auto_translation: false,
            pronunciation_generation: false,
            tts_provider: TtsProviderKind::Auto,
            translation_provider: TranslationProviderKind::Offline,
            target_language: "fa".into(),
            speed: 1.0,
            pitch: 1.0,
            preferred_voice: None,
            provider: LanguageAssistProviderKind::Offline,
        }
    }
}

impl LanguageAssistSettings {
    /// Normalize legacy `provider` into the split translation/TTS fields.
    pub fn normalize(mut self) -> Self {
        match self.provider {
            LanguageAssistProviderKind::Google => {
                self.translation_provider = TranslationProviderKind::Google;
            }
            LanguageAssistProviderKind::Offline => {
                self.translation_provider = TranslationProviderKind::Offline;
            }
            LanguageAssistProviderKind::None => {
                // Old installs with provider=none meant "no network features".
                // Keep translation_provider Offline so enabling toggles works locally.
            }
        }
        self.speed = self.speed.clamp(0.5, 2.0);
        self.pitch = self.pitch.clamp(0.5, 2.0);
        self
    }

    pub fn network_enabled(&self) -> bool {
        matches!(self.translation_provider, TranslationProviderKind::Google)
            && (self.auto_translation || self.pronunciation_generation)
    }

    /// Translation can run when an offline pack/cache or legacy Google is selected.
    pub fn translation_ready(&self) -> bool {
        !matches!(self.translation_provider, TranslationProviderKind::None)
    }

    pub fn provider_ready(&self) -> bool {
        self.translation_ready() || self.pronunciation_generation
    }
}

/// Per-deck / per-card override for a single language-assist feature.
///
/// Resolution (most specific wins): card → deck → profile master toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssistToggle {
    #[default]
    Inherit,
    Enabled,
    Disabled,
}

/// Scoped toggles for text translation and spoken pronunciation (TTS).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LanguageAssistOverrides {
    #[serde(default)]
    pub translation: AssistToggle,
    #[serde(default)]
    pub pronunciation: AssistToggle,
}

impl LanguageAssistOverrides {
    pub fn resolve_translation(
        &self,
        profile: &LanguageAssistSettings,
        deck: &LanguageAssistOverrides,
    ) -> bool {
        resolve_assist_feature(
            profile.auto_translation && profile.translation_ready(),
            deck.translation,
            self.translation,
        )
    }

    pub fn resolve_pronunciation(
        &self,
        profile: &LanguageAssistSettings,
        deck: &LanguageAssistOverrides,
    ) -> bool {
        resolve_assist_feature(
            profile.pronunciation_generation,
            deck.pronunciation,
            self.pronunciation,
        )
    }
}

/// Card override wins when set; otherwise deck; otherwise profile master.
pub fn resolve_assist_feature(
    profile_master: bool,
    deck: AssistToggle,
    card: AssistToggle,
) -> bool {
    if !profile_master {
        return false;
    }
    match card {
        AssistToggle::Enabled => true,
        AssistToggle::Disabled => false,
        AssistToggle::Inherit => match deck {
            AssistToggle::Enabled => true,
            AssistToggle::Disabled => false,
            AssistToggle::Inherit => true,
        },
    }
}

/// Background / persisted processing state for a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguageAssistStatus {
    #[default]
    Idle,
    Pending,
    Ready,
    Failed,
    Skipped,
}

/// Metadata when Persian (or similar) script is a transliteration of a foreign word.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransliterationMetadata {
    pub is_transliteration: bool,
    /// Script of the written form (e.g. `arab`).
    pub source_script: String,
    /// Best-effort original foreign lemma when resolved.
    pub resolved_word: Option<String>,
    /// Language code guessed for the resolved word (e.g. `en`).
    pub resolved_language: Option<String>,
    /// 0.0–1.0 confidence.
    pub confidence: f64,
}

/// Result of local (or provider-assisted) language detection.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDetectionResult {
    pub language: String,
    pub confidence: f64,
    pub script: String,
    pub transliteration: Option<TransliterationMetadata>,
}

/// Translation provider output.
#[derive(Debug, Clone, PartialEq)]
pub struct TranslationResult {
    pub translated_text: String,
    pub detected_source_language: Option<String>,
    pub provider: &'static str,
}

/// Speech provider output (cached under `media/audio/`).
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechAudio {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub extension: String,
    pub provider: &'static str,
}

impl From<crate::domain::speech::AudioFile> for SpeechAudio {
    fn from(audio: crate::domain::speech::AudioFile) -> Self {
        Self {
            bytes: audio.bytes,
            mime_type: audio.mime_type,
            extension: audio.extension,
            provider: audio.provider,
        }
    }
}

/// Port: detect language / transliteration without knowing providers.
pub trait LanguageDetectionService: Send + Sync {
    fn detect(&self, text: &str) -> LanguageDetectionResult;
}

/// Port: optional translation backend.
pub trait TranslationProvider: Send + Sync {
    fn translate(
        &self,
        text: &str,
        source_language: Option<&str>,
        target_language: &str,
    ) -> crate::domain::errors::JabeXResult<TranslationResult>;
}

/// Port: optional TTS backend (legacy thin wrapper around [`crate::domain::speech::TtsProvider`]).
pub trait SpeechProvider: Send + Sync {
    fn synthesize(
        &self,
        text: &str,
        language: &str,
    ) -> crate::domain::errors::JabeXResult<SpeechAudio>;
}

/// Port: cache pronunciation files under the profile media tree.
pub trait PronunciationAudioStorage: Send + Sync {
    fn store_pronunciation(
        &self,
        card_id: uuid::Uuid,
        bytes: &[u8],
        extension: &str,
    ) -> crate::domain::errors::JabeXResult<String>;

    fn store_cached_audio(
        &self,
        hash: &str,
        bytes: &[u8],
        extension: &str,
    ) -> crate::domain::errors::JabeXResult<String>;

    fn remove_pronunciation(&self, relative_path: &str) -> crate::domain::errors::JabeXResult<()>;

    fn pronunciation_exists(&self, relative_path: &str) -> bool;

    fn resolve_absolute(&self, relative_path: &str) -> std::path::PathBuf;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_offline() {
        let settings = LanguageAssistSettings::default();
        assert!(!settings.auto_translation);
        assert!(!settings.pronunciation_generation);
        assert_eq!(
            settings.translation_provider,
            TranslationProviderKind::Offline
        );
        assert_eq!(settings.tts_provider, TtsProviderKind::Auto);
        assert!(!settings.network_enabled());
    }

    #[test]
    fn offline_translation_ready_without_google() {
        let settings = LanguageAssistSettings {
            auto_translation: true,
            ..LanguageAssistSettings::default()
        };
        assert!(settings.translation_ready());
        let deck = LanguageAssistOverrides::default();
        let card = LanguageAssistOverrides::default();
        assert!(card.resolve_translation(&settings, &deck));
    }

    #[test]
    fn pronunciation_master_works_without_network() {
        let profile = LanguageAssistSettings {
            pronunciation_generation: true,
            ..LanguageAssistSettings::default()
        };
        let deck = LanguageAssistOverrides {
            pronunciation: AssistToggle::Enabled,
            ..LanguageAssistOverrides::default()
        };
        let card_off = LanguageAssistOverrides {
            pronunciation: AssistToggle::Disabled,
            ..LanguageAssistOverrides::default()
        };
        let card_on = LanguageAssistOverrides {
            pronunciation: AssistToggle::Inherit,
            ..LanguageAssistOverrides::default()
        };
        assert!(!card_off.resolve_pronunciation(&profile, &deck));
        assert!(card_on.resolve_pronunciation(&profile, &deck));
    }

    #[test]
    fn profile_master_gates_all_overrides() {
        let profile = LanguageAssistSettings::default();
        let deck = LanguageAssistOverrides {
            pronunciation: AssistToggle::Enabled,
            ..LanguageAssistOverrides::default()
        };
        let card = LanguageAssistOverrides {
            pronunciation: AssistToggle::Enabled,
            ..LanguageAssistOverrides::default()
        };
        assert!(!card.resolve_pronunciation(&profile, &deck));
    }
}
