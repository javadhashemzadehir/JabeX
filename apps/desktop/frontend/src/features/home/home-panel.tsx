"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { motion } from "framer-motion";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { createSampleDeck } from "@/features/onboarding/create-sample-deck";
import { useOnboardingTourStore } from "@/features/onboarding/tour-store";
import { BackupPanel } from "@/features/profiles/backup-panel";
import { useProfileStore } from "@/features/profiles/profile-store";
import { StudyProgressPanel } from "@/features/study-progress/study-progress-panel";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { isDevMode } from "@/lib/is-dev";
import { useDeckStore } from "@/features/decks/deck-store";
import { useStatsStore } from "./stats-store";
import { StudyStatsPanel } from "./study-stats-panel";

export function HomePanel() {
  const t = useTranslations();
  const router = useRouter();
  const tauri = useIsTauri();
  const active = useProfileStore((s) => s.active);
  const profiles = useProfileStore((s) => s.profiles);
  const desktop = useProfileStore((s) => s.desktop);
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const stats = useStatsStore((s) => s.stats);
  const hydrateStats = useStatsStore((s) => s.hydrate);
  const hydrateDecks = useDeckStore((s) => s.hydrate);
  const requestTour = useOnboardingTourStore((s) => s.requestStart);
  const [sampleBusy, setSampleBusy] = useState(false);
  const showDevSample = isDevMode() && desktop;

  useEffect(() => {
    void hydrateStats();
  }, [hydrateStats, activeProfileId]);

  // Only use first-time copy when we know there are zero decks.
  // While stats are loading (or unavailable), prefer the non-first CTA
  // so existing users never see "Create your first…".
  const isEmptyLibrary = stats?.deckCount === 0;
  const ctaLabel = isEmptyLibrary ? t.home.cta : t.home.ctaNew;
  const ctaHint = isEmptyLibrary ? t.home.ctaHint : t.home.ctaNewHint;

  async function handleSampleDeck() {
    if (!tauri || !showDevSample || sampleBusy) return;
    setSampleBusy(true);
    try {
      const deck = await createSampleDeck({
        name: t.onboarding.devSample.deckName,
        description: t.onboarding.devSample.deckDescription,
        cards: t.onboarding.devSample.cards,
      });
      await Promise.all([hydrateStats(), hydrateDecks()]);
      toast.success(t.onboarding.devSample.created);
      router.push(`/decks/view?id=${encodeURIComponent(deck.id)}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.decks.error);
    } finally {
      setSampleBusy(false);
    }
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-4"
    >
      <Card
        data-tour="home-welcome"
        className="overflow-hidden border-border/60 bg-linear-to-br from-background via-muted/40 to-background shadow-none"
      >
        <CardHeader>
          <CardDescription>{t.app.tagline}</CardDescription>
          <CardTitle className="text-3xl tracking-tight text-balance">
            {t.home.welcome}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm leading-7 text-muted-foreground text-pretty">
            {t.home.description}
          </p>
          {desktop && active ? (
            <p className="text-sm font-medium text-foreground/90">
              {t.home.profiles}: {active.displayName}
              {profiles.length > 1 ? ` (${profiles.length})` : ""}
            </p>
          ) : null}
          <div className="space-y-2">
            <div className="flex flex-wrap gap-2">
              <Button asChild>
                <Link href="/decks">{ctaLabel}</Link>
              </Button>
              {desktop ? (
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => requestTour()}
                >
                  {t.home.tourCta}
                </Button>
              ) : null}
              {showDevSample && isEmptyLibrary ? (
                <Button
                  type="button"
                  variant="ghost"
                  disabled={sampleBusy}
                  onClick={() => void handleSampleDeck()}
                >
                  {sampleBusy ? <Spinner /> : null}
                  {t.onboarding.devSample.cta}
                </Button>
              ) : null}
            </div>
            <p className="text-xs text-muted-foreground">{ctaHint}</p>
            {desktop ? (
              <p className="text-xs text-muted-foreground">{t.home.tourHint}</p>
            ) : null}
            {showDevSample && isEmptyLibrary ? (
              <p className="text-xs text-muted-foreground">
                {t.onboarding.devSample.hint}
              </p>
            ) : null}
          </div>
        </CardContent>
      </Card>

      {!isEmptyLibrary ? (
        <>
          <Card className="border-border/60 shadow-none">
            <CardHeader>
              <CardTitle className="text-lg">{t.stats.title}</CardTitle>
              <CardDescription>{t.stats.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <StudyStatsPanel />
            </CardContent>
          </Card>

          <Card className="border-border/60 shadow-none">
            <CardHeader>
              <CardTitle className="text-lg">{t.studyProgress.title}</CardTitle>
              <CardDescription>{t.studyProgress.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <StudyProgressPanel />
            </CardContent>
          </Card>

          <Card className="border-border/60 shadow-none">
            <CardHeader>
              <CardTitle className="text-lg">{t.backup.title}</CardTitle>
              <CardDescription>{t.app.privacyNote}</CardDescription>
            </CardHeader>
            <CardContent>
              <BackupPanel />
            </CardContent>
          </Card>
        </>
      ) : null}
    </motion.section>
  );
}
