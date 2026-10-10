"use client";

import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useTranslations } from "@/hooks/use-locale";
import { CardsPanel } from "./cards-panel";

export function DeckCardsView() {
  const t = useTranslations();
  const searchParams = useSearchParams();
  const deckId = searchParams.get("id");

  if (!deckId) {
    return (
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">{t.cards.notFound}</p>
        <Button variant="outline" asChild>
          <Link href="/decks">{t.cards.backToDecks}</Link>
        </Button>
      </div>
    );
  }

  return <CardsPanel deckId={deckId} />;
}
