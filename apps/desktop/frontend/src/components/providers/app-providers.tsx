"use client";

import { DirectionProvider } from "@radix-ui/react-direction";
import { useEffect } from "react";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useLocaleStore } from "@/lib/i18n/locale-store";

function LocaleHydrator({ children }: { children: React.ReactNode }) {
  const hydrate = useLocaleStore((s) => s.hydrate);

  useEffect(() => {
    hydrate();
  }, [hydrate]);

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
          <TooltipProvider delayDuration={200}>
            {children}
            <Toaster richColors closeButton />
          </TooltipProvider>
        </DirectionSync>
      </LocaleHydrator>
    </ThemeProvider>
  );
}
