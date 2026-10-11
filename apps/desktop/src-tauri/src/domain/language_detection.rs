//! Offline language / script detection and Persian transliteration heuristics.
//!
//! Network providers may refine results later; this service never calls the network.

use super::language_assist::{
    LanguageDetectionResult, LanguageDetectionService, TransliterationMetadata,
};

/// Built-in Persian-script loanword → (foreign lemma, language) map.
/// Intentionally small; Google (when enabled) can refine unknowns.
const PERSIAN_TRANSLITERATION_MAP: &[(&str, &str, &str)] = &[
    ("کامپیوتر", "computer", "en"),
    ("اینترنت", "internet", "en"),
    ("موبایل", "mobile", "en"),
    ("تلفن", "telephone", "en"),
    ("ایمیل", "email", "en"),
    ("وبسایت", "website", "en"),
    ("سایت", "site", "en"),
    ("اپلیکیشن", "application", "en"),
    ("برنامه", "program", "en"),
    ("سیستم", "system", "en"),
    ("فایل", "file", "en"),
    ("فولدر", "folder", "en"),
    ("کیبورد", "keyboard", "en"),
    ("ماوس", "mouse", "en"),
    ("مانیتور", "monitor", "en"),
    ("پرینتر", "printer", "en"),
    ("اسکنر", "scanner", "en"),
    ("باتری", "battery", "en"),
    ("شارژر", "charger", "en"),
    ("هندزفری", "handsfree", "en"),
    ("هدفون", "headphone", "en"),
    ("اسپیکر", "speaker", "en"),
    ("میکروفون", "microphone", "en"),
    ("دوربین", "camera", "en"),
    ("پروژه", "project", "en"),
    ("دیزاین", "design", "en"),
    ("کد", "code", "en"),
    ("سرور", "server", "en"),
    ("کلاینت", "client", "en"),
    ("دیتابیس", "database", "en"),
    ("الگوریتم", "algorithm", "en"),
    ("پروتکل", "protocol", "en"),
    ("پاسورد", "password", "en"),
    ("یوزرنیم", "username", "en"),
    ("اکانت", "account", "en"),
    ("لاگین", "login", "en"),
    ("لاگ‌اوت", "logout", "en"),
    ("آپدیت", "update", "en"),
    ("دانلود", "download", "en"),
    ("آپلود", "upload", "en"),
    ("کپی", "copy", "en"),
    ("پیست", "paste", "en"),
    ("کات", "cut", "en"),
    ("سیو", "save", "en"),
    ("دیلیت", "delete", "en"),
    ("ریستارت", "restart", "en"),
    ("شات‌داون", "shutdown", "en"),
    ("وای‌فای", "wifi", "en"),
    ("بلوتوث", "bluetooth", "en"),
    ("جی‌پی‌اس", "gps", "en"),
    ("یو‌اس‌بی", "usb", "en"),
    ("اس‌ام‌اس", "sms", "en"),
    ("پی‌دی‌اف", "pdf", "en"),
    ("اکسل", "excel", "en"),
    ("ورد", "word", "en"),
    ("پاورپوینت", "powerpoint", "en"),
    ("گوگل", "google", "en"),
    ("یوتیوب", "youtube", "en"),
    ("اینستاگرام", "instagram", "en"),
    ("تلگرام", "telegram", "en"),
    ("واتساپ", "whatsapp", "en"),
    ("فیسبوک", "facebook", "en"),
    ("توییتر", "twitter", "en"),
    ("لینکدین", "linkedin", "en"),
    ("نتفلیکس", "netflix", "en"),
    ("اسپاتیفای", "spotify", "en"),
    ("آیفون", "iphone", "en"),
    ("اندروید", "android", "en"),
    ("ویندوز", "windows", "en"),
    ("لینوکس", "linux", "en"),
    ("مک", "mac", "en"),
    ("آفیس", "office", "en"),
    ("هتل", "hotel", "en"),
    ("تاکسی", "taxi", "en"),
    ("اتوبوس", "bus", "en"),
    ("مترو", "metro", "en"),
    ("پارک", "park", "en"),
    ("رستوران", "restaurant", "en"),
    ("کافه", "cafe", "en"),
    ("سوپرمارکت", "supermarket", "en"),
    ("بانک", "bank", "en"),
    ("پاسپورت", "passport", "en"),
    ("ویزا", "visa", "en"),
    ("تیکت", "ticket", "en"),
    ("رزرو", "reserve", "en"),
    ("آدرس", "address", "en"),
    ("ای‌دی", "id", "en"),
    ("پی‌کی", "pk", "en"),
];

/// Local heuristic detector — no I/O, no network.
#[derive(Debug, Default, Clone)]
pub struct HeuristicLanguageDetector;

impl LanguageDetectionService for HeuristicLanguageDetector {
    fn detect(&self, text: &str) -> LanguageDetectionResult {
        detect_language(text)
    }
}

