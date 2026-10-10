use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::language_assist::{SpeechAudio, SpeechProvider, TranslationProvider, TranslationResult};

/// Used when language assist provider is `none` or features are disabled.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopTranslationProvider;

impl TranslationProvider for NoopTranslationProvider {
    fn translate(
        &self,
        _text: &str,
        _source_language: Option<&str>,
        _target_language: &str,
    ) -> JabeXResult<TranslationResult> {
        Err(JabeXError::Network(
            "translation provider is disabled".into(),
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSpeechProvider;

impl SpeechProvider for NoopSpeechProvider {
    fn synthesize(&self, _text: &str, _language: &str) -> JabeXResult<SpeechAudio> {
        Err(JabeXError::Network("speech provider is disabled".into()))
    }
}
