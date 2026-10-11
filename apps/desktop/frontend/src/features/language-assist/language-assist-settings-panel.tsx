"use client";

import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { LanguageAssistSettings, TtsProviderKind, TtsVoiceInfo } from "@jabex/types";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Spinner } from "@/components/ui/spinner";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";

const DEFAULT_SETTINGS: LanguageAssistSettings = {
  autoTranslation: false,
  pronunciationGeneration: false,
  ttsProvider: "auto",
  translationProvider: "offline",
  targetLanguage: "fa",
  speed: 1,
  pitch: 1,
  preferredVoice: null,
  provider: "offline",
};

export function LanguageAssistSettingsPanel() {
  const t = useTranslations();
  const tauri = useIsTauri();
  const [settings, setSettings] = useState<LanguageAssistSettings>(DEFAULT_SETTINGS);
  const [voices, setVoices] = useState<TtsVoiceInfo[]>([]);
  const [voicesDir, setVoicesDir] = useState<string>("");
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);
  const loading = tauri && !loaded;

  useEffect(() => {
    if (!tauri) return;
    let cancelled = false;
    void (async () => {
      try {
        const [next, voiceList, dir] = await Promise.all([
          api.getLanguageAssistSettings(),
          api.listTtsVoices().catch(() => [] as TtsVoiceInfo[]),
          api.getSpeechVoicesDir().catch(() => ""),
        ]);
        if (!cancelled) {
          setSettings({ ...DEFAULT_SETTINGS, ...next });
          setVoices(voiceList);
          setVoicesDir(dir);
        }
      } catch (err) {
        if (!cancelled) {
          toast.error(
            err instanceof Error ? err.message : t.settings.error,
          );
        }
      } finally {
        if (!cancelled) setLoaded(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [tauri, t.settings.error]);

  async function persist(next: LanguageAssistSettings) {
    if (!tauri) return;
    setSaving(true);
    setSettings(next);
    try {
      const saved = await api.setLanguageAssistSettings(next);
      setSettings({ ...DEFAULT_SETTINGS, ...saved });
      toast.success(t.settings.saved);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.settings.error);
    } finally {
      setSaving(false);
    }
  }

  if (!tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.settings.desktopOnly}</p>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Spinner className="size-4" />
        {t.settings.loading}
      </div>
    );
  }

  const ttsProvider = settings.ttsProvider ?? "auto";

  return (
    <section id="language-assist" className="space-y-4 scroll-mt-6">
      <div className="space-y-1">
        <h3 className="text-lg font-semibold tracking-tight">
          {t.speech.settingsTitle}
        </h3>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.speech.settingsDescription}
        </p>
      </div>

      <div className="space-y-4 rounded-lg border border-border/60 p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="tts-provider">{t.speech.provider}</Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.speech.providerHint}
            </p>
          </div>
          <select
            id="tts-provider"
            className="h-9 rounded-md border border-input bg-background px-2 text-sm"
            value={ttsProvider}
            disabled={saving}
            onChange={(e) => {
              const provider = e.target.value as TtsProviderKind;
              void persist({ ...settings, ttsProvider: provider, provider: "offline" });
            }}
          >
            <option value="auto">{t.speech.providerAuto}</option>
            <option value="piper">{t.speech.providerPiper}</option>
            <option value="windows_sapi">{t.speech.providerSapi}</option>
            <option value="browser">{t.speech.providerBrowser}</option>
          </select>
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="tts-voice">{t.speech.voice}</Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.speech.voiceHint}
            </p>
          </div>
          <select
            id="tts-voice"
            className="h-9 max-w-56 rounded-md border border-input bg-background px-2 text-sm"
            value={settings.preferredVoice ?? ""}
            disabled={saving}
            onChange={(e) => {
              const preferredVoice = e.target.value || null;
              void persist({ ...settings, preferredVoice });
            }}
          >
            <option value="">{t.speech.voiceDefault}</option>
            {voices.map((v) => (
              <option key={`${v.provider}-${v.id}`} value={v.id}>
                {v.name} ({v.language})
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="tts-speed">{t.speech.speed}</Label>
          </div>
          <select
            id="tts-speed"
            className="h-9 rounded-md border border-input bg-background px-2 text-sm"
            value={String(settings.speed ?? 1)}
            disabled={saving}
            onChange={(e) => {
              void persist({ ...settings, speed: Number(e.target.value) });
            }}
          >
            <option value="0.5">0.5x</option>
            <option value="0.75">0.75x</option>
            <option value="1">1x</option>
            <option value="1.25">1.25x</option>
          </select>
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="lang-auto-translate">
              {t.languageAssist.autoTranslation}
            </Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.speech.autoTranslateHint}
            </p>
          </div>
          <Switch
            id="lang-auto-translate"
            checked={settings.autoTranslation}
            disabled={saving}
            onCheckedChange={(checked) => {
              void persist({
                ...settings,
                autoTranslation: checked,
                translationProvider: "offline",
              });
            }}
          />
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="lang-pronunciation">
              {t.speech.autoGenerateAudio}
            </Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.speech.autoGenerateAudioHint}
            </p>
          </div>
          <Switch
            id="lang-pronunciation"
            checked={settings.pronunciationGeneration}
            disabled={saving}
            onCheckedChange={(checked) => {
              void persist({ ...settings, pronunciationGeneration: checked });
            }}
          />
        </div>

        {voicesDir ? (
          <p className="text-xs text-muted-foreground text-pretty">
            {t.speech.voicesDirHint.replace("{path}", voicesDir)}
          </p>
        ) : null}

        <p className="text-xs text-muted-foreground text-pretty">
          {t.speech.offlineNote}
        </p>
      </div>
    </section>
  );
}