pub fn detect_language(text: &str) -> LanguageDetectionResult {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return LanguageDetectionResult {
            language: "und".into(),
            confidence: 0.0,
            script: "zyyy".into(),
            transliteration: None,
        };
    }

    // Prefer the first line / token for vocabulary-style cards.
    let sample = primary_sample(trimmed);
    let script = dominant_script(sample);

    match script.as_str() {
        "arab" => detect_arabic_script(sample),
        "latn" => detect_latin(sample, script),
        "cyrl" => LanguageDetectionResult {
            language: "ru".into(),
            confidence: 0.5,
            script,
            transliteration: None,
        },
        "grek" => LanguageDetectionResult {
            language: "el".into(),
            confidence: 0.55,
            script,
            transliteration: None,
        },
        "hang" => LanguageDetectionResult {
            language: "ko".into(),
            confidence: 0.6,
            script,
            transliteration: None,
        },
        "hani" | "jpan" => LanguageDetectionResult {
            language: if script == "jpan" {
                "ja".into()
            } else {
                "zh".into()
            },
            confidence: 0.5,
            script,
            transliteration: None,
        },
        _ => LanguageDetectionResult {
            language: "und".into(),
            confidence: 0.2,
            script,
            transliteration: None,
        },
    }
}

fn primary_sample(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(text)
        .split_whitespace()
        .next()
        .unwrap_or(text)
}

fn dominant_script(text: &str) -> String {
    let mut arab = 0u32;
    let mut latn = 0u32;
    let mut cyrl = 0u32;
    let mut grek = 0u32;
    let mut hang = 0u32;
    let mut hani = 0u32;
    let mut hiragana_katakana = 0u32;

    for ch in text.chars() {
        if ch.is_whitespace() || ch.is_ascii_punctuation() || ch.is_ascii_digit() {
            continue;
        }
        match ch {
            '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB50}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}' => arab += 1,
            '\u{0400}'..='\u{04FF}' => cyrl += 1,
            '\u{0370}'..='\u{03FF}' => grek += 1,
            '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' => hang += 1,
            '\u{3040}'..='\u{30FF}' => hiragana_katakana += 1,
            '\u{4E00}'..='\u{9FFF}' => hani += 1,
            c if c.is_ascii_alphabetic() => latn += 1,
            _ => {}
        }
    }

    let max = arab
        .max(latn)
        .max(cyrl)
        .max(grek)
        .max(hang)
        .max(hani)
        .max(hiragana_katakana);
    if max == 0 {
        return "zyyy".into();
    }
    if hiragana_katakana == max || (hiragana_katakana > 0 && hani == max) {
        return "jpan".into();
    }
    if arab == max {
        "arab".into()
    } else if latn == max {
        "latn".into()
    } else if cyrl == max {
        "cyrl".into()
    } else if grek == max {
        "grek".into()
    } else if hang == max {
        "hang".into()
    } else if hani == max {
        "hani".into()
    } else {
        "zyyy".into()
    }
}

fn detect_latin(sample: &str, script: String) -> LanguageDetectionResult {
    let key = sample.to_ascii_lowercase();
    if let Some((lang, confidence)) = LATIN_LEXICON
        .iter()
        .find(|(w, _, _)| *w == key)
        .map(|(_, lang, conf)| (*lang, *conf))
    {
        return LanguageDetectionResult {
            language: lang.into(),
            confidence,
            script,
            transliteration: None,
        };
    }

    // Light diacritic / orthography hints.
    if key.chars().any(|c| "àâæçéèêëïîôœùûüÿ".contains(c)) {
        return LanguageDetectionResult {
            language: "fr".into(),
            confidence: 0.6,
            script,
            transliteration: None,
        };
    }
    if key.chars().any(|c| "äöüß".contains(c)) {
        return LanguageDetectionResult {
            language: "de".into(),
            confidence: 0.6,
            script,
            transliteration: None,
        };
    }
    if key.chars().any(|c| "ñ¿¡áéíóúü".contains(c)) {
        return LanguageDetectionResult {
            language: "es".into(),
            confidence: 0.55,
            script,
            transliteration: None,
        };
    }

    LanguageDetectionResult {
        language: "en".into(),
        confidence: 0.55,
        script,
        transliteration: None,
    }
}

/// High-frequency study words for offline Latin language hints.
const LATIN_LEXICON: &[(&str, &str, f64)] = &[
    ("bonjour", "fr", 0.9),
    ("merci", "fr", 0.85),
    ("oui", "fr", 0.7),
    ("non", "fr", 0.55),
    ("eau", "fr", 0.75),
    ("pomme", "fr", 0.85),
    ("hallo", "de", 0.85),
    ("danke", "de", 0.9),
    ("wasser", "de", 0.85),
    ("apfel", "de", 0.9),
    ("hola", "es", 0.85),
    ("gracias", "es", 0.9),
    ("agua", "es", 0.8),
    ("manzana", "es", 0.9),
    ("hello", "en", 0.85),
    ("apple", "en", 0.85),
    ("water", "en", 0.8),
    ("please", "en", 0.8),
    ("computer", "en", 0.85),
];

