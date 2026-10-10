"use client";

import { useSearchParams } from "next/navigation";
import { ReviewPanel } from "./review-panel";

export function ReviewView() {
  const searchParams = useSearchParams();
  const deckId = searchParams.get("deckId");

  return <ReviewPanel deckId={deckId} />;
}
