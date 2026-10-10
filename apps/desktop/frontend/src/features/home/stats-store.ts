"use client";

import { create } from "zustand";
import type { StudyStats } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface StatsState {
  stats: StudyStats | null;
  loading: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: () => Promise<void>;
}

export const useStatsStore = create<StatsState>((set) => ({
  stats: null,
  loading: false,
  error: null,
  desktop: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({ desktop: false, loading: false, error: null, stats: null });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const stats = await api.getStudyStats();
      set({ stats, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
}));