fn detect_arabic_script(sample: &str) -> LanguageDetectionResult {
    let normalized = normalize_persian(sample);
    if let Some((resolved, lang, confidence)) = lookup_transliteration(&normalized) {
        return LanguageDetectionResult {
            language: "fa".into(),
            confidence: 0.85,
            script: "arab".into(),
            transliteration: Some(TransliterationMetadata {
                is_transliteration: true,
                source_script: "arab".into(),
                resolved_word: Some(resolved),
                resolved_language: Some(lang),
                confidence,
            }),
        };
    }

    // Heuristic: short Arabic-script tokens with few native Persian vowels patterns
    // may still be loanwords — flag low-confidence transliteration for optional refine.
    let looks_like_loanword = is_likely_persian_loanword(&normalized);
    let transliteration = if looks_like_loanword {
        Some(TransliterationMetadata {
            is_transliteration: true,
            source_script: "arab".into(),
            resolved_word: None,
            resolved_language: None,
            confidence: 0.35,
        })
    } else {
        None
    };

    LanguageDetectionResult {
        language: "fa".into(),
        confidence: if looks_like_loanword { 0.6 } else { 0.75 },
        script: "arab".into(),
        transliteration,
    }
}

fn normalize_persian(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'ي' => 'ی',
            'ك' => 'ک',
            '‌' | '‍' => ' ', // ZWNJ / ZWJ → space for lookup flexibility
            other => other,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
}

fn lookup_transliteration(normalized: &str) -> Option<(String, String, f64)> {
    for (fa, en, lang) in PERSIAN_TRANSLITERATION_MAP {
        let key = normalize_persian(fa);
        if key == normalized {
            return Some(((*en).into(), (*lang).into(), 0.95));
        }
    }
    None
}

/// Very short Arabic-script words that are not common native function words
/// are treated as possible foreign transliterations.
fn is_likely_persian_loanword(normalized: &str) -> bool {
    let len = normalized.chars().count();
    if !(3..=20).contains(&len) {
        return false;
    }
    const NATIVE_STOPWORDS: &[&str] = &[
        "و",
        "در",
        "به",
        "از",
        "که",
        "این",
        "آن",
        "را",
        "با",
        "برای",
        "تا",
        "هم",
        "یا",
        "اما",
        "اگر",
        "چون",
        "چه",
        "کی",
        "کجا",
        "چرا",
        "من",
        "تو",
        "او",
        "ما",
        "شما",
        "ایشان",
        "است",
        "بود",
        "شد",
        "می",
        "نمی",
        "یک",
        "دو",
        "سه",
    ];
    if NATIVE_STOPWORDS.contains(&normalized) {
        return false;
    }
    // Presence of گ چ پ ژ strongly suggests Persian orthography (not Arabic).
    let has_persian_letters = normalized
        .chars()
        .any(|c| matches!(c, 'گ' | 'چ' | 'پ' | 'ژ'));
    has_persian_letters || len >= 5
}

/// Prefer the foreign lemma for pronunciation / translation when available.
pub fn study_lemma(detection: &LanguageDetectionResult, original: &str) -> (String, String) {
    if let Some(meta) = &detection.transliteration {
        if let (Some(word), Some(lang)) = (&meta.resolved_word, &meta.resolved_language) {
            return (word.clone(), lang.clone());
        }
    }
    (original.trim().to_string(), detection.language.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_french_bonjour() {
        let r = detect_language("bonjour");
        assert_eq!(r.language, "fr");
    }

    #[test]
    fn detects_latin_as_english() {
        let result = detect_language("hello");
        assert_eq!(result.language, "en");
        assert_eq!(result.script, "latn");
        assert!(result.transliteration.is_none());
    }

    #[test]
    fn detects_persian_native() {
        let result = detect_language("سلام");
        assert_eq!(result.language, "fa");
        assert_eq!(result.script, "arab");
    }

    #[test]
    fn resolves_known_transliteration() {
        let result = detect_language("کامپیوتر");
        let meta = result.transliteration.expect("transliteration");
        assert!(meta.is_transliteration);
        assert_eq!(meta.resolved_word.as_deref(), Some("computer"));
        assert_eq!(meta.resolved_language.as_deref(), Some("en"));
        assert!(meta.confidence > 0.9);
    }

    #[test]
    fn study_lemma_prefers_resolved_word() {
        let detection = detect_language("اینترنت");
        let (lemma, lang) = study_lemma(&detection, "اینترنت");
        assert_eq!(lemma, "internet");
        assert_eq!(lang, "en");
    }

    #[test]
    fn empty_is_undetermined() {
        let result = detect_language("   ");
        assert_eq!(result.language, "und");
    }
}
