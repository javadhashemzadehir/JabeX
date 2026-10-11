//! Offline IPA / pronunciation hints (no network).

use serde::{Deserialize, Serialize};

/// Pronunciation hint shown in the card editor / review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationHint {
    pub text: String,
    pub language: String,
    /// IPA string when known (e.g. `/ˈæpəl/`).
    pub ipa: Option<String>,
    pub source: &'static str,
}

/// Look up or approximate an IPA hint for study text.
pub fn lookup_ipa(text: &str, language: &str) -> PronunciationHint {
    let trimmed = text.trim();
    let lang = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase();

    if trimmed.is_empty() {
        return PronunciationHint {
            text: String::new(),
            language: lang,
            ipa: None,
            source: "empty",
        };
    }

    if lang == "en" || lang == "fr" {
        let key = trimmed.to_ascii_lowercase();
        if let Some(ipa) = english_ipa(&key) {
            return PronunciationHint {
                text: trimmed.to_string(),
                language: lang,
                ipa: Some(ipa.to_string()),
                source: "dictionary",
            };
        }
    }

    // Fallback: show the lemma itself as the pronunciation guide.
    PronunciationHint {
        text: trimmed.to_string(),
        language: lang,
        ipa: None,
        source: "lemma",
    }
}

fn english_ipa(word: &str) -> Option<&'static str> {
    ENGLISH_IPA
        .iter()
        .find(|(w, _)| *w == word)
        .map(|(_, ipa)| *ipa)
}

/// Compact high-frequency English / common loanword IPA subset.
const ENGLISH_IPA: &[(&str, &str)] = &[
    ("apple", "/ˈæpəl/"),
    ("hello", "/həˈloʊ/"),
    ("world", "/wɜːrld/"),
    ("computer", "/kəmˈpjuːtər/"),
    ("language", "/ˈlæŋɡwɪdʒ/"),
    ("learn", "/lɜːrn/"),
    ("study", "/ˈstʌdi/"),
    ("book", "/bʊk/"),
    ("card", "/kɑːrd/"),
    ("water", "/ˈwɔːtər/"),
    ("friend", "/frɛnd/"),
    ("house", "/haʊs/"),
    ("school", "/skuːl/"),
    ("time", "/taɪm/"),
    ("good", "/ɡʊd/"),
    ("morning", "/ˈmɔːrnɪŋ/"),
    ("night", "/naɪt/"),
    ("thank", "/θæŋk/"),
    ("please", "/pliːz/"),
    ("sorry", "/ˈsɑːri/"),
    ("yes", "/jɛs/"),
    ("no", "/noʊ/"),
    ("cat", "/kæt/"),
    ("dog", "/dɔːɡ/"),
    ("food", "/fuːd/"),
    ("love", "/lʌv/"),
    ("work", "/wɜːrk/"),
    ("play", "/pleɪ/"),
    ("read", "/riːd/"),
    ("write", "/raɪt/"),
    ("speak", "/spiːk/"),
    ("listen", "/ˈlɪsən/"),
    ("bonjour", "/bɔ̃ʒuʁ/"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_dictionary_hit() {
        let hint = lookup_ipa("Apple", "en");
        assert_eq!(hint.ipa.as_deref(), Some("/ˈæpəl/"));
        assert_eq!(hint.source, "dictionary");
    }

    #[test]
    fn unknown_falls_back_to_lemma() {
        let hint = lookup_ipa("xyzzy", "en");
        assert!(hint.ipa.is_none());
        assert_eq!(hint.text, "xyzzy");
    }
}
