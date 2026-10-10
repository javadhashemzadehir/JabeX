"use client";

import { useCallback, useEffect, useSyncExternalStore } from "react";
import { Header } from "@/components/layout/header";
import { Sidebar } from "@/components/layout/sidebar";
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

  useEffect(() => {
    void hydrateProfiles();
  }, [hydrateProfiles]);

  return (
    <div className="flex h-svh overflow-hidden bg-background text-foreground">
      <Sidebar collapsed={collapsed} />
      <div className="flex min-w-0 flex-1 flex-col">
        <Header collapsed={collapsed} onToggleSidebar={toggle} />
        <main className="flex-1 overflow-auto p-6">
          <div className="mx-auto w-full max-w-7xl">{children}</div>
        </main>
      </div>
    </div>
  );
}
