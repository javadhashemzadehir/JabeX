"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Volume2, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import type { AssistToggle, Card, LanguageAssistOverrides } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { api, isTauri } from "@/lib/tauri";
import { LanguageBadge } from "./language-badge";

const LANGUAGE_ASSIST_UPDATED_EVENT = "language-assist://updated";

const DEFAULT_OVERRIDES: LanguageAssistOverrides = {
  translation: "inherit",
  pronunciation: "inherit",
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
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const objectUrlRef = useRef<string | null>(null);

  const overrides = card.languageAssistOverrides ?? DEFAULT_OVERRIDES;

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
      if (objectUrlRef.current) {
        URL.revokeObjectURL(objectUrlRef.current);
        objectUrlRef.current = null;
      }
      audioRef.current?.pause();
    };
  }, []);

  async function playPronunciation() {
    if (!isTauri() || !card.pronunciationAudioPath) return;
    setPlaying(true);
    try {
      const bytes = await api.readPronunciationAudio(card.id);
      const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
      if (objectUrlRef.current) URL.revokeObjectURL(objectUrlRef.current);
      const url = URL.createObjectURL(blob);
      objectUrlRef.current = url;
      const audio = new Audio(url);
      audioRef.current = audio;
      audio.onended = () => setPlaying(false);
      audio.onerror = () => setPlaying(false);
      await audio.play();
    } catch (err) {
      setPlaying(false);
      toast.error(err instanceof Error ? err.message : t.languageAssist.playError);
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

  return (
    <div className="space-y-3 rounded-md border border-border/60 bg-muted/15 px-3 py-2.5">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <p className="text-xs font-medium text-muted-foreground">
            {t.languageAssist.title}
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
          {card.pronunciationAudioPath ? (
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
            {t.languageAssist.regenerate}
          </Button>
        </div>
      </div>

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

      {card.translation ? (
        <p className="text-sm text-pretty">
          <span className="text-muted-foreground">
            {t.languageAssist.translation}:{" "}
          </span>
          {card.translation}
        </p>
      ) : null}

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
          {t.languageAssist.failedHint}
        </p>
      ) : null}
    </div>
  );
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
