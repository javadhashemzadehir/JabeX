"use client";

import { Skeleton } from "@/components/ui/skeleton";

/** Generic form / algorithm panel placeholder. */
export function FormSkeleton({ fields = 3 }: { fields?: number }) {
  return (
    <div className="space-y-4" aria-busy="true" aria-hidden>
      {Array.from({ length: fields }, (_, index) => (
        <div key={index} className="space-y-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-9 w-full rounded-md" />
        </div>
      ))}
      <Skeleton className="h-9 w-36 rounded-md" />
    </div>
  );
}
