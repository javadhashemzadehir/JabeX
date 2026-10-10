"use client";

import { create } from "zustand";
import type { TrashEntityKind, TrashItem } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface TrashState {
  items: TrashItem[];
  loading: boolean;
  busy: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: () => Promise<void>;
  restore: (entityKind: TrashEntityKind, id: string) => Promise<void>;
  permanentlyDelete: (entityKind: TrashEntityKind, id: string) => Promise<void>;
}

export const useTrashStore = create<TrashState>((set, get) => ({
  items: [],
  loading: false,
  busy: false,
  error: null,
  desktop: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({ desktop: false, loading: false, error: null, items: [] });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const items = await api.listDeletedItems();
      set({ items, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  restore: async (entityKind, id) => {
    set({ busy: true, error: null });
    try {
      await api.restoreTrashItem(entityKind, id);
      await get().hydrate();
      set({ busy: false });
    } catch (err) {
      set({
        busy: false,
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    }
  },
  permanentlyDelete: async (entityKind, id) => {
    set({ busy: true, error: null });
    try {
      await api.permanentlyDeleteTrashItem(entityKind, id);
      await get().hydrate();
      set({ busy: false });
    } catch (err) {
      set({
        busy: false,
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    }
  },
}));
