"use client";

import type { StudyConsistencyStats } from "@jabex/types";
import { useLocale, useTranslations } from "@/hooks/use-locale";

function formatCount(value: number, locale: string) {
  return new Intl.NumberFormat(locale).format(value);
}

function formatAverage(value: number, locale: string) {
  return new Intl.NumberFormat(locale, {
    maximumFractionDigits: 1,
  }).format(value);
}

export function StudyStatistics({ stats }: { stats: StudyConsistencyStats }) {
  const t = useTranslations();
  const { locale } = useLocale();

  const items = [
    {
      label: t.studyProgress.totalDays,
      value: formatCount(stats.totalStudyDays, locale),
    },
    {
      label: t.studyProgress.week,
      value: formatCount(stats.cardsReviewedThisWeek, locale),
    },
    {
      label: t.studyProgress.month,
      value: formatCount(stats.cardsReviewedThisMonth, locale),
    },
    {
      label: t.studyProgress.average,
      value: formatAverage(stats.averageCardsPerStudyDay, locale),
    },
  ];

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
      {items.map((item) => (
        <div
          key={item.label}
          className="space-y-1 rounded-md border border-border/50 px-3 py-2.5"
        >
          <p className="text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
            {item.label}
          </p>
          <p className="text-lg font-semibold tabular-nums tracking-tight">
            {item.value}
          </p>
        </div>
      ))}
    </div>
  );
}
