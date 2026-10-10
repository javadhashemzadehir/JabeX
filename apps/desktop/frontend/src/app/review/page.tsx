import { Suspense } from "react";
import { AppShell } from "@/components/layout/app-shell";
import { ReviewView } from "@/features/review/review-view";

export default function ReviewPage() {
  return (
    <AppShell>
      <Suspense fallback={null}>
        <ReviewView />
      </Suspense>
    </AppShell>
  );
}
