"use client";

import { Skeleton } from "@/components/ui/skeleton";

/** Preference rows while app settings load. */
export function SettingsSkeleton({ rows = 4 }: { rows?: number }) {
  return (
    <ul
      aria-busy="true"
      aria-hidden
      className="divide-y divide-border rounded-lg border border-border/60"
    >
      {Array.from({ length: rows }, (_, index) => (
        <li
          key={index}
          className="flex items-start justify-between gap-4 px-4 py-3"
        >
          <div className="min-w-0 flex-1 space-y-2">
            <Skeleton className="h-4 w-40" />
            <Skeleton className="h-3 w-full max-w-md" />
          </div>
          <Skeleton className="mt-1 h-5 w-9 rounded-full" />
        </li>
      ))}
    </ul>
  );
}
