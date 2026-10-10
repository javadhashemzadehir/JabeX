"use client";

import { cn } from "@/lib/utils";

export function HighlightText({
  text,
  query,
  className,
  dir,
}: {
  text: string;
  query: string;
  className?: string;
  dir?: "rtl" | "ltr" | "auto";
}) {
  const parts = splitHighlights(text, query);
  return (
    <span className={cn(className)} dir={dir ?? "auto"}>
      {parts.map((part, index) =>
        part.highlight ? (
          <mark
            key={`${part.text}-${index}`}
            className="rounded-sm bg-primary/20 px-0.5 text-foreground"
          >
            {part.text}
          </mark>
        ) : (
          <span key={`${part.text}-${index}`}>{part.text}</span>
        ),
      )}
    </span>
  );
}

function splitHighlights(
  text: string,
  query: string,
): Array<{ text: string; highlight: boolean }> {
  const terms = query
    .trim()
    .split(/\s+/)
    .filter((t) => t.length > 0);
  if (!text || terms.length === 0) {
    return [{ text, highlight: false }];
  }

  const lower = text.toLocaleLowerCase();
  const ranges: Array<{ start: number; end: number }> = [];
  for (const term of terms) {
    const needle = term.toLocaleLowerCase();
    let from = 0;
    while (from < lower.length) {
      const idx = lower.indexOf(needle, from);
      if (idx === -1) break;
      ranges.push({ start: idx, end: idx + needle.length });
      from = idx + needle.length;
    }
  }
  if (ranges.length === 0) {
    return [{ text, highlight: false }];
  }
  ranges.sort((a, b) => a.start - b.start);
  const merged: Array<{ start: number; end: number }> = [];
  for (const range of ranges) {
    const last = merged[merged.length - 1];
    if (last && range.start <= last.end) {
      last.end = Math.max(last.end, range.end);
    } else {
      merged.push({ ...range });
    }
  }

  const parts: Array<{ text: string; highlight: boolean }> = [];
  let cursor = 0;
  for (const range of merged) {
    if (range.start > cursor) {
      parts.push({ text: text.slice(cursor, range.start), highlight: false });
    }
    parts.push({ text: text.slice(range.start, range.end), highlight: true });
    cursor = range.end;
  }
  if (cursor < text.length) {
    parts.push({ text: text.slice(cursor), highlight: false });
  }
  return parts;
}
