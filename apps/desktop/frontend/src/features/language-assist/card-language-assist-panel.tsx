"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Volume2, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import type { AssistToggle, Card, LanguageAssistOverrides } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { PronunciationControls } from "@/features/speech/pronunciation-controls";
import {
  playAudioBytes,
  sleep,
  type PlaybackSpeed,
} from "@/features/speech/audio-player";
import {
  browserTtsAvailable,
  speakWithBrowser,
  stopBrowserSpeech,
} from "@/features/speech/browser-tts";
import { useTranslations } from "@/hooks/use-locale";
import { api, isTauri } from "@/lib/tauri";
import { LanguageBadge } from "./language-badge";

const LANGUAGE_ASSIST_UPDATED_EVENT = "language-assist://updated";

const DEFAULT_OVERRIDES: LanguageAssistOverrides = {
  translation: "inherit",
  pronunciation: "inherit",
};

const LANG_FLAGS: Record<string, string> = {
  en: "🇬🇧",
  fa: "🇮🇷",
  fr: "🇫🇷",
  de: "🇩🇪",
  es: "🇪🇸",
  ar: "🇸🇦",
  ja: "🇯🇵",
  zh: "🇨🇳",
};

type CardLanguageAssistPanelProps = {
  card: Card;
  onCardUpdated?: (card: Card) => void;
};

