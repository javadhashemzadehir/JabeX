//! Offline translation: SQLite cache → bundled dictionary packs → optional Argos CLI.
//!
//! Never calls the network. Argos is only used when the user has installed the
//! `argos-translate` CLI locally.

use std::collections::HashMap;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::language_assist::{TranslationProvider, TranslationResult};
use crate::domain::speech::TranslationCacheEntry;
use crate::infrastructure::process;
use crate::infrastructure::repositories::SqliteTranslationCacheRepository;
use crate::shared::ids::create_id;
use crate::shared::time::now_utc;

/// Combines cache + dictionary + optional Argos.
pub struct OfflineTranslationProvider {
    cache: SqliteTranslationCacheRepository,
    dictionary: OfflineDictionary,
    /// When true, try `argos-translate` on cache/dictionary miss.
    try_argos: bool,
}

impl OfflineTranslationProvider {
    pub fn new(cache: SqliteTranslationCacheRepository) -> Self {
        Self {
            cache,
            dictionary: OfflineDictionary::bundled(),
            try_argos: true,
        }
    }

    fn normalize_lang(language: &str) -> String {
        language
            .split(['-', '_'])
            .next()
            .unwrap_or(language)
            .trim()
            .to_ascii_lowercase()
    }

    fn lookup_cache(&self, text: &str, source: &str, target: &str) -> JabeXResult<Option<String>> {
        Ok(self
            .cache
            .find(text, source, target)?
            .map(|e| e.translated_text))
    }

    fn store_cache(
        &self,
        text: &str,
        source: &str,
        target: &str,
        translated: &str,
    ) -> JabeXResult<()> {
        let now = now_utc();
        let entry = TranslationCacheEntry {
            id: create_id(),
            source_text: text.to_string(),
            source_language: source.to_string(),
            target_language: target.to_string(),
            translated_text: translated.to_string(),
            created_at: now,
        };
        self.cache.upsert(&entry)
    }

    fn try_argos(text: &str, source: &str, target: &str) -> Option<String> {
        let output = process::command("argos-translate")
            .args(["--from-lang", source, "--to-lang", target, text])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let translated = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if translated.is_empty() || translated.eq_ignore_ascii_case(text) {
            None
        } else {
            Some(translated)
        }
    }
}

impl TranslationProvider for OfflineTranslationProvider {
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

        let source = Self::normalize_lang(source_language.unwrap_or("auto"));
        let target = Self::normalize_lang(target_language);
        if source == target {
            return Ok(TranslationResult {
                translated_text: trimmed.to_string(),
                detected_source_language: Some(source),
                provider: "offline",
            });
        }

        let cache_source = if source == "auto" { "und" } else { &source };

        if let Some(hit) = self.lookup_cache(trimmed, cache_source, &target)? {
            return Ok(TranslationResult {
                translated_text: hit,
                detected_source_language: Some(cache_source.to_string()),
                provider: "cache",
            });
        }

        // Also try lowercase key for Latin text.
        let lower = trimmed.to_ascii_lowercase();
        if lower != trimmed {
            if let Some(hit) = self.lookup_cache(&lower, cache_source, &target)? {
                return Ok(TranslationResult {
                    translated_text: hit,
                    detected_source_language: Some(cache_source.to_string()),
                    provider: "cache",
                });
            }
        }

        if let Some((translated, detected)) = self
            .dictionary
            .lookup(
                trimmed,
                if source == "auto" {
                    None
                } else {
                    Some(&source)
                },
                &target,
            )
            .or_else(|| {
                // Mis-detected Latin source: still allow dictionary hit.
                if source != "auto" {
                    self.dictionary.lookup(trimmed, None, &target)
                } else {
                    None
                }
            })
        {
            let detected = detected.unwrap_or_else(|| cache_source.to_string());
            let _ = self.store_cache(trimmed, &detected, &target, &translated);
            return Ok(TranslationResult {
                translated_text: translated,
                detected_source_language: Some(detected),
                provider: "dictionary",
            });
        }

        if self.try_argos && source != "auto" {
            if let Some(translated) = Self::try_argos(trimmed, &source, &target) {
                let _ = self.store_cache(trimmed, &source, &target, &translated);
                return Ok(TranslationResult {
                    translated_text: translated,
                    detected_source_language: Some(source),
                    provider: "argos",
                });
            }
        }

        Err(JabeXError::Validation(format!(
            "No offline translation for '{trimmed}' ({source}→{target}). Add a dictionary pack or install Argos Translate."
        )))
    }
}

/// In-memory bilingual packs for common flashcard vocabulary.
#[derive(Debug, Default, Clone)]
pub struct OfflineDictionary {
    /// key: "src|tgt|lemma" → translation
    entries: HashMap<String, String>,
}

