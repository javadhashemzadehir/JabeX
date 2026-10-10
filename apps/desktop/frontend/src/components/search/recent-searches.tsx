"use client";

import type { PopularSearch, RecentSearch } from "@jabex/types";
import { Clock, Flame, Trash2, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  CommandGroup,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { useTranslations } from "@/hooks/use-locale";

export function RecentSearches({
  items,
  popular,
  onSelect,
  onClear,
  onRemove,
}: {
  items: RecentSearch[];
  popular: PopularSearch[];
  onSelect: (query: string) => void;
  onClear: () => void;
  onRemove: (id: string) => void;
}) {
  const t = useTranslations();
  const empty = items.length === 0 && popular.length === 0;

  if (empty) {
    return (
      <CommandList>
        <div className="px-3 py-6 text-center text-sm text-muted-foreground">
          {t.search.recentEmpty}
        </div>
      </CommandList>
    );
  }

  return (
    <>
      {items.length > 0 ? (
        <>
          <div className="flex items-center justify-between gap-2 border-b px-3 py-2">
            <span className="text-xs font-medium text-muted-foreground">
              {t.search.recent}
            </span>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs"
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                onClear();
              }}
            >
              <Trash2 className="me-1 h-3 w-3" aria-hidden />
              {t.search.clearRecent}
            </Button>
          </div>
          <CommandList className={popular.length > 0 ? "max-h-48" : undefined}>
            <CommandGroup>
              {items.map((item) => (
                <CommandItem
                  key={item.id}
                  value={`recent:${item.id}`}
                  onSelect={() => onSelect(item.query)}
                  className="group"
                >
                  <Clock
                    className="h-4 w-4 text-muted-foreground"
                    aria-hidden
                  />
                  <span className="min-w-0 flex-1 truncate" dir="auto">
                    {item.query}
                  </span>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 shrink-0 opacity-70 hover:opacity-100"
                    aria-label={t.search.removeRecent}
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      onRemove(item.id);
                    }}
                    onPointerDown={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                    }}
                  >
                    <X className="h-3.5 w-3.5" aria-hidden />
                  </Button>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </>
      ) : null}

      {popular.length > 0 ? (
        <>
          <div className="flex items-center justify-between gap-2 border-b border-t px-3 py-2">
            <span className="text-xs font-medium text-muted-foreground">
              {t.search.popular}
            </span>
          </div>
          <CommandList>
            <CommandGroup>
              {popular.map((item) => (
                <CommandItem
                  key={`popular:${item.query}`}
                  value={`popular:${item.query}`}
                  onSelect={() => onSelect(item.query)}
                >
                  <Flame
                    className="h-4 w-4 text-muted-foreground"
                    aria-hidden
                  />
                  <span className="min-w-0 flex-1 truncate" dir="auto">
                    {item.query}
                  </span>
                  <span className="ms-auto text-xs tabular-nums text-muted-foreground">
                    {item.hitCount}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </>
      ) : null}
    </>
  );
}
