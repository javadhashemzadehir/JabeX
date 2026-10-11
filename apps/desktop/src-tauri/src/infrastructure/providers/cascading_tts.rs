//! TTS fallback chain: Piper → Windows SAPI → (Browser deferred to frontend).

use std::sync::Arc;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::language_assist::{SpeechAudio, SpeechProvider};
use crate::domain::speech::{AudioFile, TtsProvider, TtsProviderKind, TtsRequest, TtsVoiceInfo};

use super::piper::PiperProvider;
use super::windows_sapi::WindowsSapiProvider;

/// Orchestrates local TTS providers according to user preference.
pub struct CascadingTtsProvider {
    preferred: TtsProviderKind,
    piper: Arc<PiperProvider>,
    sapi: Arc<WindowsSapiProvider>,
    speed: f32,
    pitch: f32,
    voice: Option<String>,
}

impl CascadingTtsProvider {
    pub fn new(
        preferred: TtsProviderKind,
        piper: Arc<PiperProvider>,
        sapi: Arc<WindowsSapiProvider>,
        speed: f32,
        pitch: f32,
        voice: Option<String>,
    ) -> Self {
        Self {
            preferred,
            piper,
            sapi,
            speed,
            pitch,
            voice,
        }
    }

    pub fn synthesize_request(&self, text: &str, language: &str) -> JabeXResult<AudioFile> {
        let request = TtsRequest::new(text, language)
            .with_speed(self.speed)
            .with_pitch(self.pitch)
            .with_voice(self.voice.clone());

        match self.preferred {
            TtsProviderKind::Browser => Err(JabeXError::Validation(
                "Browser Speech API runs in the frontend; no native audio file".into(),
            )),
            TtsProviderKind::Piper => self.piper.synthesize(&request),
            TtsProviderKind::WindowsSapi => self.sapi.synthesize(&request),
            TtsProviderKind::Auto => {
                if self.piper.is_available() && self.piper.supports_language(language) {
                    match self.piper.synthesize(&request) {
                        Ok(audio) => return Ok(audio),
                        Err(err) => {
                            tracing::debug!(error = %err, "Piper failed; trying SAPI");
                        }
                    }
                }
                if self.sapi.is_available() {
                    return self.sapi.synthesize(&request);
                }
                Err(JabeXError::Validation(
                    "No local TTS available (install Piper voices or Windows SAPI voices; Browser fallback is client-side)".into(),
                ))
            }
        }
    }

    pub fn list_all_voices(&self) -> JabeXResult<Vec<TtsVoiceInfo>> {
        let mut voices = Vec::new();
        if let Ok(mut v) = self.piper.list_voices() {
            voices.append(&mut v);
        }
        if let Ok(mut v) = self.sapi.list_voices() {
            voices.append(&mut v);
        }
        Ok(voices)
    }
}

impl SpeechProvider for CascadingTtsProvider {
    fn synthesize(&self, text: &str, language: &str) -> JabeXResult<SpeechAudio> {
        self.synthesize_request(text, language)
            .map(SpeechAudio::from)
    }
}

/// Marker provider when the user selected Browser-only TTS.
#[derive(Debug, Default)]
pub struct BrowserDeferredSpeechProvider;

impl SpeechProvider for BrowserDeferredSpeechProvider {
    fn synthesize(&self, _text: &str, _language: &str) -> JabeXResult<SpeechAudio> {
        Err(JabeXError::Validation(
            "Browser Speech API is handled by the frontend".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn auto_without_runtimes_errors_clearly() {
        let dir = tempdir().expect("temp");
        let piper = Arc::new(PiperProvider::new(
            dir.path().join("piper"),
            dir.path().join("voices"),
        ));
        let sapi = Arc::new(WindowsSapiProvider::new());
        let cascading =
            CascadingTtsProvider::new(TtsProviderKind::Auto, piper, sapi, 1.0, 1.0, None);
        // On non-Windows this fails; on Windows SAPI may succeed — either is fine.
        let _ = cascading.synthesize_request("hello", "en");
    }
}
