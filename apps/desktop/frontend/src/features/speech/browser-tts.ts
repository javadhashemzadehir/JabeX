/**
 * Browser Speech Synthesis fallback (Web Speech API).
 * Used when native Piper / Windows SAPI audio is unavailable.
 */

export type BrowserSpeakOptions = {
  text: string;
  language?: string;
  rate?: number;
  pitch?: number;
  voiceURI?: string | null;
};

export function browserTtsAvailable(): boolean {
  return typeof window !== "undefined" && "speechSynthesis" in window;
}

export function speakWithBrowser(options: BrowserSpeakOptions): Promise<void> {
  return new Promise((resolve, reject) => {
    if (!browserTtsAvailable()) {
      reject(new Error("Browser Speech API unavailable"));
      return;
    }
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(options.text);
    utterance.lang = cultureForLanguage(options.language ?? "en");
    utterance.rate = clamp(options.rate ?? 1, 0.5, 2);
    utterance.pitch = clamp(options.pitch ?? 1, 0.5, 2);
    if (options.voiceURI) {
      const voice = window.speechSynthesis
        .getVoices()
        .find((v) => v.voiceURI === options.voiceURI || v.name === options.voiceURI);
      if (voice) utterance.voice = voice;
    }
    utterance.onend = () => resolve();
    utterance.onerror = () => reject(new Error("Browser TTS failed"));
    window.speechSynthesis.speak(utterance);
  });
}

export function stopBrowserSpeech(): void {
  if (browserTtsAvailable()) {
    window.speechSynthesis.cancel();
  }
}

function cultureForLanguage(language: string): string {
  const lang = language.split(/[-_]/)[0]?.toLowerCase() ?? "en";
  switch (lang) {
    case "fa":
      return "fa-IR";
    case "fr":
      return "fr-FR";
    case "de":
      return "de-DE";
    case "es":
      return "es-ES";
    case "ar":
      return "ar-SA";
    case "ja":
      return "ja-JP";
    case "zh":
      return "zh-CN";
    default:
      return "en-US";
  }
}

function clamp(n: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, n));
}
