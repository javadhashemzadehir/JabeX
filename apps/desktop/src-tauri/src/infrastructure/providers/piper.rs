//! Piper TTS provider — local binary + optional voice models (no cloud).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::speech::{AudioFile, TtsProvider, TtsProviderKind, TtsRequest, TtsVoiceInfo};
use crate::infrastructure::process;

/// Shared cancel flag for the in-flight Piper job (best-effort).
#[derive(Debug, Default)]
pub struct PiperCancelFlag {
    cancelled: AtomicBool,
}

impl PiperCancelFlag {
    pub fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct PiperProvider {
    /// Directory containing `piper` / `piper.exe` (optional; also searches PATH).
    runtime_dir: PathBuf,
    /// `{root}/voices/{lang}/…onnx`
    voices_dir: PathBuf,
    cancel: Arc<PiperCancelFlag>,
    /// Simple single-slot queue state for status/tests.
    busy: Arc<Mutex<bool>>,
}

impl PiperProvider {
    pub fn new(runtime_dir: PathBuf, voices_dir: PathBuf) -> Self {
        Self {
            runtime_dir,
            voices_dir,
            cancel: Arc::new(PiperCancelFlag::new()),
            busy: Arc::new(Mutex::new(false)),
        }
    }

    pub fn from_app_root(app_root: &Path) -> Self {
        let runtime_dir = app_root.join("data").join("piper");
        let voices_dir = app_root.join("data").join("voices");
        Self::new(runtime_dir, voices_dir)
    }

    pub fn cancel_flag(&self) -> Arc<PiperCancelFlag> {
        self.cancel.clone()
    }

    pub fn cancel_current(&self) {
        self.cancel.cancel();
    }

    pub fn voices_dir(&self) -> &Path {
        &self.voices_dir
    }

    fn resolve_binary(&self) -> Option<PathBuf> {
        let candidates = [
            self.runtime_dir.join("piper.exe"),
            self.runtime_dir.join("piper"),
            PathBuf::from("piper.exe"),
            PathBuf::from("piper"),
        ];
        for path in candidates {
            if path.is_file() {
                return Some(path);
            }
            // PATH lookup for bare names
            if path.components().count() == 1 {
                if let Ok(output) = process::command(if cfg!(windows) { "where" } else { "which" })
                    .arg(path.as_os_str())
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Some(first) = stdout.lines().next() {
                            let found = PathBuf::from(first.trim());
                            if found.is_file() {
                                return Some(found);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn find_model(&self, language: &str, voice: Option<&str>) -> Option<PathBuf> {
        let lang = language
            .split(['-', '_'])
            .next()
            .unwrap_or(language)
            .to_ascii_lowercase();
        let lang_dir = self.voices_dir.join(&lang);
        if !lang_dir.is_dir() {
            return None;
        }

        if let Some(v) = voice {
            let named = lang_dir.join(format!("{v}.onnx"));
            if named.is_file() {
                return Some(named);
            }
            let nested = lang_dir.join(v).with_extension("onnx");
            if nested.is_file() {
                return Some(nested);
            }
        }

        // First *.onnx in the language folder (and one level of subdirs).
        if let Ok(entries) = std::fs::read_dir(&lang_dir) {
            let mut onnx_files: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case("onnx"))
                })
                .collect();
            onnx_files.sort();
            if let Some(first) = onnx_files.into_iter().next() {
                return Some(first);
            }
            // Subdirectories
            if let Ok(entries) = std::fs::read_dir(&lang_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(inner) = std::fs::read_dir(&path) {
                            let mut files: Vec<PathBuf> = inner
                                .flatten()
                                .map(|e| e.path())
                                .filter(|p| {
                                    p.extension()
                                        .and_then(|e| e.to_str())
                                        .is_some_and(|e| e.eq_ignore_ascii_case("onnx"))
                                })
                                .collect();
                            files.sort();
                            if let Some(first) = files.into_iter().next() {
                                return Some(first);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn temp_wav_path() -> JabeXResult<PathBuf> {
        let mut dir = std::env::temp_dir();
        dir.push("jabex-tts");
        std::fs::create_dir_all(&dir).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create temp TTS dir: {e}"))
        })?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Ok(dir.join(format!("piper-{nanos}.wav")))
    }
}

impl TtsProvider for PiperProvider {
    fn id(&self) -> &'static str {
        "piper"
    }

    fn is_available(&self) -> bool {
        self.resolve_binary().is_some()
    }

    fn supports_language(&self, language: &str) -> bool {
        self.find_model(language, None).is_some()
    }

    fn synthesize(&self, request: &TtsRequest) -> JabeXResult<AudioFile> {
        let text = request.text.trim();
        if text.is_empty() {
            return Err(JabeXError::Validation("TTS text is empty".into()));
        }

        self.cancel.reset();
        {
            let mut busy = self.busy.lock();
            *busy = true;
        }

        let result = (|| {
            let binary = self.resolve_binary().ok_or_else(|| {
                JabeXError::Validation(
                    "Piper runtime not found. Install Piper under data/piper or PATH.".into(),
                )
            })?;
            let model = self
                .find_model(&request.language, request.voice.as_deref())
                .ok_or_else(|| {
                    JabeXError::Validation(format!(
                        "No Piper voice model for language '{}'. Download a model into data/voices/{}/",
                        request.language,
                        request
                            .language
                            .split(['-', '_'])
                            .next()
                            .unwrap_or(&request.language)
                    ))
                })?;

            if self.cancel.is_cancelled() {
                return Err(JabeXError::Validation("Piper generation cancelled".into()));
            }

            let out = Self::temp_wav_path()?;
            let length_scale = if request.speed > 0.0 {
                1.0 / request.speed
            } else {
                1.0
            };

            let mut child = process::command(&binary)
                .arg("--model")
                .arg(&model)
                .arg("--output_file")
                .arg(&out)
                .arg("--length_scale")
                .arg(format!("{length_scale:.3}"))
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| JabeXError::Internal(format!("failed to start Piper: {e}")))?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(text.as_bytes())
                    .map_err(|e| JabeXError::Internal(format!("Piper stdin failed: {e}")))?;
            }

            if self.cancel.is_cancelled() {
                let _ = child.kill();
                let _ = std::fs::remove_file(&out);
                return Err(JabeXError::Validation("Piper generation cancelled".into()));
            }

            let output = child
                .wait_with_output()
                .map_err(|e| JabeXError::Internal(format!("Piper wait failed: {e}")))?;

            if self.cancel.is_cancelled() {
                let _ = std::fs::remove_file(&out);
                return Err(JabeXError::Validation("Piper generation cancelled".into()));
            }

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = std::fs::remove_file(&out);
                return Err(JabeXError::Internal(format!("Piper failed: {stderr}")));
            }

            let bytes = std::fs::read(&out)
                .map_err(|e| JabeXError::Filesystem(format!("failed to read Piper wav: {e}")))?;
            let _ = std::fs::remove_file(&out);
            if bytes.len() < 44 {
                return Err(JabeXError::Internal(
                    "Piper produced an empty audio file".into(),
                ));
            }

            let voice_name = model
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());

            Ok(AudioFile::wav(bytes, "piper", voice_name))
        })();

        *self.busy.lock() = false;
        result
    }