export function CardLanguageAssistPanel({
  card,
  onCardUpdated,
}: CardLanguageAssistPanelProps) {
  const t = useTranslations();
  const [busy, setBusy] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<PlaybackSpeed>(1);
  const [repeat, setRepeat] = useState(false);
  const [shadowing, setShadowing] = useState(false);
  const abortRef = useRef<AbortController | null>(null);

  const overrides = card.languageAssistOverrides ?? DEFAULT_OVERRIDES;
  const lang = card.detectedLanguage ?? "und";
  const flag = LANG_FLAGS[lang] ?? "";

  const refreshCard = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const next = await api.getCard(card.id);
      onCardUpdated?.(next);
    } catch {
      // ignore — card may have been deleted
    }
  }, [card.id, onCardUpdated]);

  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void listen<{ cardId: string }>(LANGUAGE_ASSIST_UPDATED_EVENT, (event) => {
      if (event.payload.cardId === card.id) {
        void refreshCard();
      }
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [card.id, refreshCard]);

  useEffect(() => {
    return () => {
      abortRef.current?.abort();
      stopBrowserSpeech();
    };
  }, []);

  async function playPronunciation() {
    abortRef.current?.abort();
    stopBrowserSpeech();
    const controller = new AbortController();
    abortRef.current = controller;
    setPlaying(true);
    try {
      do {
        if (card.pronunciationAudioPath && isTauri()) {
          const bytes = await api.readPronunciationAudio(card.id);
          await playAudioBytes(bytes, {
            playbackRate: speed,
            signal: controller.signal,
          });
        } else if (browserTtsAvailable()) {
          const text =
            card.transliteration?.resolvedWord ||
            card.pronunciationIpa ||
            lemmaFromCard(card);
          await speakWithBrowser({
            text,
            language: card.detectedLanguage ?? "en",
            rate: speed,
          });
        } else {
          throw new Error(t.languageAssist.playError);
        }
        if (shadowing) {
          await sleep(1500, controller.signal);
        }
      } while (repeat && !controller.signal.aborted);
    } catch (err) {
      if (!(err instanceof DOMException && err.name === "AbortError")) {
        toast.error(err instanceof Error ? err.message : t.languageAssist.playError);
      }
    } finally {
      setPlaying(false);
    }
  }

  async function regenerate() {
    if (!isTauri()) return;
    setBusy(true);
    try {
      const next = await api.regenerateLanguageAssist(card.id);
      onCardUpdated?.(next);
      toast.success(t.languageAssist.regenerated);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : t.languageAssist.regenerateError,
      );
    } finally {
      setBusy(false);
    }
  }

  async function updateOverrides(next: LanguageAssistOverrides) {
    if (!isTauri()) return;
    setBusy(true);
    try {
      const updated = await api.setCardLanguageAssist(card.id, next);
      onCardUpdated?.(updated);
      toast.success(t.languageAssist.cardSaved);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : t.languageAssist.cardSaveError,
      );
    } finally {
      setBusy(false);
    }
  }

  const pending = card.languageAssistStatus === "pending" || busy;
  const canPlay =
    Boolean(card.pronunciationAudioPath) || browserTtsAvailable();

  return (
    <div className="space-y-3 rounded-md border border-border/60 bg-muted/15 px-3 py-2.5">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <p className="text-xs font-medium text-muted-foreground">
            {t.speech.languageTools}
          </p>
          <LanguageBadge card={card} />
          {pending ? (
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <Spinner className="size-3" />
              {t.languageAssist.pending}
            </span>
          ) : null}
        </div>
        <div className="flex flex-wrap items-center gap-1">
          {canPlay ? (
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={playing}
              onClick={() => void playPronunciation()}
              aria-label={t.languageAssist.play}
            >
              <Volume2 className="size-3.5" />
              {t.languageAssist.play}
            </Button>
          ) : null}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            disabled={busy || pending}
            onClick={() => void regenerate()}
          >
            {busy ? (
              <Spinner className="size-3.5" />
            ) : (
              <RefreshCw className="size-3.5" />
            )}
            {t.speech.generateAudio}
          </Button>
        </div>
      </div>

      <div className="space-y-1 text-sm">
        <p className="text-pretty">
          <span className="text-muted-foreground">
            {t.speech.detectedLanguage}:{" "}
          </span>
          {flag ? `${flag} ` : null}
          {lang === "und" ? t.speech.unknownLanguage : lang}
        </p>
        {card.translation ? (
          <p className="text-pretty">
            <span className="text-muted-foreground">
              {t.languageAssist.translation}:{" "}
            </span>
            {card.translation}
          </p>
        ) : null}
        <p className="text-pretty">
          <span className="text-muted-foreground">
            {t.speech.pronunciation}:{" "}
          </span>
          {card.pronunciationIpa ||
            card.transliteration?.resolvedWord ||
            lemmaFromCard(card) ||
            "—"}
        </p>
      </div>

      <PronunciationControls
        speed={speed}
        onSpeedChange={setSpeed}
        repeat={repeat}
        onRepeatChange={setRepeat}
        shadowing={shadowing}
        onShadowingChange={setShadowing}
        disabled={playing}
      />

      <p className="text-xs text-muted-foreground text-pretty">
        {t.languageAssist.cardHint}
      </p>

      <div className="grid gap-2 sm:grid-cols-2">
        <OverrideSelect
          id={`card-lang-translation-${card.id}`}
          label={t.languageAssist.autoTranslation}
          value={overrides.translation}
          disabled={busy}
          labels={t.languageAssist.toggles}
          onChange={(translation) =>
            void updateOverrides({ ...overrides, translation })
          }
        />
        <OverrideSelect
          id={`card-lang-pronunciation-${card.id}`}
          label={t.languageAssist.pronunciation}
          value={overrides.pronunciation}
          disabled={busy}
          labels={t.languageAssist.toggles}
          onChange={(pronunciation) =>
            void updateOverrides({ ...overrides, pronunciation })
          }
        />
      </div>

      {card.transliteration?.resolvedWord ? (
        <p className="text-xs text-muted-foreground text-pretty">
          {t.languageAssist.resolvedAs.replace(
            "{word}",
            card.transliteration.resolvedWord,
          )}
        </p>
      ) : null}

      {card.languageAssistStatus === "failed" ? (
        <p className="text-xs text-destructive text-pretty">
          {t.speech.failedHint}
        </p>
      ) : null}
    </div>
  );
}

function lemmaFromCard(card: Card): string {
  try {
    const walk = (nodes: unknown[]): string =>
      nodes
        .map((n) => {
          if (!n || typeof n !== "object") return "";
          const node = n as { text?: string; content?: unknown[] };
          if (typeof node.text === "string") return node.text;
          if (Array.isArray(node.content)) return walk(node.content);
          return "";
        })
        .join("");
    return walk(card.frontContent.content ?? []).trim().split(/\s+/)[0] ?? "";
  } catch {
    return "";
  }
}

function OverrideSelect({
  id,
  label,
  value,
  disabled,
  labels,
  onChange,
}: {
  id: string;
  label: string;
  value: AssistToggle;
  disabled: boolean;
  labels: { inherit: string; enabled: string; disabled: string };
  onChange: (value: AssistToggle) => void;
}) {
  return (
    <div className="space-y-1">
      <Label htmlFor={id} className="text-xs">
        {label}
      </Label>
      <select
        id={id}
        className="h-9 w-full rounded-md border border-input bg-background px-2 text-sm"
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value as AssistToggle)}
      >
        <option value="inherit">{labels.inherit}</option>
        <option value="enabled">{labels.enabled}</option>
        <option value="disabled">{labels.disabled}</option>
      </select>
    </div>
  );
}
