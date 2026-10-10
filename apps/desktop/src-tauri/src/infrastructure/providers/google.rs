//! Optional Google Translate / TTS adapters (no API key, no account).
//!
//! These call public translate.google.* endpoints only when the user enables
//! language assist in settings. Failures must never block card CRUD.

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::language_assist::{
    SpeechAudio, SpeechProvider, TranslationProvider, TranslationResult,
};

const TRANSLATE_URL: &str = "https://translate.googleapis.com/translate_a/single";
const TTS_URL: &str = "https://translate.google.com/translate_tts";
const USER_AGENT: &str = "JabeX/0.1 (local-first; optional language assist)";

#[derive(Debug, Clone)]
pub struct GoogleTranslationProvider {
    client: reqwest::blocking::Client,
}

impl Default for GoogleTranslationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleTranslationProvider {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }
}

impl TranslationProvider for GoogleTranslationProvider {
    fn translate(
        &self,
        text: &str,
        source_language: Option<&str>,
        target_language: &str,
    ) -> JabeXResult<TranslationResult> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(JabeXError::Validation("translation text is empty".into()));
        }
        if trimmed.chars().count() > 500 {
            return Err(JabeXError::Validation(
                "translation text is too long (max 500 characters)".into(),
            ));
        }

        let sl = source_language.unwrap_or("auto");
        let response = self
            .client
            .get(TRANSLATE_URL)
            .query(&[
                ("client", "gtx"),
                ("sl", sl),
                ("tl", target_language),
                ("dt", "t"),
                ("q", trimmed),
            ])
            .send()
            .map_err(|e| JabeXError::Network(format!("google translate request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(JabeXError::Network(format!(
                "google translate HTTP {}",
                response.status()
            )));
        }

        let body_text = response
            .text()
            .map_err(|e| JabeXError::Network(format!("google translate body failed: {e}")))?;
        let body: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| JabeXError::Network(format!("google translate parse failed: {e}")))?;

        let translated = extract_translated_text(&body).ok_or_else(|| {
            JabeXError::Network("google translate returned an unexpected payload".into())
        })?;
        let detected = extract_detected_language(&body);

        Ok(TranslationResult {
            translated_text: translated,
            detected_source_language: detected,
            provider: "google",
        })
    }
}

#[derive(Debug, Clone)]
pub struct GoogleSpeechProvider {
    client: reqwest::blocking::Client,
}

impl Default for GoogleSpeechProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleSpeechProvider {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }
}

impl SpeechProvider for GoogleSpeechProvider {
    fn synthesize(&self, text: &str, language: &str) -> JabeXResult<SpeechAudio> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(JabeXError::Validation("speech text is empty".into()));
        }
        if trimmed.chars().count() > 200 {
            return Err(JabeXError::Validation(
                "speech text is too long (max 200 characters)".into(),
            ));
        }

        let response = self
            .client
            .get(TTS_URL)
            .query(&[
                ("ie", "UTF-8"),
                ("client", "tw-ob"),
                ("q", trimmed),
                ("tl", language),
            ])
            .send()
            .map_err(|e| JabeXError::Network(format!("google tts request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(JabeXError::Network(format!(
                "google tts HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| JabeXError::Network(format!("google tts body failed: {e}")))?
            .to_vec();

        if bytes.is_empty() {
            return Err(JabeXError::Network(
                "google tts returned empty audio".into(),
            ));
        }

        Ok(SpeechAudio {
            bytes,
            mime_type: "audio/mpeg".into(),
            extension: "mp3".into(),
            provider: "google",
        })
    }
}

fn extract_translated_text(body: &serde_json::Value) -> Option<String> {
    let segments = body.get(0)?.as_array()?;
    let mut out = String::new();
    for segment in segments {
        if let Some(piece) = segment.get(0).and_then(|v| v.as_str()) {
            out.push_str(piece);
        }
    }
    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn extract_detected_language(body: &serde_json::Value) -> Option<String> {
    body.get(2)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty() && s != "und")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_google_translate_payload() {
        let body = serde_json::json!([[["سلام", "hello", null, null, 10]], null, "en"]);
        assert_eq!(extract_translated_text(&body).as_deref(), Some("سلام"));
        assert_eq!(extract_detected_language(&body).as_deref(), Some("en"));
    }
}