    fn list_voices(&self) -> JabeXResult<Vec<TtsVoiceInfo>> {
        let mut voices = Vec::new();
        if !self.voices_dir.is_dir() {
            return Ok(voices);
        }
        let entries = std::fs::read_dir(&self.voices_dir).map_err(|e| {
            JabeXError::Filesystem(format!("failed to read voices dir: {e}"))
        })?;
        for entry in entries.flatten() {
            let lang_path = entry.path();
            if !lang_path.is_dir() {
                continue;
            }
            let language = lang_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("und")
                .to_string();
            let collect = |dir: &Path, language: &str, voices: &mut Vec<TtsVoiceInfo>| {
                if let Ok(inner) = std::fs::read_dir(dir) {
                    for file in inner.flatten() {
                        let path = file.path();
                        if path
                            .extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case("onnx"))
                        {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("voice")
                                .to_string();
                            voices.push(TtsVoiceInfo {
                                id: name.clone(),
                                name,
                                language: language.to_string(),
                                provider: TtsProviderKind::Piper,
                            });
                        }
                    }
                }
            };
            collect(&lang_path, &language, &mut voices);
            if let Ok(sub) = std::fs::read_dir(&lang_path) {
                for sub_entry in sub.flatten() {
                    let path = sub_entry.path();
                    if path.is_dir() {
                        collect(&path, &language, &mut voices);
                    }
                }
            }
        }
        voices.sort_by(|a, b| (&a.language, &a.name).cmp(&(&b.language, &b.name)));
        Ok(voices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn unavailable_without_binary() {
        let dir = tempdir().expect("temp");
        let p = PiperProvider::new(dir.path().join("missing"), dir.path().join("voices"));
        assert!(!p.is_available());
    }

    #[test]
    fn finds_model_in_lang_folder() {
        let dir = tempdir().expect("temp");
        let voices = dir.path().join("voices").join("en");
        std::fs::create_dir_all(&voices).expect("mkdir");
        let model = voices.join("en_US-test.onnx");
        std::fs::write(&model, b"fake").expect("write");
        let p = PiperProvider::new(dir.path().join("piper"), dir.path().join("voices"));
        assert!(p.supports_language("en"));
        assert_eq!(p.find_model("en", None), Some(model));
    }
}
