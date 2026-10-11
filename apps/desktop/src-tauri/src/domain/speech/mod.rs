//! Offline speech / TTS domain ports (ADR 016).
//!
//! Providers never call the network. Synthesis may use local OS APIs (SAPI),
//! bundled Piper models, or defer to the frontend Browser Speech API.

mod audio_cache;
mod pronunciation;

pub use audio_cache::{audio_cache_hash, AudioCacheKey};
pub use pronunciation::{lookup_ipa, PronunciationHint};

use serde::{Deserialize, Serialize};

/// Preferred TTS backend. `Auto` tries Piper → Windows SAPI → Browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TtsProviderKind {
    #[default]
    Auto,
    Piper,
    WindowsSapi,
    Browser,
}

/// Synthesis request shared by all TTS providers.
#[derive(Debug, Clone, PartialEq)]
pub struct TtsRequest {
    pub text: String,
    pub language: String,
    pub voice: Option<String>,
    /// Relative speed (1.0 = normal). Typical range 0.5–1.25.
    pub speed: f32,
    /// Relative pitch (1.0 = normal). SAPI uses a mapped range; Piper may ignore.
    pub pitch: f32,
}

impl TtsRequest {
    pub fn new(text: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            language: language.into(),
            voice: None,
            speed: 1.0,
            pitch: 1.0,
        }
    }

    pub fn with_voice(mut self, voice: Option<String>) -> Self {
        self.voice = voice;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.clamp(0.5, 2.0);
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.5, 2.0);
        self
    }
}

/// WAV (or other) audio produced by a local TTS provider.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioFile {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub extension: String,
    pub provider: &'static str,
    pub voice: Option<String>,
    pub duration_ms: Option<u32>,
}

impl AudioFile {
    pub fn wav(bytes: Vec<u8>, provider: &'static str, voice: Option<String>) -> Self {
        Self {
            bytes,
            mime_type: "audio/wav".into(),
            extension: "wav".into(),
            provider,
            voice,
            duration_ms: None,
        }
    }
}

/// Installed / available voice metadata for settings UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub provider: TtsProviderKind,
}

/// Port: local text-to-speech.
pub trait TtsProvider: Send + Sync {
    fn id(&self) -> &'static str;

    fn synthesize(&self, request: &TtsRequest) -> crate::domain::errors::JabeXResult<AudioFile>;

    fn list_voices(&self) -> crate::domain::errors::JabeXResult<Vec<TtsVoiceInfo>> {
        Ok(Vec::new())
    }

    fn supports_language(&self, language: &str) -> bool {
        let _ = language;
        true
    }

    /// True when the runtime/binary/models needed by this provider are present.
    fn is_available(&self) -> bool {
        true
    }
}

/// Supported study languages for speech / packs UI.
pub const SUPPORTED_SPEECH_LANGUAGES: &[&str] = &["en", "fa", "fr", "de", "es", "ar", "ja", "zh"];

/// Per-language voice preference row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePreference {
    pub id: uuid::Uuid,
    pub language: String,
    pub preferred_voice: Option<String>,
    pub speed: f32,
    pub provider: TtsProviderKind,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cached audio metadata for a card face / lemma.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardAudio {
    pub id: uuid::Uuid,
    pub card_id: uuid::Uuid,
    pub language: String,
    pub provider: String,
    pub voice: Option<String>,
    pub file_path: String,
    pub duration_ms: Option<u32>,
    pub hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Offline translation cache row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationCacheEntry {
    pub id: uuid::Uuid,
    pub source_text: String,
    pub source_language: String,
    pub target_language: String,
    pub translated_text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_and_pitch_are_clamped() {
        let req = TtsRequest::new("hi", "en").with_speed(9.0).with_pitch(0.1);
        assert_eq!(req.speed, 2.0);
        assert_eq!(req.pitch, 0.5);
    }
}
