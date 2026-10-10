"use client";

import { useState } from "react";
import { Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { formatChordLabel } from "@/features/shortcuts/chord";
import {
  useShortcut,
  useShortcutChord,
} from "@/features/shortcuts/use-shortcut";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

import { SearchDialog } from "./search-dialog";

export function GlobalSearch({ className }: { className?: string }) {
  const t = useTranslations();
  const { locale } = useLocale();
  const [open, setOpen] = useState(false);
  const openSearchChord = useShortcutChord("openSearch");
  const shortcutLabel = formatChordLabel(openSearchChord);

  useShortcut("openSearch", () => {
    setOpen((prev) => !prev);
  });

  return (
    <>
      <Button
        type="button"
        variant="outline"
        className={cn(
          "h-9 w-full max-w-md justify-start gap-2 px-3 text-muted-foreground sm:w-72",
          className,
        )}
        onClick={() => setOpen(true)}
        aria-label={t.search.open}
        aria-keyshortcuts={shortcutLabel}
      >
        <Search className="h-4 w-4 shrink-0" aria-hidden />
        <span className="min-w-0 flex-1 truncate text-start text-sm">
          {t.search.placeholder}
        </span>
        <kbd
          className="pointer-events-none hidden h-5 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground sm:inline-flex"
          aria-hidden
        >
          {shortcutLabel}
        </kbd>
      </Button>
      <SearchDialog open={open} onOpenChange={setOpen} key={locale} />
    </>
  );
}
