"use client";

import { create } from "zustand";
import type { StudyConsistencyStats } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface StudyProgressState {
  stats: StudyConsistencyStats | null;
  loading: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: () => Promise<void>;
}

export const useStudyProgressStore = create<StudyProgressState>((set) => ({
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
      const stats = await api.getStudyConsistencyStats();
      set({ stats, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
}));
