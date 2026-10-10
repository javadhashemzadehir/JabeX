"use client";

import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { LanguageSwitcher } from "@/components/language-switcher";
import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ProfileSwitcher } from "@/features/profiles/profile-switcher";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

export function Header({
  collapsed,
  onToggleSidebar,
}: {
  collapsed: boolean;
  onToggleSidebar: () => void;
}) {
  const t = useTranslations();
  const { direction } = useLocale();
  const ToggleIcon = collapsed ? PanelLeftOpen : PanelLeftClose;
  const toggleLabel = collapsed ? t.sidebar.expand : t.sidebar.collapse;

  return (
    <header className="flex h-14 items-center justify-between gap-4 border-b border-border bg-background/80 px-4 backdrop-blur supports-backdrop-filter:bg-background/60">
      <div className="flex min-w-0 items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="shrink-0"
              aria-label={toggleLabel}
              aria-expanded={!collapsed}
              aria-controls="app-sidebar"
              onClick={onToggleSidebar}
            >
              <ToggleIcon
                aria-hidden
                className={cn(direction === "rtl" && "rotate-180")}
              />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">{toggleLabel}</TooltipContent>
        </Tooltip>

        <div className="min-w-0">
          <h1 className="truncate text-sm font-semibold tracking-tight">
            {t.header.title}
          </h1>
          <p className="truncate text-xs text-muted-foreground">
            {t.header.foundation}
          </p>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-2">
        <ProfileSwitcher />
        <LanguageSwitcher />
        <ThemeToggle />
      </div>
    </header>
  );
}
