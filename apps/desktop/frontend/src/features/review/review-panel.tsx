"use client";

import { useEffect, useEffectEvent } from "react";
import Link from "next/link";
import { BookOpen, CheckCircle2, Eye } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { toast } from "sonner";
import type { Grade } from "@jabex/types";
import { ReviewSkeleton } from "@/components/feedback/review-skeleton";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { RichTextViewer } from "@/features/editor/rich-text-viewer";
import { LanguageBadge } from "@/features/language-assist/language-badge";
import { ReviewAudioButton } from "@/features/speech/review-audio-button";
import {
  playAudioBytes,
} from "@/features/speech/audio-player";
import {
  browserTtsAvailable,
  speakWithBrowser,
} from "@/features/speech/browser-tts";
import { formatChordLabel, matchesChord } from "@/features/shortcuts/chord";
import { DEFAULT_KEYBOARD_SHORTCUTS } from "@/features/shortcuts/defaults";
import {
  isTypingTarget,
  useShortcutChord,
} from "@/features/shortcuts/use-shortcut";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useAppPreferencesStore } from "@/features/settings/app-preferences-store";
import { api } from "@/lib/tauri";
import { useReviewStore } from "./review-store";

const GRADES: Grade[] = ["again", "hard", "good", "easy"];

export function ReviewPanel({ deckId }: { deckId?: string | null }) {
  const t = useTranslations();
  const tauri = useIsTauri();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const {
    queue,
    revealed,
    loading,
    submitting,
    error,
    desktop,
    reviewedCount,
    hydrate,
    reveal,
    submit,
    reset,
  } = useReviewStore();

  const current = queue[0] ?? null;
  const remaining = queue.length;
  const totalInSession = remaining + reviewedCount;
  const shortcuts =
    useAppPreferencesStore((s) => s.preferences.keyboardShortcuts) ??
    DEFAULT_KEYBOARD_SHORTCUTS;
  const flipChord = useShortcutChord("reviewFlipOrGood");
  const gradeAgainChord = useShortcutChord("reviewGradeAgain");
  const gradeHardChord = useShortcutChord("reviewGradeHard");
  const gradeGoodChord = useShortcutChord("reviewGradeGood");
  const gradeEasyChord = useShortcutChord("reviewGradeEasy");
  const playAudioChord = useShortcutChord("reviewPlayAudio");
  const gradeChords = {
    again: gradeAgainChord,
    hard: gradeHardChord,
    good: gradeGoodChord,
    easy: gradeEasyChord,
  };

  async function playCurrentAudio() {
    if (!current) return;
    try {
      if (current.pronunciationAudioPath && tauri) {
        const bytes = await api.readPronunciationAudio(current.id);
        await playAudioBytes(bytes);
        return;
      }
      if (browserTtsAvailable()) {
        const lemma =
          current.transliteration?.resolvedWord ||
          current.pronunciationIpa?.replace(/\//g, "") ||
          "";
        if (lemma.trim()) {
          await speakWithBrowser({
            text: lemma,
            language: current.detectedLanguage ?? "en",
          });
        }
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.languageAssist.playError);
    }
  }

  useEffect(() => {
    void hydrate(deckId ?? null);
    return () => reset();
  }, [hydrate, reset, deckId, activeProfileId]);

  async function handleGrade(grade: Grade) {
    try {
      const outcome = await submit(grade);
      if (!outcome) return;
      const moved =
        outcome.review.boxAfter > outcome.review.boxBefore
          ? t.review.promoted
          : outcome.review.boxAfter < outcome.review.boxBefore
            ? t.review.demoted
            : t.review.stayed;
      toast.success(
        moved.replace("{box}", String(outcome.review.boxAfter)),
      );
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.review.error);
    }
  }

  const onReviewKeyDown = useEffectEvent((event: KeyboardEvent) => {
    if (!current || submitting || loading) return;
    if (isTypingTarget(event.target)) return;

    if (matchesChord(event, shortcuts.reviewPlayAudio ?? "p")) {
      event.preventDefault();
      void playCurrentAudio();
      return;
    }

    if (
      matchesChord(event, shortcuts.reviewFlipOrGood) ||
      matchesChord(event, shortcuts.reviewFlipOrGoodAlt)
    ) {
      event.preventDefault();
      if (!revealed) {
        reveal();
        return;
      }
      void handleGrade("good");
      return;
    }

    if (!revealed) return;

    const gradeEntries: [Grade, string][] = [
      ["again", shortcuts.reviewGradeAgain],
      ["hard", shortcuts.reviewGradeHard],
      ["good", shortcuts.reviewGradeGood],
      ["easy", shortcuts.reviewGradeEasy],
    ];
    for (const [grade, chord] of gradeEntries) {
      if (!matchesChord(event, chord)) continue;
      event.preventDefault();
      void handleGrade(grade);
      return;
    }
  });

  useEffect(() => {
    window.addEventListener("keydown", onReviewKeyDown);
    return () => window.removeEventListener("keydown", onReviewKeyDown);
  }, [onReviewKeyDown]);

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.review.desktopOnly}</p>
    );
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-6"
    >
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-1">
          <h2 className="text-2xl font-semibold tracking-tight">
            {t.review.title}
          </h2>
          <p className="text-sm text-muted-foreground text-pretty">
            {t.review.description}
          </p>
        </div>
        {!loading && totalInSession > 0 ? (
          <p
            className="text-sm text-muted-foreground tabular-nums"
            aria-live="polite"
          >
            {t.review.progress
              .replace("{done}", String(reviewedCount))
              .replace("{total}", String(totalInSession))}
          </p>
        ) : null}
      </div>

      {loading && remaining === 0 ? <ReviewSkeleton /> : null}

      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}

      {!loading && !error && remaining === 0 ? (
        <div
          className="flex flex-col items-start gap-4 rounded-lg border border-dashed border-border/80 bg-muted/20 px-6 py-10"
          role="status"
        >
          <div className="flex size-10 items-center justify-center rounded-md bg-muted">
            {reviewedCount > 0 ? (
              <CheckCircle2 className="size-5 text-muted-foreground" aria-hidden />
            ) : (
              <BookOpen className="size-5 text-muted-foreground" aria-hidden />
            )}
          </div>
          <div className="space-y-1">
            <p className="font-medium">
              {reviewedCount > 0
                ? t.review.sessionDoneTitle
                : t.review.emptyTitle}
            </p>
            <p className="max-w-md text-sm text-muted-foreground text-pretty">
              {reviewedCount > 0
                ? t.review.sessionDoneHint.replace(
                    "{count}",
                    String(reviewedCount),
                  )
                : deckId
                  ? t.review.emptyHintDeck
                  : t.review.emptyHint}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            {reviewedCount > 0 ? (
              <Button
                type="button"
                variant="outline"
                onClick={() => void hydrate(deckId ?? null)}
              >
                {t.review.refresh}
              </Button>
            ) : deckId ? (
              <Button asChild>
                <Link
                  href={`/decks/view?id=${encodeURIComponent(deckId)}&create=1`}
                >
                  {t.review.addCards}
                </Link>
              </Button>
            ) : (
              <Button asChild>
                <Link href="/decks">{t.review.goToDecks}</Link>
              </Button>
            )}
          </div>
        </div>
      ) : null}

      {current ? (
        <div className="mx-auto flex w-full max-w-2xl flex-col gap-4">
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>
              {t.review.boxLabel.replace("{box}", String(current.boxPosition))}
              {" · "}
              {t.review.remaining.replace("{count}", String(remaining))}
            </span>
            <LanguageBadge card={current} />
          </div>

          <ReviewAudioButton
            card={current}
            side={revealed ? "back" : "front"}
          />
          <p className="text-xs text-muted-foreground">
            {t.speech.playShortcutHint.replace(
              "{key}",
              formatChordLabel(playAudioChord),
            )}
          </p>

          <AnimatePresence mode="wait">
            <motion.div
              key={`${current.id}-${revealed ? "back" : "front"}`}
              role={revealed ? "region" : "button"}
              tabIndex={revealed ? undefined : 0}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              onClick={() => {
                if (!revealed) reveal();
              }}
              onKeyDown={(event) => {
                if (revealed) return;
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  reveal();
                }
              }}
              className={
                revealed
                  ? "min-h-56 w-full rounded-lg border border-border/60 bg-linear-to-br from-background via-muted/30 to-background px-6 py-10 text-start"
                  : "min-h-56 w-full cursor-pointer rounded-lg border border-border/60 bg-linear-to-br from-background via-muted/30 to-background px-6 py-10 text-start transition-colors hover:bg-muted/20"
              }
              aria-label={
                revealed ? t.review.backSide : t.review.revealHint
              }
            >
              <p className="mb-3 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {revealed ? t.review.backSide : t.review.frontSide}
              </p>
              <RichTextViewer
                document={revealed ? current.backContent : current.frontContent}
                contentClassName="text-xl leading-relaxed text-balance"
              />
              {!revealed ? (
                <p className="mt-8 flex items-center gap-2 text-sm text-muted-foreground">
                  <Eye className="size-4" aria-hidden />
                  {t.review.revealHint}
                </p>
              ) : null}
            </motion.div>
          </AnimatePresence>

          {!revealed ? (
            <Button type="button" onClick={reveal} className="self-start">
              <Eye className="size-4" aria-hidden />
              {t.review.reveal}
              <kbd className="ms-1 rounded border border-border/80 bg-muted px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground">
                {formatChordLabel(flipChord)}
              </kbd>
            </Button>
          ) : (
            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                {GRADES.map((grade) => (
                  <Button
                    key={grade}
                    type="button"
                    variant={grade === "again" ? "destructive" : "outline"}
                    disabled={submitting}
                    onClick={() => void handleGrade(grade)}
                    className="flex-col gap-1 py-2 sm:h-auto"
                  >
                    {submitting ? (
                      <Spinner className="size-4" />
                    ) : (
                      <>
                        <span>{t.review.grades[grade]}</span>
                        <kbd className="rounded border border-border/60 bg-muted/80 px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground">
                          {formatChordLabel(gradeChords[grade])}
                        </kbd>
                      </>
                    )}
                  </Button>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                {t.review.shortcuts.hint}
              </p>
            </div>
          )}
        </div>
      ) : null}
    </motion.section>
  );
}
