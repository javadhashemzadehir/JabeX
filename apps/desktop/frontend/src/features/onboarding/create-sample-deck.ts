import type { Deck } from "@jabex/types";
import { api } from "@/lib/tauri";
import { richTextFromPlainText } from "@/lib/rich-text";

export interface SampleDeckCopy {
  name: string;
  description: string;
  cards: Array<{ front: string; back: string }>;
}

/** Creates one local sample deck with Leitner-ready cards (no network). */
export async function createSampleDeck(copy: SampleDeckCopy): Promise<Deck> {
  const deck = await api.createDeck(copy.name, copy.description);
  for (const card of copy.cards) {
    await api.createCard(
      deck.id,
      richTextFromPlainText(card.front),
      richTextFromPlainText(card.back),
    );
  }
  return deck;
}
