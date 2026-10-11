//! Windows SAPI TTS via System.Speech (no install, no network).
//!
//! Uses PowerShell + System.Speech.Synthesis so we avoid heavy COM crates while
//! remaining compatible with voices already installed on Windows.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::speech::{AudioFile, TtsProvider, TtsProviderKind, TtsRequest, TtsVoiceInfo};
use crate::infrastructure::process;

#[derive(Debug, Default, Clone)]
pub struct WindowsSapiProvider;

impl WindowsSapiProvider {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    fn temp_wav_path() -> JabeXResult<std::path::PathBuf> {
        let mut dir = std::env::temp_dir();
        dir.push("jabex-tts");
        std::fs::create_dir_all(&dir).map_err(|e| {
            JabeXError::Filesystem(format!("failed to create temp TTS dir: {e}"))
        })?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Ok(dir.join(format!("sapi-{nanos}.wav")))
    }

    #[cfg(windows)]
    fn rate_from_speed(speed: f32) -> i32 {
        // SAPI rate is -10..=10. Map 0.5→-5, 1.0→0, 1.25→2.5≈3, 2.0→10.
        (((speed - 1.0) * 10.0).round() as i32).clamp(-10, 10)
    }

    #[cfg(windows)]
    fn pitch_prosody(pitch: f32) -> String {
        // SSML pitch: map 0.5→-50%, 1.0→+0%, 2.0→+50%.
        let pct = (((pitch - 1.0) * 50.0).round() as i32).clamp(-50, 50);
        format!("{pct:+}%")
    }
}

impl TtsProvider for WindowsSapiProvider {
    fn id(&self) -> &'static str {
        "windows_sapi"
    }

    fn is_available(&self) -> bool {
        cfg!(windows)
    }

    fn supports_language(&self, language: &str) -> bool {
        let lang = language
            .split(['-', '_'])
            .next()
            .unwrap_or(language)
            .to_ascii_lowercase();
        matches!(
            lang.as_str(),
            "en" | "fa" | "fr" | "de" | "es" | "ar" | "ja" | "zh" | "und"
        )
    }

    fn synthesize(&self, request: &TtsRequest) -> JabeXResult<AudioFile> {
        #[cfg(not(windows))]
        {
            let _ = request;
            return Err(JabeXError::Validation(
                "Windows SAPI is only available on Windows".into(),
            ));
        }

        #[cfg(windows)]
        {
            let text = request.text.trim();
            if text.is_empty() {
                return Err(JabeXError::Validation("TTS text is empty".into()));
            }
            if text.chars().count() > 2000 {
                return Err(JabeXError::Validation(
                    "TTS text is too long (max 2000 characters)".into(),
                ));
            }

            let out = Self::temp_wav_path()?;
            let out_str = out.to_string_lossy().replace('\'', "''");
            let escaped = text.replace('\'', "''");
            let culture = culture_for_language(&request.language);
            let rate = Self::rate_from_speed(request.speed);
            let pitch = Self::pitch_prosody(request.pitch);
            let voice_select = match &request.voice {
                Some(v) if !v.trim().is_empty() => {
                    let v = v.replace('\'', "''");
                    format!("$synth.SelectVoice('{v}')")
                }
                _ => format!(
                    "$synth.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::NotSet, [System.Speech.Synthesis.VoiceAge]::NotSet, 0, (New-Object System.Globalization.CultureInfo '{culture}'))"
                ),
            };

            let script = format!(
                r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {{
  {voice_select}
}} catch {{}}
$synth.Rate = {rate}
$synth.SetOutputToWaveFile('{out_str}')
$ssml = @"
<speak version='1.0' xml:lang='{culture}'>
  <prosody pitch='{pitch}'>{escaped}</prosody>
</speak>
"@
try {{
  $synth.SpeakSsml($ssml)
}} catch {{
  $synth.Speak('{escaped}')
}}
$synth.Dispose()
"#
            );

            let output = process::command("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-WindowStyle",
                    "Hidden",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &script,
                ])
                .output()
                .map_err(|e| JabeXError::Internal(format!("SAPI PowerShell failed: {e}")))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = std::fs::remove_file(&out);
                return Err(JabeXError::Internal(format!(
                    "Windows SAPI synthesis failed: {stderr}"
                )));
            }

            let bytes = std::fs::read(&out).map_err(|e| {
                JabeXError::Filesystem(format!("failed to read SAPI wav: {e}"))
            })?;
            let _ = std::fs::remove_file(&out);
            if bytes.len() < 44 {
                return Err(JabeXError::Internal(
                    "Windows SAPI produced an empty audio file".into(),
                ));
            }

            Ok(AudioFile::wav(
                bytes,
                "windows_sapi",
                request.voice.clone(),
            ))
        }
    }

    fn list_voices(&self) -> JabeXResult<Vec<TtsVoiceInfo>> {
        #[cfg(not(windows))]
        {
            return Ok(Vec::new());
        }

        #[cfg(windows)]
        {
            let script = r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.GetInstalledVoices() | ForEach-Object {
  $i = $_.VoiceInfo
  '{0}|{1}|{2}' -f $i.Name, $i.Culture.Name, $i.Name
}
$synth.Dispose()
"#;
            let output = process::command("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-WindowStyle",
                    "Hidden",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| JabeXError::Internal(format!("SAPI voice list failed: {e}")))?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut voices = Vec::new();
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(3, '|');
                let name = parts.next().unwrap_or("").to_string();
                let culture = parts.next().unwrap_or("en-US").to_string();
                let id = parts.next().unwrap_or(&name).to_string();
                if name.is_empty() {
                    continue;
                }
                let language = culture
                    .split(['-', '_'])
                    .next()
                    .unwrap_or("en")
                    .to_ascii_lowercase();
                voices.push(TtsVoiceInfo {
                    id,
                    name,
                    language,
                    provider: TtsProviderKind::WindowsSapi,
                });
            }
            Ok(voices)
        }
    }
}

fn culture_for_language(language: &str) -> &'static str {
    match language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase()
        .as_str()
    {
        "fa" | "per" => "fa-IR",
        "fr" => "fr-FR",
        "de" => "de-DE",
        "es" => "es-ES",
        "ar" => "ar-SA",
        "ja" => "ja-JP",
        "zh" => "zh-CN",
        _ => "en-US",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_core_languages() {
        let p = WindowsSapiProvider::new();
        assert!(p.supports_language("en"));
        assert!(p.supports_language("fa-IR"));
        assert!(p.supports_language("ja"));
    }

    #[test]
    fn rate_mapping() {
        #[cfg(windows)]
        {
            assert_eq!(WindowsSapiProvider::rate_from_speed(1.0), 0);
            assert_eq!(WindowsSapiProvider::rate_from_speed(0.5), -5);
        }
    }
}
