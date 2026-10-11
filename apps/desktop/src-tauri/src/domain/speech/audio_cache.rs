//! Deterministic audio cache keys (SHA-256).

use sha2::{Digest, Sha256};

/// Inputs that uniquely identify a synthesized utterance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCacheKey {
    pub text: String,
    pub language: String,
    pub voice: String,
    pub speed: String,
}

impl AudioCacheKey {
    pub fn new(text: &str, language: &str, voice: Option<&str>, speed: f32) -> Self {
        Self {
            text: text.trim().to_string(),
            language: normalize_lang(language),
            voice: voice.unwrap_or("default").trim().to_string(),
            speed: format!("{speed:.2}"),
        }
    }

    pub fn hash(&self) -> String {
        audio_cache_hash(&self.text, &self.language, Some(&self.voice), &self.speed)
    }
}

/// SHA-256 hex of `text + language + voice + speed`.
pub fn audio_cache_hash(text: &str, language: &str, voice: Option<&str>, speed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.trim().as_bytes());
    hasher.update(b"|");
    hasher.update(normalize_lang(language).as_bytes());
    hasher.update(b"|");
    hasher.update(voice.unwrap_or("default").trim().as_bytes());
    hasher.update(b"|");
    hasher.update(speed.as_bytes());
    hex::encode(hasher.finalize())
}

fn normalize_lang(language: &str) -> String {
    language
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_for_same_inputs() {
        let a = audio_cache_hash("bonjour", "fr", Some("fr-FR"), "1.00");
        let b = audio_cache_hash("bonjour", "fr", Some("fr-FR"), "1.00");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn hash_changes_with_speed_or_voice() {
        let base = audio_cache_hash("apple", "en", Some("default"), "1.00");
        let speed = audio_cache_hash("apple", "en", Some("default"), "0.50");
        let voice = audio_cache_hash("apple", "en", Some("other"), "1.00");
        assert_ne!(base, speed);
        assert_ne!(base, voice);
    }

    #[test]
    fn language_region_normalizes() {
        let a = AudioCacheKey::new("hi", "en-US", None, 1.0).hash();
        let b = AudioCacheKey::new("hi", "en_GB", None, 1.0).hash();
        assert_eq!(a, b);
    }
}
