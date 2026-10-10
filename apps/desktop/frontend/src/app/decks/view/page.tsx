import { Suspense } from "react";
import { AppShell } from "@/components/layout/app-shell";
import { DeckCardsView } from "@/features/cards/deck-cards-view";

export default function DeckCardsPage() {
  return (
    <AppShell>
      <Suspense fallback={null}>
        <DeckCardsView />
      </Suspense>
    </AppShell>
  );
}
