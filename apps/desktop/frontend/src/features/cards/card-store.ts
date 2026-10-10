"use client";

import { create } from "zustand";
import type { Card, CardId, Deck, DeckId, RichTextDocument } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

interface CardState {
  deckId: DeckId | null;
  deck: Deck | null;
  cards: Card[];
  loading: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: (deckId: DeckId) => Promise<void>;
  create: (
    frontContent: RichTextDocument,
    backContent: RichTextDocument,
  ) => Promise<Card | null>;
  update: (
    id: CardId,
    frontContent: RichTextDocument,
    backContent: RichTextDocument,
  ) => Promise<Card | null>;
  remove: (id: CardId) => Promise<void>;
  setDeck: (deck: Deck) => void;
  reset: () => void;
}

export const useCardStore = create<CardState>((set, get) => ({
  deckId: null,
  deck: null,
  cards: [],
  loading: false,
  error: null,
  desktop: false,
  hydrate: async (deckId) => {
    if (!isTauri()) {
      set({
        desktop: false,
        loading: false,
        error: null,
        deckId,
        deck: null,
        cards: [],
      });
      return;
    }
    set({ loading: true, error: null, desktop: true, deckId });
    try {
      const [deck, cards] = await Promise.all([
        api.getDeck(deckId),
        api.listCards(deckId),
      ]);
      set({ deck, cards, loading: false });
    } catch (err) {
      set({
        loading: false,
        deck: null,
        cards: [],
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  create: async (frontContent, backContent) => {
    const deckId = get().deckId;
    if (!isTauri() || !deckId) return null;
    const card = await api.createCard(deckId, frontContent, backContent);
    await get().hydrate(deckId);
    return card;
  },
  update: async (id, frontContent, backContent) => {
    const deckId = get().deckId;
    if (!isTauri() || !deckId) return null;
    const card = await api.updateCard(id, frontContent, backContent);
    await get().hydrate(deckId);
    return card;
  },
  remove: async (id) => {
    const deckId = get().deckId;
    if (!isTauri() || !deckId) return;
    await api.deleteCard(id);
    await get().hydrate(deckId);
  },
  setDeck: (deck) => {
    set({ deck });
  },
  reset: () => {
    set({
      deckId: null,
      deck: null,
      cards: [],
      loading: false,
      error: null,
    });
  },
}));
