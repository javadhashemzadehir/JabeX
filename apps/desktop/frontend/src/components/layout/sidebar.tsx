"use client";

import { BookOpen, Home, Layers, Settings } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslations, useLocale } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const items = [
  { key: "home" as const, icon: Home, href: "#" },
  { key: "decks" as const, icon: Layers, href: "#" },
  { key: "review" as const, icon: BookOpen, href: "#" },
  { key: "settings" as const, icon: Settings, href: "#" },
];

export function Sidebar({ collapsed }: { collapsed: boolean }) {
  const t = useTranslations();
  const { direction } = useLocale();
  const tooltipSide = direction === "rtl" ? "left" : "right";

  return (
    <aside
      id="app-sidebar"
      className={cn(
        "flex shrink-0 flex-col overflow-hidden border-e border-border bg-sidebar text-sidebar-foreground transition-[width] duration-200 ease-out",
        collapsed ? "w-16" : "w-60",
      )}
    >
      <div
        className={cn(
          "flex h-14 shrink-0 items-center border-b border-border px-4",
          collapsed ? "justify-center" : "gap-2",
        )}
      >
        <motion.img
          src="/brand/mark.png"
          alt={collapsed ? t.sidebar.brand : ""}
          width={32}
          height={32}
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.35 }}
          className="size-8 shrink-0"
        />
        <div
          className={cn(
            "min-w-0 overflow-hidden whitespace-nowrap",
            collapsed && "hidden",
          )}
        >
          <p className="truncate text-sm font-semibold tracking-tight">
            {t.sidebar.brand}
          </p>
          <p className="truncate text-xs text-muted-foreground">
            {t.sidebar.section}
          </p>
        </div>
      </div>

      <nav className="flex flex-1 flex-col gap-1 p-3" aria-label={t.sidebar.section}>
        {items.map((item, index) => {
          const Icon = item.icon;
          const active = item.key === "home";
          const label = t.nav[item.key];
          const link = (
            <motion.a
              key={item.key}
              href={item.href}
              initial={{ opacity: 0, x: direction === "rtl" ? 8 : -8 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.05 * index, duration: 0.25 }}
              className={cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                active
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "text-muted-foreground hover:bg-sidebar-accent/70 hover:text-sidebar-accent-foreground",
              )}
              aria-current={active ? "page" : undefined}
            >
              <Icon className="size-4 shrink-0" />
              <span className={cn("truncate", collapsed && "sr-only")}>{label}</span>
            </motion.a>
          );

          if (!collapsed) return link;

          return (
            <Tooltip key={item.key}>
              <TooltipTrigger asChild>{link}</TooltipTrigger>
              <TooltipContent side={tooltipSide}>{label}</TooltipContent>
            </Tooltip>
          );
        })}
      </nav>

      <div
        className={cn(
          "overflow-hidden border-t border-border",
          collapsed ? "h-0 border-transparent p-0" : "p-4",
        )}
      >
        <p className="text-xs leading-relaxed text-muted-foreground">
          {t.app.privacyNote}
        </p>
      </div>
    </aside>
  );
}
