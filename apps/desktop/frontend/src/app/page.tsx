import { AppShell } from "@/components/layout/app-shell";
import { HomePanel } from "@/features/home/home-panel";

export default function HomePage() {
  return (
    <AppShell>
      <HomePanel />
    </AppShell>
  );
}
