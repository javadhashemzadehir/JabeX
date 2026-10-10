"use client";

import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { Button } from "@/components/ui/button";
import { useIsClient } from "@/hooks/use-is-client";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

const options = [
  { value: "light", icon: Sun, labelKey: "light" as const },
  { value: "dark", icon: Moon, labelKey: "dark" as const },
  { value: "system", icon: Monitor, labelKey: "system" as const },
];

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const t = useTranslations();
  const isClient = useIsClient();

  if (!isClient) {
    return (
      <div
        className="inline-flex h-9 items-center gap-1 rounded-lg border border-border bg-muted/40 p-1"
        aria-hidden
      >
        <div className="h-7 w-7 rounded-md bg-muted" />
        <div className="h-7 w-7 rounded-md bg-muted" />
        <div className="h-7 w-7 rounded-md bg-muted" />
      </div>
    );
  }

  return (
    <div
      role="group"
      aria-label={t.theme.label}
      className="inline-flex items-center gap-1 rounded-lg border border-border bg-muted/40 p-1"
    >
      {options.map(({ value, icon: Icon, labelKey }) => {
        const active = theme === value;
        return (
          <Button
            key={value}
            type="button"
            size="icon"
            variant="ghost"
            aria-label={t.theme[labelKey]}
            aria-pressed={active}
            className={cn(
              "h-7 w-7",
              active && "bg-background text-foreground shadow-sm",
            )}
            onClick={() => setTheme(value)}
          >
            <Icon />
          </Button>
        );
      })}
    </div>
  );
}
