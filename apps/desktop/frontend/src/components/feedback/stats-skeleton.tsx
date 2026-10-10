"use client";

import { Skeleton } from "@/components/ui/skeleton";

/** Matches the home study-stats metric grid + box bars. */
export function StatsSkeleton() {
  return (
    <div className="space-y-5" aria-busy="true" aria-hidden>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {Array.from({ length: 4 }, (_, index) => (
          <div
            key={index}
            className="flex min-h-18 flex-col justify-between gap-2 rounded-lg border border-border/60 px-3 py-3"
          >
            <Skeleton className="h-3 w-16" />
            <Skeleton className="h-7 w-12" />
          </div>
        ))}
      </div>
      <div className="space-y-3 rounded-lg border border-border/60 bg-muted/30 px-3 py-3.5 sm:px-4">
        <Skeleton className="h-3 w-24" />
        <div className="grid grid-cols-5 items-end gap-2 sm:gap-3">
          {Array.from({ length: 5 }, (_, index) => (
            <div
              key={index}
              className="flex min-w-0 flex-col items-center gap-1.5"
            >
              <div className="flex h-14 w-full items-end justify-center">
                <Skeleton
                  className="w-full max-w-8 rounded-md"
                  style={{ height: 12 + index * 6 }}
                />
              </div>
              <Skeleton className="h-3 w-3" />
              <Skeleton className="h-3 w-5" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