impl OfflineDictionary {
    pub fn bundled() -> Self {
        let mut dict = Self::default();
        for (src, tgt, from, to) in BUNDLED_PAIRS {
            dict.insert(from, to, src, tgt);
            // reverse for study convenience
            dict.insert(to, from, tgt, src);
        }
        dict
    }

    pub fn insert(&mut self, from: &str, to: &str, source: &str, target: &str) {
        let key = format!(
            "{}|{}|{}",
            from.to_ascii_lowercase(),
            to.to_ascii_lowercase(),
            source.trim().to_ascii_lowercase()
        );
        self.entries.insert(key, target.trim().to_string());
    }

    pub fn lookup(
        &self,
        text: &str,
        source: Option<&str>,
        target: &str,
    ) -> Option<(String, Option<String>)> {
        let lemma = text.trim().to_ascii_lowercase();
        let target = target.to_ascii_lowercase();

        if let Some(src) = source {
            let key = format!("{}|{}|{}", src.to_ascii_lowercase(), target, lemma);
            if let Some(hit) = self.entries.get(&key) {
                return Some((hit.clone(), Some(src.to_string())));
            }
        } else {
            for lang in ["en", "fr", "de", "es", "fa", "ar", "ja", "zh"] {
                let key = format!("{lang}|{target}|{lemma}");
                if let Some(hit) = self.entries.get(&key) {
                    return Some((hit.clone(), Some(lang.to_string())));
                }
            }
        }
        None
    }
}

/// Compact starter pack (FR/EN/DE/ES ↔ FA and common EN↔FA).
const BUNDLED_PAIRS: &[(&str, &str, &str, &str)] = &[
    ("bonjour", "سلام", "fr", "fa"),
    ("merci", "متشکرم", "fr", "fa"),
    ("oui", "بله", "fr", "fa"),
    ("non", "نه", "fr", "fa"),
    ("eau", "آب", "fr", "fa"),
    ("pomme", "سیب", "fr", "fa"),
    ("hello", "سلام", "en", "fa"),
    ("thank you", "متشکرم", "en", "fa"),
    ("please", "لطفاً", "en", "fa"),
    ("yes", "بله", "en", "fa"),
    ("no", "نه", "en", "fa"),
    ("water", "آب", "en", "fa"),
    ("apple", "سیب", "en", "fa"),
    ("book", "کتاب", "en", "fa"),
    ("school", "مدرسه", "en", "fa"),
    ("friend", "دوست", "en", "fa"),
    ("house", "خانه", "en", "fa"),
    ("time", "زمان", "en", "fa"),
    ("good morning", "صبح بخیر", "en", "fa"),
    ("good night", "شب بخیر", "en", "fa"),
    ("computer", "کامپیوتر", "en", "fa"),
    ("language", "زبان", "en", "fa"),
    ("learn", "یاد گرفتن", "en", "fa"),
    ("study", "مطالعه", "en", "fa"),
    ("speak", "صحبت کردن", "en", "fa"),
    ("listen", "گوش دادن", "en", "fa"),
    ("read", "خواندن", "en", "fa"),
    ("write", "نوشتن", "en", "fa"),
    ("hallo", "سلام", "de", "fa"),
    ("danke", "متشکرم", "de", "fa"),
    ("wasser", "آب", "de", "fa"),
    ("apfel", "سیب", "de", "fa"),
    ("hola", "سلام", "es", "fa"),
    ("gracias", "متشکرم", "es", "fa"),
    ("agua", "آب", "es", "fa"),
    ("manzana", "سیب", "es", "fa"),
    ("سلام", "hello", "fa", "en"),
    ("متشکرم", "thank you", "fa", "en"),
    ("کتاب", "book", "fa", "en"),
    ("آب", "water", "fa", "en"),
    ("سیب", "apple", "fa", "en"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::open_database_at;
    use tempfile::tempdir;

    #[test]
    fn dictionary_bonjour_to_persian() {
        let dict = OfflineDictionary::bundled();
        let (text, src) = dict.lookup("Bonjour", Some("fr"), "fa").expect("hit");
        assert_eq!(text, "سلام");
        assert_eq!(src.as_deref(), Some("fr"));
    }

    #[test]
    fn cache_roundtrip() {
        let dir = tempdir().expect("temp");
        let db = open_database_at(dir.path()).expect("db");
        let cache = SqliteTranslationCacheRepository::new(db.clone());
        let provider = OfflineTranslationProvider::new(cache);
        let first = provider
            .translate("bonjour", Some("fr"), "fa")
            .expect("translate");
        assert_eq!(first.translated_text, "سلام");
        assert_eq!(first.provider, "dictionary");

        let second = provider
            .translate("bonjour", Some("fr"), "fa")
            .expect("cached");
        assert_eq!(second.translated_text, "سلام");
        assert_eq!(second.provider, "cache");
    }
}
