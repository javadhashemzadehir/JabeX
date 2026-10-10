"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

type ListSkeletonProps = {
  rows?: number;
  className?: string;
  /** Show a trailing action stub (e.g. edit/delete icons). */
  withActions?: boolean;
};

/** Placeholder for bordered list rows (decks, cards). */
export function ListSkeleton({
  rows = 4,
  className,
  withActions = true,
}: ListSkeletonProps) {
  return (
    <ul
      aria-busy="true"
      aria-hidden
      className={cn(
        "divide-y divide-border rounded-lg border border-border/60",
        className,
      )}
    >
      {Array.from({ length: rows }, (_, index) => (
        <li
          key={index}
          className="flex items-center justify-between gap-3 px-4 py-3"
        >
          <div className="min-w-0 flex-1 space-y-2">
            <Skeleton className="h-4 w-2/5 max-w-48" />
            <Skeleton className="h-3 w-3/5 max-w-72" />
          </div>
          {withActions ? (
            <div className="flex shrink-0 gap-2">
              <Skeleton className="size-8 rounded-md" />
              <Skeleton className="size-8 rounded-md" />
            </div>
          ) : null}
        </li>
      ))}
    </ul>
  );
}
