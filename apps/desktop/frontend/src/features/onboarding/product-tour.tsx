"use client";

import { useEffect, useMemo, useRef } from "react";
import { usePathname, useRouter } from "next/navigation";
import { STATUS, useJoyride, type Step } from "react-joyride";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import {
  isProductTourCompleted,
  markProductTourCompleted,
} from "./tour-storage";
import { useOnboardingTourStore } from "./tour-store";
import { readTourTheme } from "./tour-theme";
import { waitForSelector } from "./wait-for-selector";

async function ensurePath(pathname: string, href: string, router: { push: (url: string) => void }) {
  if (pathname === href || pathname.startsWith(`${href}/`)) return;
  router.push(href);
}

export function ProductTour() {
  const t = useTranslations();
  const { direction } = useLocale();
  const router = useRouter();
  const pathname = usePathname();
  const pathnameRef = useRef(pathname);

  useEffect(() => {
    pathnameRef.current = pathname;
  }, [pathname]);

  const desktop = useProfileStore((s) => s.desktop);
  const loading = useProfileStore((s) => s.loading);
  const initialSetupPending = useProfileStore((s) => s.initialSetupPending);
  const profileId = useProfileStore((s) => s.active?.id);
  const startToken = useOnboardingTourStore((s) => s.startToken);
  const autoStartedForProfile = useRef<string | null>(null);

  // CSS variables; re-read each render so light/dark switches apply.
  const theme = readTourTheme();

  const steps = useMemo<Step[]>(() => {
    const tour = t.onboarding.tour;
    return [
      {
        id: "welcome",
        target: "[data-tour='home-welcome']",
        placement: "center",
        title: tour.welcomeTitle,
        content: tour.welcomeBody,
        skipBeacon: true,
        before: async () => {
          await ensurePath(pathnameRef.current, "/", router);
          await waitForSelector("[data-tour='home-welcome']");
        },
      },
      {
        id: "nav-decks",
        target: "[data-tour='nav-decks']",
        placement: direction === "rtl" ? "left" : "right",
        title: tour.decksTitle,
        content: tour.decksBody,
        skipBeacon: true,
      },
      {
        id: "create-deck",
        target: "[data-tour='create-deck']",
        placement: "bottom",
        title: tour.createDeckTitle,
        content: tour.createDeckBody,
        skipBeacon: true,
        before: async () => {
          await ensurePath(pathnameRef.current, "/decks", router);
          await waitForSelector("[data-tour='create-deck']");
        },
      },
      {
        id: "create-card",
        target: "[data-tour='create-card'], [data-tour='create-deck']",
        placement: "bottom",
        title: tour.createCardTitle,
        content: tour.createCardBody,
        skipBeacon: true,
        before: async () => {
          const openDeck = document.querySelector<HTMLAnchorElement>(
            '[href^="/decks/view"]',
          );
          if (openDeck?.href) {
            const path = openDeck.getAttribute("href");
            if (path) {
              router.push(path);
              try {
                await waitForSelector("[data-tour='create-card']");
                return;
              } catch {
                // Fall back to decks create button if the deck view did not mount.
              }
            }
          }
          await ensurePath(pathnameRef.current, "/decks", router);
          await waitForSelector("[data-tour='create-deck']");
        },
      },
      {
        id: "nav-review",
        target: "[data-tour='nav-review']",
        placement: direction === "rtl" ? "left" : "right",
        title: tour.reviewTitle,
        content: tour.reviewBody,
        skipBeacon: true,
      },
      {
        id: "nav-settings",
        target: "[data-tour='nav-settings']",
        placement: direction === "rtl" ? "left" : "right",
        title: tour.settingsTitle,
        content: tour.settingsBody,
        skipBeacon: true,
      },
      {
        id: "settings-application",
        target: "[data-tour='settings-application']",
        placement: "top",
        title: tour.appSettingsTitle,
        content: tour.appSettingsBody,
        skipBeacon: true,
        before: async () => {
          await ensurePath(pathnameRef.current, "/settings", router);
          await waitForSelector("[data-tour='settings-application']");
        },
      },
      {
        id: "appearance",
        target: "[data-tour='header-appearance']",
        placement: "bottom",
        title: tour.appearanceTitle,
        content: tour.appearanceBody,
        skipBeacon: true,
      },
      {
        id: "done",
        target: "[data-tour='home-welcome']",
        placement: "center",
        title: tour.doneTitle,
        content: tour.doneBody,
        skipBeacon: true,
        before: async () => {
          await ensurePath(pathnameRef.current, "/", router);
          await waitForSelector("[data-tour='home-welcome']");
        },
      },
    ];
  }, [t, direction, router]);

  const { Tour, controls, on, state } = useJoyride({
    continuous: true,
    scrollToFirstStep: true,
    steps,
    locale: {
      back: t.onboarding.tour.back,
      close: t.onboarding.tour.close,
      last: t.onboarding.tour.finish,
      next: t.onboarding.tour.next,
      nextWithProgress: t.onboarding.tour.nextWithProgress,
      skip: t.onboarding.tour.skip,
    },
    options: {
      buttons: ["back", "skip", "primary"],
      closeButtonAction: "skip",
      overlayClickAction: false,
      showProgress: true,
      skipBeacon: true,
      targetWaitTimeout: 5000,
      beforeTimeout: 8000,
      zIndex: 80,
      primaryColor: theme.primaryColor,
      backgroundColor: theme.backgroundColor,
      textColor: theme.textColor,
      overlayColor: theme.overlayColor,
      arrowColor: theme.arrowColor,
      spotlightRadius: 8,
    },
  });

  useEffect(() => {
    return on("tour:end", (data) => {
      if (!profileId) return;
      if (
        data.status === STATUS.FINISHED ||
        data.status === STATUS.SKIPPED
      ) {
        markProductTourCompleted(profileId);
      }
    });
  }, [on, profileId]);

  // Auto-start once per profile after first-run setup (empty library path).
  useEffect(() => {
    if (!desktop || loading || initialSetupPending || !profileId) return;
    if (isProductTourCompleted(profileId)) return;
    if (autoStartedForProfile.current === profileId) return;
    if (state.status === STATUS.RUNNING || state.status === STATUS.WAITING) {
      return;
    }
    autoStartedForProfile.current = profileId;
    const timer = window.setTimeout(() => {
      controls.start();
    }, 450);
    return () => window.clearTimeout(timer);
  }, [
    desktop,
    loading,
    initialSetupPending,
    profileId,
    controls,
    state.status,
  ]);

  // Explicit restart from Home / Settings.
  useEffect(() => {
    if (startToken === 0 || !desktop || !profileId) return;
    controls.start();
  }, [startToken, desktop, profileId, controls]);

  if (!desktop) return null;

  return Tour;
}
