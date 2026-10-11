import { Suspense } from "react";
import { ReviewView } from "@/features/review/review-view";

export default function ReviewPage() {
  return (
    <Suspense fallback={null}>
      <ReviewView />
    </Suspense>
  );
}
