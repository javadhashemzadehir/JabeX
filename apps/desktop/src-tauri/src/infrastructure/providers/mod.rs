pub mod google;
pub mod noop;

pub use google::{GoogleSpeechProvider, GoogleTranslationProvider};
pub use noop::{NoopSpeechProvider, NoopTranslationProvider};
