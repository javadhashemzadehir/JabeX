"use client";

import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { LanguageAssistSettings } from "@jabex/types";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Spinner } from "@/components/ui/spinner";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";

const DEFAULT_SETTINGS: LanguageAssistSettings = {
  autoTranslation: false,
  pronunciationGeneration: false,
  provider: "none",
  targetLanguage: "fa",
};

export function LanguageAssistSettingsPanel() {
  const t = useTranslations();
  const tauri = useIsTauri();
  const [settings, setSettings] = useState<LanguageAssistSettings>(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);
  const loading = tauri && !loaded;

  useEffect(() => {
    if (!tauri) return;
    let cancelled = false;
    void (async () => {
      try {
        const next = await api.getLanguageAssistSettings();
        if (!cancelled) setSettings(next);
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
      setSettings(saved);
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

  const networkOn = settings.provider === "google";

  return (
    <section id="language-assist" className="space-y-4 scroll-mt-6">
      <div className="space-y-1">
        <h3 className="text-lg font-semibold tracking-tight">
          {t.languageAssist.settingsTitle}
        </h3>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.languageAssist.settingsDescription}
        </p>
      </div>

      <div className="space-y-4 rounded-lg border border-border/60 p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="lang-provider">{t.languageAssist.provider}</Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.languageAssist.providerHint}
            </p>
          </div>
          <select
            id="lang-provider"
            className="h-9 rounded-md border border-input bg-background px-2 text-sm"
            value={settings.provider}
            disabled={saving}
            onChange={(e) => {
              const provider = e.target.value === "google" ? "google" : "none";
              void persist({ ...settings, provider });
            }}
          >
            <option value="none">{t.languageAssist.providerNone}</option>
            <option value="google">{t.languageAssist.providerGoogle}</option>
          </select>
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="lang-auto-translate">
              {t.languageAssist.autoTranslation}
            </Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.languageAssist.autoTranslationHint}
            </p>
          </div>
          <Switch
            id="lang-auto-translate"
            checked={settings.autoTranslation}
            disabled={saving || !networkOn}
            onCheckedChange={(checked) => {
              void persist({ ...settings, autoTranslation: checked });
            }}
          />
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="lang-pronunciation">
              {t.languageAssist.pronunciation}
            </Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.languageAssist.pronunciationHint}
            </p>
          </div>
          <Switch
            id="lang-pronunciation"
            checked={settings.pronunciationGeneration}
            disabled={saving || !networkOn}
            onCheckedChange={(checked) => {
              void persist({ ...settings, pronunciationGeneration: checked });
            }}
          />
        </div>

        <p className="text-xs text-muted-foreground text-pretty">
          {t.languageAssist.offlineNote}
        </p>
      </div>
    </section>
  );
}
