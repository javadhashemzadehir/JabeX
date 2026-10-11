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

function livePathname(fallback: string) {
  if (typeof window === "undefined") return fallback;
  return window.location.pathname || fallback;
}

async function ensurePath(
  fallbackPathname: string,
  href: string,
  router: { push: (url: string) => void },
) {
  const pathname = livePathname(fallbackPathname);
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
  const startedToken = useOnboardingTourStore((s) => s.startedToken);
  const autoStartedProfileId = useOnboardingTourStore((s) => s.autoStartedProfileId);
  const dismissed = useOnboardingTourStore((s) => s.dismissed);
  const markStarted = useOnboardingTourStore((s) => s.markStarted);
  const markAutoStarted = useOnboardingTourStore((s) => s.markAutoStarted);
  const dismissTour = useOnboardingTourStore((s) => s.dismiss);

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
        isFixed: true,
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
        isFixed: true,
        skipBeacon: true,
      },
      {
        id: "create-deck",
        target: "[data-tour='create-deck']",
        placement: "bottom",
        title: tour.createDeckTitle,
        content: tour.createDeckBody,
        isFixed: true,
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
        isFixed: true,
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
        isFixed: true,
        skipBeacon: true,
      },
      {
        id: "nav-settings",
        target: "[data-tour='nav-settings']",
        placement: direction === "rtl" ? "left" : "right",
        title: tour.settingsTitle,
        content: tour.settingsBody,
        isFixed: true,
        skipBeacon: true,
      },
      {
        id: "settings-application",
        target: "[data-tour='settings-application']",
        placement: "top",
        title: tour.appSettingsTitle,
        content: tour.appSettingsBody,
        isFixed: true,
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
        isFixed: true,
        skipBeacon: true,
      },
      {
        id: "done",
        target: "[data-tour='home-welcome']",
        placement: "center",
        title: tour.doneTitle,
        content: tour.doneBody,
        isFixed: true,
        skipBeacon: true,
        before: async () => {
          await ensurePath(pathnameRef.current, "/", router);
          await waitForSelector("[data-tour='home-welcome']");
        },
      },
    ];
  }, [t, direction, router]);

  const { Tour, controls, on } = useJoyride({
    continuous: true,
    scrollToFirstStep: false,
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
      blockTargetInteraction: true,
      overlayClickAction: false,
      showProgress: true,
      skipBeacon: true,
      // Scrolling a sidebar target can leave the tooltip at opacity 0, so Next/Skip never appear.
      skipScroll: true,
      targetWaitTimeout: 5000,
      beforeTimeout: 8000,
      zIndex: 10000,
      primaryColor: theme.primaryColor,
      backgroundColor: theme.backgroundColor,
      textColor: theme.textColor,
      overlayColor: theme.overlayColor,
      arrowColor: theme.arrowColor,
      spotlightRadius: 8,
    },
  });

  useEffect(() => {
    return on("tour:end", () => {
      const status = controls.info().status;
      if (status !== STATUS.FINISHED && status !== STATUS.SKIPPED) return;
      if (profileId) markProductTourCompleted(profileId);
      dismissTour();
    });
  }, [on, profileId, controls, dismissTour]);

  // Auto-start once per profile after first-run setup.
  // The claim lives in the store so a route change cannot start the tour again.
  useEffect(() => {
    if (!desktop || loading || initialSetupPending || !profileId) return;
    if (dismissed || isProductTourCompleted(profileId)) return;
    if (autoStartedProfileId === profileId) return;
    const timer = window.setTimeout(() => {
      const tour = useOnboardingTourStore.getState();
      if (tour.dismissed || tour.autoStartedProfileId === profileId) return;
      if (isProductTourCompleted(profileId)) return;
      const status = controls.info().status;
      if (status === STATUS.RUNNING || status === STATUS.WAITING) {
        markAutoStarted(profileId);
        return;
      }
      markAutoStarted(profileId);
      controls.start(0);
    }, 450);
    return () => window.clearTimeout(timer);
  }, [
    desktop,
    loading,
    initialSetupPending,
    profileId,
    dismissed,
    autoStartedProfileId,
    controls,
    markAutoStarted,
  ]);

  // Explicit restart from Home / Settings. Ignore a token we already started,
  // otherwise remounting the shell replays the tour from step one.
  useEffect(() => {
    if (startToken === 0 || startToken === startedToken) return;
    if (!desktop || !profileId) return;
    markStarted(startToken);
    controls.start(0);
  }, [startToken, startedToken, desktop, profileId, controls, markStarted]);

  if (!desktop) return null;

  return Tour;
}
