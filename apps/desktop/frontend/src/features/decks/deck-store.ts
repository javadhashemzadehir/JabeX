"use client";

import { create } from "zustand";
import type { Deck, DeckId } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface DeckState {
  decks: Deck[];
  loading: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: () => Promise<void>;
  create: (name: string, description?: string | null) => Promise<Deck | null>;
  rename: (
    id: DeckId,
    name: string,
    description?: string | null,
  ) => Promise<Deck | null>;
  remove: (id: DeckId) => Promise<void>;
}

export const useDeckStore = create<DeckState>((set, get) => ({
  decks: [],
  loading: false,
  error: null,
  desktop: false,
  hydrate: async () => {
    if (!isTauri()) {
      set({ desktop: false, loading: false, error: null, decks: [] });
      return;
    }
    set({ loading: true, error: null, desktop: true });
    try {
      const decks = await api.listDecks();
      set({ decks, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  create: async (name, description) => {
    if (!isTauri()) return null;
    const deck = await api.createDeck(name, description);
    await get().hydrate();
    return deck;
  },
  rename: async (id, name, description) => {
    if (!isTauri()) return null;
    const deck = await api.renameDeck(id, name, description);
    await get().hydrate();
    return deck;
  },
  remove: async (id) => {
    if (!isTauri()) return;
    await api.deleteDeck(id);
    await get().hydrate();
  },
}));
