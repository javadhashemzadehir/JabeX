"use client";

import { Button } from "@/components/ui/button";
import { useLocale } from "@/hooks/use-locale";
import { locales, type Locale } from "@/lib/i18n/config";
import { cn } from "@/lib/utils";

export function LanguageSwitcher() {
  const { locale, setLocale, dictionary } = useLocale();

  return (
    <div
      role="group"
      aria-label={dictionary.language.label}
      className="inline-flex items-center gap-1 rounded-lg border border-border bg-muted/40 p-1"
    >
      {locales.map((code: Locale) => {
        const active = locale === code;
        return (
          <Button
            key={code}
            type="button"
            size="sm"
            variant="ghost"
            aria-pressed={active}
            className={cn(
              "h-7 px-2.5 text-xs",
              active && "bg-background text-foreground shadow-sm",
            )}
            onClick={() => setLocale(code)}
          >
            {dictionary.language[code]}
          </Button>
        );
      })}
    </div>
  );
}
