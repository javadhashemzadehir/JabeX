"use client";

import type { SearchFilter } from "@jabex/types";

import { Button } from "@/components/ui/button";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

const FILTERS: SearchFilter[] = [
  "all",
  "cards",
  "decks",
  "tags",
  "reviews",
  "commands",
];

export function SearchFilterBar({
  value,
  onChange,
}: {
  value: SearchFilter;
  onChange: (filter: SearchFilter) => void;
}) {
  const t = useTranslations();

  return (
    <div
      className="flex flex-wrap gap-1 border-b px-3 py-2"
      role="toolbar"
      aria-label={t.search.filtersLabel}
    >
      {FILTERS.map((filter) => (
        <Button
          key={filter}
          type="button"
          size="sm"
          variant={value === filter ? "secondary" : "ghost"}
          className={cn("h-7 rounded-md px-2 text-xs")}
          aria-pressed={value === filter}
          onClick={() => onChange(filter)}
        >
          {t.search.filters[filter]}
        </Button>
      ))}
    </div>
  );
}
