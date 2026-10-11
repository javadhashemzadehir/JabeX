import { Suspense } from "react";
import { DeckCardsView } from "@/features/cards/deck-cards-view";

export default function DeckCardsPage() {
  return (
    <Suspense fallback={null}>
      <DeckCardsView />
    </Suspense>
  );
}
