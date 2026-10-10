"use client";

import { useCallback, useEffect, useState, useSyncExternalStore } from "react";
import { AppPreloader } from "@/components/feedback/app-preloader";
import { Header } from "@/components/layout/header";
import { Sidebar } from "@/components/layout/sidebar";
import { SkipToContent } from "@/components/layout/skip-to-content";
import { InitialProfileSetupDialog } from "@/features/profiles/initial-profile-setup-dialog";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";

const SIDEBAR_STORAGE_KEY = "jabex.sidebar.collapsed";

const listeners = new Set<() => void>();

function readCollapsed() {
  try {
    return window.localStorage.getItem(SIDEBAR_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

function writeCollapsed(collapsed: boolean) {
  try {
    window.localStorage.setItem(SIDEBAR_STORAGE_KEY, String(collapsed));
  } catch {
    // Ignore storage failures (private mode, etc.)
  }
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function useSidebarCollapsed() {
  const collapsed = useSyncExternalStore(subscribe, readCollapsed, () => false);
  const toggle = useCallback(() => {
    writeCollapsed(!readCollapsed());
  }, []);
  return { collapsed, toggle };
}

export function AppShell({ children }: { children: React.ReactNode }) {
  const { collapsed, toggle } = useSidebarCollapsed();
  const hydrateProfiles = useProfileStore((s) => s.hydrate);
  const { hydrated: localeReady } = useLocale();
  const t = useTranslations();
  const [bootReady, setBootReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void hydrateProfiles().finally(() => {
      if (!cancelled) setBootReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, [hydrateProfiles]);

  if (!localeReady || !bootReady) {
    return <AppPreloader />;
  }

  return (
    <div className="flex h-svh overflow-hidden bg-background text-foreground">
      <SkipToContent />
      <Sidebar collapsed={collapsed} />
      <div className="flex min-w-0 flex-1 flex-col">
        <Header collapsed={collapsed} onToggleSidebar={toggle} />
        <main
          id="main-content"
          tabIndex={-1}
          aria-label={t.a11y.main}
          className="flex-1 overflow-auto p-6 outline-none"
        >
          <div className="mx-auto w-full max-w-7xl">{children}</div>
        </main>
      </div>
      <InitialProfileSetupDialog />
    </div>
  );
}
