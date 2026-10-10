"use client";

import { useEffect } from "react";
import { StatsSkeleton } from "@/components/feedback/stats-skeleton";
import { StudyHeatmap } from "@/components/study-progress/study-heatmap";
import { StudyStatistics } from "@/components/study-progress/study-statistics";
import { StudyStreakCard } from "@/components/study-progress/study-streak-card";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useTranslations } from "@/hooks/use-locale";
import { useStudyProgressStore } from "./study-progress-store";

export function StudyProgressPanel() {
  const t = useTranslations();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const { stats, loading, error, desktop, hydrate } = useStudyProgressStore();

  useEffect(() => {
    void hydrate();
  }, [hydrate, activeProfileId]);

  if (!desktop) {
    return (
      <p className="text-sm text-muted-foreground">
        {t.studyProgress.desktopOnly}
      </p>
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

  return (
    <div className="space-y-5" aria-live="polite">
      <StudyStreakCard
        currentStreak={stats.currentStreak}
        longestStreak={stats.longestStreak}
        cardsReviewedToday={stats.cardsReviewedToday}
      />
      <StudyStatistics stats={stats} />
      <StudyHeatmap days={stats.heatmap} />
    </div>
  );
}
