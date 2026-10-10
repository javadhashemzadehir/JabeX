"use client";

import type { SearchResult } from "@jabex/types";
import {
  BookOpen,
  Command,
  FileText,
  Hash,
  Layers,
  Paperclip,
  Settings,
  StickyNote,
} from "lucide-react";

import { CommandItem } from "@/components/ui/command";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

import { HighlightText } from "./highlight-text";

const ICONS = {
  card: FileText,
  deck: Layers,
  tag: Hash,
  review: BookOpen,
  note: StickyNote,
  attachment: Paperclip,
  setting: Settings,
  command: Command,
} as const;

export function SearchResultItem({
  result,
  query,
  onSelect,
}: {
  result: SearchResult;
  query: string;
  onSelect: (result: SearchResult) => void;
}) {
  const t = useTranslations();
  const Icon = ICONS[result.entityType] ?? FileText;
  const dir =
    result.textDirection === "rtl" || result.textDirection === "ltr"
      ? result.textDirection
      : "auto";

  return (
    <CommandItem
      value={`${result.entityType}:${result.entityId}`}
      onSelect={() => onSelect(result)}
      className="items-start gap-3"
    >
      <Icon className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" aria-hidden />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            {t.search.entity[result.entityType]}
          </span>
          {result.breadcrumb ? (
            <span className="truncate text-xs text-muted-foreground">
              {result.breadcrumb}
            </span>
          ) : null}
        </div>
        <HighlightText
          text={result.title}
          query={query}
          className={cn("block truncate font-medium")}
          dir={dir}
        />
        {result.snippet ? (
          <HighlightText
            text={result.snippet}
            query={query}
            className="mt-0.5 line-clamp-2 text-xs text-muted-foreground"
            dir={dir}
          />
        ) : null}
      </div>
    </CommandItem>
  );
}
