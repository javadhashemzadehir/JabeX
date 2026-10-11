"use client";

import { useRef, useState } from "react";
import { Volume2 } from "lucide-react";
import { toast } from "sonner";
import type { Card } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { useTranslations } from "@/hooks/use-locale";
import { api, isTauri } from "@/lib/tauri";
import { playAudioBytes, sleep, type PlaybackSpeed } from "./audio-player";
import { browserTtsAvailable, speakWithBrowser, stopBrowserSpeech } from "./browser-tts";
import { PronunciationControls } from "./pronunciation-controls";

type ReviewAudioButtonProps = {
  card: Card;
  /** Play front lemma vs back face text. */
  side?: "front" | "back";
};

export function ReviewAudioButton({ card, side = "front" }: ReviewAudioButtonProps) {
  const t = useTranslations();
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<PlaybackSpeed>(1);
  const [repeat, setRepeat] = useState(false);
  const [shadowing, setShadowing] = useState(false);
  const abortRef = useRef<AbortController | null>(null);

  const hasNative = Boolean(card.pronunciationAudioPath);
  const canBrowser = browserTtsAvailable();
  if (!hasNative && !canBrowser) return null;

  async function playOnce(signal: AbortSignal) {
    if (hasNative && isTauri()) {
      const bytes = await api.readPronunciationAudio(card.id);
      await playAudioBytes(bytes, { playbackRate: speed, signal });
      return;
    }
    const text =
      side === "back"
        ? plainPreview(card.backContent)
        : card.transliteration?.resolvedWord ||
          plainPreview(card.frontContent);
    await speakWithBrowser({
      text,
      language: card.detectedLanguage ?? "en",
      rate: speed,
    });
  }

  async function handlePlay() {
    abortRef.current?.abort();
    stopBrowserSpeech();
    const controller = new AbortController();
    abortRef.current = controller;
    setPlaying(true);
    try {
      do {
        await playOnce(controller.signal);
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

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={playing}
          onClick={() => void handlePlay()}
          aria-label={t.speech.playAudio}
        >
          <Volume2 className="size-3.5" />
          {t.speech.playAudio}
        </Button>
        {playing ? (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => {
              abortRef.current?.abort();
              stopBrowserSpeech();
            }}
          >
            {t.speech.stop}
          </Button>
        ) : null}
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
    </div>
  );
}

function plainPreview(doc: Card["frontContent"]): string {
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
    return walk(doc.content ?? []).trim();
  } catch {
    return "";
  }
}
