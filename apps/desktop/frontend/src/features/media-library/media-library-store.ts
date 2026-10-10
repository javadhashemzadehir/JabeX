"use client";

import { create } from "zustand";
import type {
  AttachmentType,
  MediaIntegrityReport,
  MediaLibraryItem,
  MediaLibraryStats,
  MediaPurgeResult,
} from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

export type MediaTypeFilter = "all" | AttachmentType;

interface MediaLibraryState {
  items: MediaLibraryItem[];
  stats: MediaLibraryStats | null;
  integrity: MediaIntegrityReport | null;
  loading: boolean;
  busy: boolean;
  error: string | null;
  desktop: boolean;
  query: string;
  typeFilter: MediaTypeFilter;
  includeDeleted: boolean;
  setQuery: (query: string) => void;
  setTypeFilter: (type: MediaTypeFilter) => void;
  setIncludeDeleted: (value: boolean) => void;
  hydrate: () => Promise<void>;
  softDelete: (id: string) => Promise<void>;
  restore: (id: string) => Promise<void>;
  purgeOne: (id: string) => Promise<MediaPurgeResult>;
  purgeAllDeleted: () => Promise<MediaPurgeResult>;
  removeOrphans: () => Promise<number>;
  verify: () => Promise<MediaIntegrityReport>;
}

function buildFilter(state: {
  query: string;
  typeFilter: MediaTypeFilter;
  includeDeleted: boolean;
}) {
  return {
    includeDeleted: state.includeDeleted,
    attachmentType: state.typeFilter === "all" ? null : state.typeFilter,
    query: state.query.trim() || null,
  };
}

export const useMediaLibraryStore = create<MediaLibraryState>((set, get) => ({
  items: [],
  stats: null,
  integrity: null,
  loading: false,
  busy: false,
  error: null,
  desktop: false,
  query: "",
  typeFilter: "all",
  includeDeleted: false,

  setQuery: (query) => set({ query }),
  setTypeFilter: (typeFilter) => set({ typeFilter }),
  setIncludeDeleted: (includeDeleted) => set({ includeDeleted }),

  hydrate: async () => {
    if (!isTauri()) {
      set({
        desktop: false,
        loading: false,
        error: null,
        items: [],
        stats: null,
      });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const filter = buildFilter(get());
      const [items, stats] = await Promise.all([
        api.listMediaLibrary(filter),
        api.getMediaLibraryStats(),
      ]);
      set({ items, stats, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },

  softDelete: async (id) => {
    set({ busy: true, error: null });
    try {
      await api.deleteAttachment(id);
      await get().hydrate();
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },

  restore: async (id) => {
    set({ busy: true, error: null });
    try {
      await api.restoreAttachment(id);
      await get().hydrate();
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },

  purgeOne: async (id) => {
    set({ busy: true, error: null });
    try {
      const result = await api.purgeAttachment(id);
      await get().hydrate();
      return result;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },

  purgeAllDeleted: async () => {
    set({ busy: true, error: null });
    try {
      const result = await api.purgeDeletedAttachments();
      await get().hydrate();
      return result;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },

  removeOrphans: async () => {
    set({ busy: true, error: null });
    try {
      const removed = await api.removeOrphanMediaDirs();
      await get().hydrate();
      return removed;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },

  verify: async () => {
    set({ busy: true, error: null });
    try {
      const integrity = await api.verifyMediaLibrary();
      set({ integrity });
      await get().hydrate();
      return integrity;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    } finally {
      set({ busy: false });
    }
  },
}));
