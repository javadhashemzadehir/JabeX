"use client";

import { Flame } from "lucide-react";
import { useLocale, useTranslations } from "@/hooks/use-locale";

function formatCount(value: number, locale: string) {
  return new Intl.NumberFormat(locale).format(value);
}

export function StudyStreakCard({
  currentStreak,
  longestStreak,
  cardsReviewedToday,
}: {
  currentStreak: number;
  longestStreak: number;
  cardsReviewedToday: number;
}) {
  const t = useTranslations();
  const { locale } = useLocale();

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-border/60 bg-linear-to-br from-primary/5 via-background to-background p-4">
      <div className="flex items-start gap-3">
        <div
          className="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary"
          aria-hidden
        >
          <Flame className="size-5" />
        </div>
        <div className="min-w-0 space-y-1">
          <p className="text-xl font-semibold tracking-tight tabular-nums">
            {t.studyProgress.streakTitle.replace(
              "{count}",
              formatCount(currentStreak, locale),
            )}
          </p>
          <p className="text-sm leading-6 text-muted-foreground text-pretty">
            {currentStreak > 0
              ? t.studyProgress.streakHint.replace(
                  "{count}",
                  formatCount(currentStreak, locale),
                )
              : t.studyProgress.streakEmpty}
          </p>
        </div>
      </div>

      <dl className="grid grid-cols-2 gap-3 text-sm">
        <div className="space-y-0.5">
          <dt className="text-xs text-muted-foreground">
            {t.studyProgress.today}
          </dt>
          <dd className="font-medium tabular-nums">
            {t.studyProgress.cardsReviewed.replace(
              "{count}",
              formatCount(cardsReviewedToday, locale),
            )}
          </dd>
        </div>
        <div className="space-y-0.5">
          <dt className="text-xs text-muted-foreground">
            {t.studyProgress.best}
          </dt>
          <dd className="font-medium tabular-nums">
            {t.studyProgress.bestDays.replace(
              "{count}",
              formatCount(longestStreak, locale),
            )}
          </dd>
        </div>
      </dl>
    </div>
  );
}
