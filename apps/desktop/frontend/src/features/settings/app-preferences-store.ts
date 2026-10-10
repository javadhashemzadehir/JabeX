"use client";

import { create } from "zustand";
import type { AppPreferences, KeyboardShortcuts } from "@jabex/types";
import { DEFAULT_KEYBOARD_SHORTCUTS } from "@/features/shortcuts/defaults";
import { api, isTauri } from "@/lib/tauri";

const defaults: AppPreferences = {
  enableMinimizeToTray: true,
  closeButtonHidesToTray: true,
  startMinimized: false,
  launchOnStartup: false,
  keyboardShortcuts: { ...DEFAULT_KEYBOARD_SHORTCUTS },
};

function mergePreferences(prefs: AppPreferences): AppPreferences {
  return {
    ...defaults,
    ...prefs,
    keyboardShortcuts: {
      ...DEFAULT_KEYBOARD_SHORTCUTS,
      ...(prefs.keyboardShortcuts ?? {}),
    },
  };
}

interface AppPreferencesState {
  preferences: AppPreferences;
  loading: boolean;
  saving: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: () => Promise<void>;
  update: (patch: Partial<AppPreferences>) => Promise<AppPreferences | null>;
  setShortcut: (
    action: keyof KeyboardShortcuts,
    chord: string,
  ) => Promise<AppPreferences | null>;
  resetShortcuts: () => Promise<AppPreferences | null>;
}

export const useAppPreferencesStore = create<AppPreferencesState>((set, get) => ({
  preferences: defaults,
  loading: false,
  saving: false,
  error: null,
  desktop: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({ desktop: false, loading: false, error: null, preferences: defaults });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const preferences = mergePreferences(await api.getAppPreferences());
      set({ preferences, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  update: async (patch) => {
    if (!isTauri()) return null;
    const next = mergePreferences({ ...get().preferences, ...patch });
    set({ saving: true, error: null, preferences: next });
    try {
      const preferences = mergePreferences(await api.setAppPreferences(next));
      set({ preferences, saving: false });
      return preferences;
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    }
  },
  setShortcut: async (action, chord) => {
    const keyboardShortcuts = {
      ...get().preferences.keyboardShortcuts,
      [action]: chord,
    };
    return get().update({ keyboardShortcuts });
  },
  resetShortcuts: async () => {
    return get().update({
      keyboardShortcuts: { ...DEFAULT_KEYBOARD_SHORTCUTS },
    });
  },
}));
