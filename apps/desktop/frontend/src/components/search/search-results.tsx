"use client";

import type { SearchResult } from "@jabex/types";

import {
  CommandEmpty,
  CommandGroup,
  CommandList,
} from "@/components/ui/command";
import { useTranslations } from "@/hooks/use-locale";

import { SearchResultItem } from "./search-result-item";

export function SearchResults({
  results,
  query,
  loading,
  error,
  onSelect,
}: {
  results: SearchResult[];
  query: string;
  loading: boolean;
  error?: string | null;
  onSelect: (result: SearchResult) => void;
}) {
  const t = useTranslations();

  return (
    <CommandList>
      {!loading && error ? (
        <CommandEmpty>
          <span className="text-destructive" role="alert">
            {error}
          </span>
        </CommandEmpty>
      ) : null}
      {!loading && !error && query.trim() && results.length === 0 ? (
        <CommandEmpty>{t.search.empty}</CommandEmpty>
      ) : null}
      {results.length > 0 ? (
        <CommandGroup heading={t.search.results}>
          {results.map((result) => (
            <SearchResultItem
              key={`${result.entityType}:${result.entityId}`}
              result={result}
              query={query}
              onSelect={onSelect}
            />
          ))}
        </CommandGroup>
      ) : null}
      {loading ? (
        <div className="px-3 py-4 text-center text-sm text-muted-foreground">
          {t.search.loading}
        </div>
      ) : null}
    </CommandList>
  );
}
