"use client";

import { useTranslations } from "@/hooks/use-locale";

export function SkipToContent() {
  const t = useTranslations();

  return (
    <a
      href="#main-content"
      className="bg-background text-foreground ring-offset-background fixed inset-s-4 top-4 z-100 -translate-y-16 rounded-md px-4 py-2 text-sm font-medium opacity-0 shadow-md transition-[transform,opacity] focus:translate-y-0 focus:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {t.a11y.skipToContent}
    </a>
  );
}
