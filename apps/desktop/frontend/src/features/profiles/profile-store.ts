"use client";

import { create } from "zustand";
import type { Profile } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface ProfileState {
  profiles: Profile[];
  active: Profile | null;
  loading: boolean;
  error: string | null;
  desktop: boolean;
  initialSetupPending: boolean;
  hydrate: () => Promise<void>;
  create: (displayName: string) => Promise<Profile | null>;
  switchTo: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  completeInitialSetup: (displayName: string) => Promise<Profile | null>;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profiles: [],
  active: null,
  loading: false,
  error: null,
  desktop: false,
  initialSetupPending: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({
        desktop: false,
        loading: false,
        error: null,
        initialSetupPending: false,
      });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const [profiles, active, initialSetupPending] = await Promise.all([
        api.listProfiles(),
        api.getActiveProfile(),
        api.isInitialProfileSetupPending(),
      ]);
      set({ profiles, active, initialSetupPending, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  create: async (displayName) => {
    if (!isTauri()) return null;
    const profile = await api.createProfile(displayName);
    await get().hydrate();
    return profile;
  },
  switchTo: async (id) => {
    if (!isTauri()) return;
    await api.switchProfile(id);
    await get().hydrate();
  },
  remove: async (id) => {
    if (!isTauri()) return;
    await api.deleteProfile(id);
    await get().hydrate();
  },
  completeInitialSetup: async (displayName) => {
    if (!isTauri()) return null;
    const profile = await api.completeInitialProfileSetup(displayName);
    await get().hydrate();
    return profile;
  },
}));
