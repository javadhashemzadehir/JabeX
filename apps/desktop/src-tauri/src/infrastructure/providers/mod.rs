pub mod cascading_tts;
pub mod google;
pub mod noop;
pub mod offline_translation;
pub mod piper;
pub mod windows_sapi;

pub use cascading_tts::{BrowserDeferredSpeechProvider, CascadingTtsProvider};
pub use google::{GoogleSpeechProvider, GoogleTranslationProvider};
pub use noop::{NoopSpeechProvider, NoopTranslationProvider};
pub use offline_translation::OfflineTranslationProvider;
pub use piper::PiperProvider;
pub use windows_sapi::WindowsSapiProvider;
