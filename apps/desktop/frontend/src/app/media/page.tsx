import { AppShell } from "@/components/layout/app-shell";
import { MediaLibraryPanel } from "@/features/media-library/media-library-panel";

export default function MediaLibraryPage() {
  return (
    <AppShell>
      <MediaLibraryPanel />
    </AppShell>
  );
}
