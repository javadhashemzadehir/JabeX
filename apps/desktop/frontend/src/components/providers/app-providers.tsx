"use client";

import { DirectionProvider } from "@radix-ui/react-direction";
import { MotionConfig } from "framer-motion";
import { useEffect } from "react";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useAppPreferencesStore } from "@/features/settings/app-preferences-store";
import { TrayActionListener } from "@/features/settings/tray-action-listener";
import { useLocaleStore } from "@/lib/i18n/locale-store";

function LocaleHydrator({ children }: { children: React.ReactNode }) {
  const hydrate = useLocaleStore((s) => s.hydrate);
  const hydratePreferences = useAppPreferencesStore((s) => s.hydrate);

  useEffect(() => {
    hydrate();
    void hydratePreferences();
  }, [hydrate, hydratePreferences]);

  return children;
}

function DirectionSync({ children }: { children: React.ReactNode }) {
  const direction = useLocaleStore((s) => s.direction);
  return <DirectionProvider dir={direction}>{children}</DirectionProvider>;
}

export function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider>
      <LocaleHydrator>
        <DirectionSync>
          <MotionConfig reducedMotion="user">
            <TooltipProvider delayDuration={200}>
              {children}
              <TrayActionListener />
              <Toaster richColors closeButton />
            </TooltipProvider>
          </MotionConfig>
        </DirectionSync>
      </LocaleHydrator>
    </ThemeProvider>
  );
}
