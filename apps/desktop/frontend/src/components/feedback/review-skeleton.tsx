"use client";

import { Skeleton } from "@/components/ui/skeleton";

/** Review card placeholder while due cards fetch. */
export function ReviewSkeleton() {
  return (
    <div
      className="space-y-4 rounded-lg border border-border/60 px-5 py-6"
      aria-busy="true"
      aria-hidden
    >
      <div className="flex items-center justify-between gap-3">
        <Skeleton className="h-5 w-20" />
        <Skeleton className="h-4 w-28" />
      </div>
      <div className="space-y-3 py-8">
        <Skeleton className="mx-auto h-5 w-3/4 max-w-md" />
        <Skeleton className="mx-auto h-5 w-1/2 max-w-xs" />
      </div>
      <div className="flex flex-wrap justify-center gap-2">
        <Skeleton className="h-9 w-24 rounded-md" />
        <Skeleton className="h-9 w-24 rounded-md" />
        <Skeleton className="h-9 w-24 rounded-md" />
        <Skeleton className="h-9 w-24 rounded-md" />
      </div>
    </div>
  );
}
