"use client";

import { create } from "zustand";
import type { Card, DeckId, Grade, ReviewOutcome } from "@jabex/types";
import { api, isTauri } from "@/lib/tauri";

const DUE_WINDOW = 40;
const REFETCH_THRESHOLD = 8;

interface ReviewState {
  deckId: DeckId | null;
  queue: Card[];
  revealed: boolean;
  loading: boolean;
  submitting: boolean;
  error: string | null;
  desktop: boolean;
  reviewedCount: number;
  cardShownAt: number | null;
  hydrate: (deckId?: DeckId | null) => Promise<void>;
  reveal: () => void;
  submit: (grade: Grade) => Promise<ReviewOutcome | null>;
  reset: () => void;
}

async function fetchDueWindow(deckId: DeckId | null): Promise<Card[]> {
  return api.listDueCards(deckId, DUE_WINDOW);
}

export const useReviewStore = create<ReviewState>((set, get) => ({
  deckId: null,
  queue: [],
  revealed: false,
  loading: false,
  submitting: false,
  error: null,
  desktop: false,
  reviewedCount: 0,
  cardShownAt: null,
  hydrate: async (deckId = null) => {
    if (!isTauri()) {
      set({
        desktop: false,
        loading: false,
        error: null,
        deckId,
        queue: [],
        revealed: false,
        reviewedCount: 0,
        cardShownAt: null,
      });
      return;
    }
    set({
      loading: true,
      error: null,
      desktop: true,
      deckId,
      revealed: false,
      reviewedCount: 0,
    });
    try {
      const queue = await fetchDueWindow(deckId);
      set({
        queue,
        loading: false,
        cardShownAt: queue.length > 0 ? Date.now() : null,
      });
    } catch (err) {
      set({
        loading: false,
        queue: [],
        cardShownAt: null,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },
  reveal: () => {
    if (get().queue.length === 0) return;
    set({ revealed: true });
  },
  submit: async (grade) => {
    const { queue, revealed, submitting, cardShownAt, deckId } = get();
    const current = queue[0];
    if (!isTauri() || !current || !revealed || submitting) return null;

    set({ submitting: true, error: null });
    try {
      const durationMs =
        cardShownAt != null
          ? Math.max(0, Math.round(Date.now() - cardShownAt))
          : undefined;
      const outcome = await api.submitReview(current.id, grade, durationMs);
      const nextQueue = queue.slice(1);

      // Top up the window when the local buffer runs low.
      if (nextQueue.length <= REFETCH_THRESHOLD) {
        try {
          const fresh = await fetchDueWindow(deckId);
          const seen = new Set(nextQueue.map((c) => c.id));
          for (const card of fresh) {
            if (!seen.has(card.id)) {
              nextQueue.push(card);
              seen.add(card.id);
            }
          }
        } catch {
          // Keep remaining local queue if refetch fails.
        }
      }

      set({
        queue: nextQueue,
        revealed: false,
        submitting: false,
        reviewedCount: get().reviewedCount + 1,
        cardShownAt: nextQueue.length > 0 ? Date.now() : null,
      });
      return outcome;
    } catch (err) {
      set({
        submitting: false,
        error: err instanceof Error ? err.message : String(err),
      });
      throw err;
    }
  },
  reset: () => {
    set({
      deckId: null,
      queue: [],
      revealed: false,
      loading: false,
      submitting: false,
      error: null,
      reviewedCount: 0,
      cardShownAt: null,
    });
  },
}));
