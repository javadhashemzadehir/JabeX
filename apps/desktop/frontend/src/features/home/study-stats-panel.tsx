"use client";

import { useEffect } from "react";
import Link from "next/link";
import { StatsSkeleton } from "@/components/feedback/stats-skeleton";
import { Button } from "@/components/ui/button";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useStatsStore } from "./stats-store";

function formatCount(value: number, locale: string) {
  return new Intl.NumberFormat(locale).format(value);
}

export function StudyStatsPanel() {
  const t = useTranslations();
  const { locale } = useLocale();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const { stats, loading, error, desktop, hydrate } = useStatsStore();

  useEffect(() => {
    void hydrate();
  }, [hydrate, activeProfileId]);

  if (!desktop) {
    return (
      <p className="text-sm text-muted-foreground">{t.stats.desktopOnly}</p>
    );
  }

  if (loading && !stats) {
    return <StatsSkeleton />;
  }

  if (error && !stats) {
    return (
      <p className="text-sm text-destructive" role="alert">
        {error}
      </p>
    );
  }

  if (!stats) return null;

  const maxBox = Math.max(1, ...stats.cardsByBox);

  return (
    <div className="space-y-5" aria-live="polite">
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <StatMetric
          label={t.stats.due}
          value={formatCount(stats.dueCount, locale)}
          emphasize={stats.dueCount > 0}
        />
        <StatMetric
          label={t.stats.reviewsToday}
          value={formatCount(stats.reviewsToday, locale)}
        />
        <StatMetric
          label={t.stats.cards}
          value={formatCount(stats.cardCount, locale)}
        />
        <StatMetric
          label={t.stats.decks}
          value={formatCount(stats.deckCount, locale)}
        />
      </div>

      {stats.cardCount > 0 ? (
        <div className="space-y-3 rounded-lg border border-border/60 bg-muted/30 px-3 py-3.5 sm:px-4">
          <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            {t.stats.boxes}
          </p>
          <div className="grid grid-cols-5 items-end gap-2 sm:gap-3">
            {stats.cardsByBox.map((count, index) => {
              const box = index + 1;
              const height = Math.max(8, Math.round((count / maxBox) * 56));
              return (
                <div
                  key={box}
                  className="flex min-w-0 flex-col items-center gap-1.5 text-center"
                  title={t.stats.boxCount
                    .replace("{box}", String(box))
                    .replace("{count}", formatCount(count, locale))}
                >
                  <div className="flex h-14 w-full items-end justify-center">
                    <div
                      className="w-full max-w-8 rounded-md bg-foreground/70"
                      style={{ height }}
                      aria-hidden
                    />
                  </div>
                  <span className="text-[11px] text-muted-foreground tabular-nums">
                    {box}
                  </span>
                  <span className="text-xs font-semibold tabular-nums tracking-tight">
                    {formatCount(count, locale)}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      ) : null}

      {stats.dueCount > 0 ? (
        <Button asChild size="sm">
          <Link href="/review">{t.stats.reviewCta}</Link>
        </Button>
      ) : null}
    </div>
  );
}

function StatMetric({
  label,
  value,
  emphasize = false,
}: {
  label: string;
  value: string;
  emphasize?: boolean;
}) {
  return (
    <div className="flex min-h-18 flex-col justify-between gap-2 rounded-lg border border-border/60 bg-background px-3 py-3">
      <p className="text-xs leading-4 text-muted-foreground text-pretty">
        {label}
      </p>
      <p
        className={
          emphasize
            ? "text-2xl font-semibold tracking-tight tabular-nums text-foreground"
            : "text-2xl font-semibold tracking-tight tabular-nums text-foreground/90"
        }
      >
        {value}
      </p>
    </div>
  );
}
