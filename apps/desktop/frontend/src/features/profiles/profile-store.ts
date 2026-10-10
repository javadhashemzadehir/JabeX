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
  hydrate: () => Promise<void>;
  create: (displayName: string) => Promise<Profile | null>;
  switchTo: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profiles: [],
  active: null,
  loading: false,
  error: null,
  desktop: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({ desktop: false, loading: false, error: null });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const [profiles, active] = await Promise.all([
        api.listProfiles(),
        api.getActiveProfile(),
      ]);
      set({ profiles, active, loading: false });
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
}));
