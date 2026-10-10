import { AppShell } from "@/components/layout/app-shell";
import { DecksPanel } from "@/features/decks/decks-panel";

export default function DecksPage() {
  return (
    <AppShell>
      <DecksPanel />
    </AppShell>
  );
}
