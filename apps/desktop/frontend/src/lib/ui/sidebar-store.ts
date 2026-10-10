"use client";

import { create } from "zustand";

export const SIDEBAR_STORAGE_KEY = "jabex.sidebar.collapsed";

interface SidebarState {
  collapsed: boolean;
  hydrated: boolean;
  setCollapsed: (collapsed: boolean) => void;
  toggle: () => void;
  hydrate: () => void;
}

function readStoredCollapsed(): boolean {
  try {
    return window.localStorage.getItem(SIDEBAR_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

function writeStoredCollapsed(collapsed: boolean) {
  try {
    window.localStorage.setItem(SIDEBAR_STORAGE_KEY, collapsed ? "1" : "0");
  } catch {
    // Ignore storage failures (private mode, etc.)
  }
}

export const useSidebarStore = create<SidebarState>((set, get) => ({
  collapsed: false,
  hydrated: false,
  setCollapsed: (collapsed) => {
    writeStoredCollapsed(collapsed);
    set({ collapsed });
  },
  toggle: () => {
    get().setCollapsed(!get().collapsed);
  },
  hydrate: () => {
    set({ collapsed: readStoredCollapsed(), hydrated: true });
  },
}));
