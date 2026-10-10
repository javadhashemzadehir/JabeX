"use client";

import { useEffect, useRef, useState, useTransition } from "react";
import type {
  PopularSearch,
  RecentSearch,
  SearchFilter,
  SearchResult,
} from "@jabex/types";
import { useRouter } from "next/navigation";

import { CommandDialog } from "@/components/ui/command";
import { useTranslations } from "@/hooks/use-locale";
import { api, isTauri } from "@/lib/tauri";

import { RecentSearches } from "./recent-searches";
import { SearchFilterBar } from "./search-filter";
import { SearchInput } from "./search-input";
import { SearchResults } from "./search-results";

const DEBOUNCE_MS = 180;

export function SearchDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useTranslations();
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<SearchFilter>("all");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [recent, setRecent] = useState<RecentSearch[]>([]);
  const [popular, setPopular] = useState<PopularSearch[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [, startTransition] = useTransition();
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastRecorded = useRef("");

  useEffect(() => {
    if (!open || !isTauri()) return;
    let cancelled = false;
    void Promise.all([
      api.listRecentSearches(12),
      api.listPopularSearches(8),
    ])
      .then(([recentItems, popularItems]) => {
        if (cancelled) return;
        setRecent(recentItems);
        setPopular(popularItems);
      })
      .catch(() => {
        if (cancelled) return;
        setRecent([]);
        setPopular([]);
      });
    return () => {
      cancelled = true;
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const trimmed = query.trim();
    if (!trimmed || !isTauri()) {
      return;
    }

    if (debounceRef.current) clearTimeout(debounceRef.current);
    let cancelled = false;
    debounceRef.current = setTimeout(() => {
      startTransition(() => {
        setLoading(true);
        setError(null);
        void api
          .searchGlobal(trimmed, filter, 40, false)
          .then((response) => {
            if (cancelled) return;
            setResults(response.results);
            setLoading(false);
          })
          .catch((err) => {
            if (cancelled) return;
            setResults([]);
            setLoading(false);
            setError(err instanceof Error ? err.message : t.search.error);
          });
      });
    }, DEBOUNCE_MS);

    return () => {
      cancelled = true;
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query, filter, open, t.search.error]);

  // Persist recent after the user pauses on a query with results.
  useEffect(() => {
    if (!open || !isTauri()) return;
    const trimmed = query.trim();
    if (!trimmed || results.length === 0) return;
    if (lastRecorded.current === `${trimmed}|${filter}`) return;
    const timer = setTimeout(() => {
      lastRecorded.current = `${trimmed}|${filter}`;
      void api.searchGlobal(trimmed, filter, 1, true).then(() => {
        void api.listRecentSearches(12).then(setRecent).catch(() => undefined);
      });
    }, 900);
    return () => clearTimeout(timer);
  }, [query, filter, results.length, open]);

  function handleOpenChange(next: boolean) {
    if (!next) {
      setQuery("");
      setResults([]);
      setFilter("all");
      setLoading(false);
      setError(null);
      lastRecorded.current = "";
    }
    onOpenChange(next);
  }

  async function handleSelect(result: SearchResult) {
    const q = query.trim();
    if (isTauri() && q) {
      try {
        await api.searchGlobal(q, filter, 1, true);
        await api.recordSearchSelection(q, result.entityType, result.entityId);
      } catch {
        // Local history is best-effort.
      }
    }
    handleOpenChange(false);
    if (result.href) {
      router.push(result.href);
    }
  }

  async function handleClearRecent() {
    if (!isTauri()) return;
    try {
      await api.clearRecentSearches();
      setRecent([]);
    } catch {
      // ignore
    }
  }

  async function handleRemoveRecent(id: string) {
    if (!isTauri()) return;
    try {
      await api.deleteRecentSearch(id);
      setRecent((prev) => prev.filter((item) => item.id !== id));
    } catch {
      // ignore
    }
  }

  const showRecent = query.trim().length === 0;
  const visibleResults = showRecent ? [] : results;
  const visibleLoading = showRecent ? false : loading;

  return (
    <CommandDialog
      open={open}
      onOpenChange={handleOpenChange}
      title={t.search.title}
    >
      <SearchInput
        value={query}
        onValueChange={(value) => {
          setQuery(value);
          setError(null);
          if (!value.trim()) {
            setResults([]);
            setLoading(false);
          }
        }}
        placeholder={t.search.placeholder}
      />
      <SearchFilterBar value={filter} onChange={setFilter} />
      {showRecent ? (
        <RecentSearches
          items={recent}
          popular={popular}
          onSelect={setQuery}
          onClear={() => void handleClearRecent()}
          onRemove={(id) => void handleRemoveRecent(id)}
        />
      ) : (
        <SearchResults
          results={visibleResults}
          query={query}
          loading={visibleLoading}
          error={error}
          onSelect={(result) => void handleSelect(result)}
        />
      )}
      {!isTauri() ? (
        <p className="border-t px-3 py-2 text-xs text-muted-foreground">
          {t.search.desktopOnly}
        </p>
      ) : null}
    </CommandDialog>
  );
}
