"use client";

import { create } from "zustand";
import type {
  Card,
  CardId,
  CardSummary,
  Deck,
  DeckId,
  RichTextDocument,
} from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

const PAGE_SIZE = 50;

interface CardState {
  deckId: DeckId | null;
  deck: Deck | null;
  cards: CardSummary[];
  total: number;
  hasMore: boolean;
  loading: boolean;
  loadingMore: boolean;
  error: string | null;
  desktop: boolean;
  hydrate: (deckId: DeckId) => Promise<void>;
  loadMore: () => Promise<void>;
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
  getCard: (id: CardId) => Promise<Card | null>;
  setDeck: (deck: Deck) => void;
  reset: () => void;
}

export const useCardStore = create<CardState>((set, get) => ({
  deckId: null,
  deck: null,
  cards: [],
  total: 0,
  hasMore: false,
  loading: false,
  loadingMore: false,
  error: null,
  desktop: false,
  hydrate: async (deckId) => {
    if (!isTauri()) {
      set({
        desktop: false,
        loading: false,
        loadingMore: false,
        error: null,
        deckId,
        deck: null,
        cards: [],
        total: 0,
        hasMore: false,
      });
      return;
    }
    set({
      loading: true,
      loadingMore: false,
      error: null,
      desktop: true,
      deckId,
      cards: [],
      total: 0,
      hasMore: false,
    });
    try {
      const [deck, page] = await Promise.all([
        api.getDeck(deckId),
        api.listCards(deckId, PAGE_SIZE, 0),
      ]);
      set({
        deck,
        cards: page.items,
        total: page.total,
        hasMore: page.hasMore,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        deck: null,
        cards: [],
        total: 0,
        hasMore: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  loadMore: async () => {
    const { deckId, cards, hasMore, loading, loadingMore } = get();
    if (!isTauri() || !deckId || !hasMore || loading || loadingMore) return;
    set({ loadingMore: true, error: null });
    try {
      const page = await api.listCards(deckId, PAGE_SIZE, cards.length);
      set({
        cards: [...cards, ...page.items],
        total: page.total,
        hasMore: page.hasMore,
        loadingMore: false,
      });
    } catch (err) {
      set({
        loadingMore: false,
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
  getCard: async (id) => {
    if (!isTauri()) return null;
    return api.getCard(id);
  },
  setDeck: (deck) => {
    set({ deck });
  },
  reset: () => {
    set({
      deckId: null,
      deck: null,
      cards: [],
      total: 0,
      hasMore: false,
      loading: false,
      loadingMore: false,
      error: null,
    });
  },
}));
