use serde::{Deserialize, Serialize};

/// Profile settings key for optional language assist (stored in `settings`).
pub const LANGUAGE_ASSIST_SETTING_KEY: &str = "language_assist";

/// Optional network provider for translation / TTS. Default is none (fully offline).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguageAssistProviderKind {
    #[default]
    None,
    Google,
}

/// Per-profile toggles. Network work is never mandatory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageAssistSettings {
    /// When true and a network provider is selected, translate after local detection.
    pub auto_translation: bool,
    /// When true and a network provider is selected, synthesize pronunciation audio.
    pub pronunciation_generation: bool,
    pub provider: LanguageAssistProviderKind,
    /// BCP-47 / Google-style language code for translation target (default Persian).
    pub target_language: String,
}

impl Default for LanguageAssistSettings {
    fn default() -> Self {
        Self {
            auto_translation: false,
            pronunciation_generation: false,
            provider: LanguageAssistProviderKind::None,
            target_language: "fa".into(),
        }
    }
}

impl LanguageAssistSettings {
    pub fn network_enabled(&self) -> bool {
        !matches!(self.provider, LanguageAssistProviderKind::None)
            && (self.auto_translation || self.pronunciation_generation)
    }

    pub fn provider_ready(&self) -> bool {
        !matches!(self.provider, LanguageAssistProviderKind::None)
    }
}

/// Per-deck / per-card override for a single language-assist feature.
///
/// Resolution (most specific wins): card → deck → profile master toggle.
/// Profile provider + feature flag remain the hard gate.
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
            profile.provider_ready() && profile.auto_translation,
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
            profile.provider_ready() && profile.pronunciation_generation,
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

/// Port: optional TTS backend.
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

    fn remove_pronunciation(&self, relative_path: &str) -> crate::domain::errors::JabeXResult<()>;

    fn pronunciation_exists(&self, relative_path: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_offline() {
        let settings = LanguageAssistSettings::default();
        assert!(!settings.auto_translation);
        assert!(!settings.pronunciation_generation);
        assert_eq!(settings.provider, LanguageAssistProviderKind::None);
        assert!(!settings.network_enabled());
    }

    #[test]
    fn network_requires_provider_and_feature() {
        let mut settings = LanguageAssistSettings {
            auto_translation: true,
            provider: LanguageAssistProviderKind::None,
            ..LanguageAssistSettings::default()
        };
        assert!(!settings.network_enabled());
        settings.provider = LanguageAssistProviderKind::Google;
        assert!(settings.network_enabled());
    }

    #[test]
    fn card_can_disable_pronunciation_inside_enabled_deck() {
        let profile = LanguageAssistSettings {
            pronunciation_generation: true,
            provider: LanguageAssistProviderKind::Google,
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
        let profile = LanguageAssistSettings::default(); // pronunciation off
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
