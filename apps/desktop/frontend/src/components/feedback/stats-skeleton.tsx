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
            className="space-y-2 rounded-lg border border-border/60 px-3 py-3"
          >
            <Skeleton className="h-3 w-16" />
            <Skeleton className="h-7 w-12" />
          </div>
        ))}
      </div>
      <div className="space-y-2">
        <Skeleton className="h-3 w-24" />
        <div className="grid grid-cols-5 gap-2">
          {Array.from({ length: 5 }, (_, index) => (
            <div key={index} className="flex flex-col items-center gap-1.5">
              <div className="flex h-12 w-full items-end justify-center">
                <Skeleton
                  className="w-full max-w-6 rounded-sm"
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
