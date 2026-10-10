"use client";

import type { LanguageAssistStatus, TransliterationMetadata } from "@jabex/types";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

const LANGUAGE_LABELS: Record<string, string> = {
  en: "EN",
  fa: "FA",
  ar: "AR",
  de: "DE",
  fr: "FR",
  es: "ES",
  ru: "RU",
  ja: "JA",
  zh: "ZH",
  ko: "KO",
  el: "EL",
  und: "?",
};

/** Minimal card shape for list rows (`Card` or `CardSummary`). */
type LanguageBadgeCard = {
  detectedLanguage?: string | null;
  languageAssistStatus?: LanguageAssistStatus;
  transliteration?: TransliterationMetadata | null;
};

type LanguageBadgeProps = {
  card: LanguageBadgeCard;
  className?: string;
};

export function LanguageBadge({ card, className }: LanguageBadgeProps) {
  const t = useTranslations();
  const code = card.detectedLanguage?.trim().toLowerCase();
  if (!code || code === "und") {
    if (card.languageAssistStatus === "pending") {
      return (
        <span
          className={cn(
            "inline-flex items-center rounded-md border border-border/70 bg-muted/40 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-muted-foreground",
            className,
          )}
        >
          {t.languageAssist.pending}
        </span>
      );
    }
    return null;
  }

  const label = LANGUAGE_LABELS[code] ?? code.toUpperCase().slice(0, 3);
  const isTranslit = card.transliteration?.isTransliteration;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-md border border-border/70 bg-muted/40 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-muted-foreground",
        className,
      )}
      title={
        isTranslit && card.transliteration?.resolvedWord
          ? t.languageAssist.transliterationHint.replace(
              "{word}",
              card.transliteration.resolvedWord,
            )
          : t.languageAssist.detectedHint.replace("{lang}", code)
      }
    >
      {label}
      {isTranslit ? (
        <span className="normal-case tracking-normal text-[10px] opacity-80">
          → {card.transliteration?.resolvedLanguage?.toUpperCase() ?? "?"}
        </span>
      ) : null}
    </span>
  );
}
