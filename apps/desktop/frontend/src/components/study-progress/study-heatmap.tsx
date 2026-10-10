"use client";

import type { ActivityIntensity, HeatmapDay } from "@jabex/types";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

const INTENSITY_CLASS: Record<ActivityIntensity, string> = {
  none: "bg-muted/80",
  low: "bg-primary/25",
  medium: "bg-primary/55",
  high: "bg-primary",
};

function formatCount(value: number, locale: string) {
  return new Intl.NumberFormat(locale).format(value);
}

/** Align days into Sunday-start columns (GitHub-style weeks). */
function weeksFromDays(days: HeatmapDay[]): Array<Array<HeatmapDay | null>> {
  if (days.length === 0) return [];
  const first = new Date(`${days[0].date}T12:00:00`);
  const pad = first.getDay();
  const padded: Array<HeatmapDay | null> = [
    ...Array.from({ length: pad }, () => null),
    ...days,
  ];
  const weeks: Array<Array<HeatmapDay | null>> = [];
  for (let i = 0; i < padded.length; i += 7) {
    weeks.push(padded.slice(i, i + 7));
  }
  return weeks;
}

export function StudyHeatmap({ days }: { days: HeatmapDay[] }) {
  const t = useTranslations();
  const { locale } = useLocale();
  const weeks = weeksFromDays(days);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
          {t.studyProgress.heatmapTitle}
        </p>
        <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
          <span>{t.studyProgress.heatmapLess}</span>
          {(["none", "low", "medium", "high"] as ActivityIntensity[]).map(
            (level) => (
              <span
                key={level}
                className={cn("size-2.5 rounded-[2px]", INTENSITY_CLASS[level])}
                aria-hidden
              />
            ),
          )}
          <span>{t.studyProgress.heatmapMore}</span>
        </div>
      </div>

      <div
        className="overflow-x-auto pb-1"
        role="img"
        aria-label={t.studyProgress.heatmapAria}
      >
        <div className="inline-flex min-w-full gap-1">
          {weeks.map((week, wi) => (
            <div key={wi} className="flex flex-col gap-1">
              {week.map((day, di) => (
                <div
                  key={`${wi}-${di}`}
                  title={
                    day
                      ? t.studyProgress.heatmapDay
                          .replace("{date}", day.date)
                          .replace(
                            "{count}",
                            formatCount(day.cardsReviewed, locale),
                          )
                      : undefined
                  }
                  className={cn(
                    "size-2.5 rounded-[2px]",
                    day ? INTENSITY_CLASS[day.intensity] : "bg-transparent",
                  )}
                />
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
