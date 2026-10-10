"use client";

import { useEffect } from "react";
import { motion } from "framer-motion";
import { toast } from "sonner";
import type { AppPreferences } from "@jabex/types";
import { SettingsSkeleton } from "@/components/feedback/settings-skeleton";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useAppPreferencesStore } from "./app-preferences-store";

type BoolKey = keyof AppPreferences;

const toggles: {
  key: BoolKey;
  labelKey: "enableTray" | "closeToTray" | "startMinimized" | "launchOnStartup";
  hintKey:
    | "enableTrayHint"
    | "closeToTrayHint"
    | "startMinimizedHint"
    | "launchOnStartupHint";
  requiresTray?: boolean;
}[] = [
  {
    key: "enableMinimizeToTray",
    labelKey: "enableTray",
    hintKey: "enableTrayHint",
  },
  {
    key: "closeButtonHidesToTray",
    labelKey: "closeToTray",
    hintKey: "closeToTrayHint",
    requiresTray: true,
  },
  {
    key: "startMinimized",
    labelKey: "startMinimized",
    hintKey: "startMinimizedHint",
    requiresTray: true,
  },
  {
    key: "launchOnStartup",
    labelKey: "launchOnStartup",
    hintKey: "launchOnStartupHint",
  },
];

export function ApplicationSettingsPanel() {
  const t = useTranslations();
  const tauri = useIsTauri();
  const { preferences, loading, saving, desktop, hydrate, update } =
    useAppPreferencesStore();

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.settings.desktopOnly}</p>
    );
  }

  async function onToggle(key: BoolKey, checked: boolean) {
    try {
      await update({ [key]: checked });
      toast.success(t.settings.saved);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.settings.error);
      await hydrate();
    }
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className="space-y-6"
    >
      <div className="space-y-1">
        <h3 className="text-lg font-semibold tracking-tight">
          {t.settings.application.title}
        </h3>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.settings.application.description}
        </p>
      </div>

      {loading ? (
        <SettingsSkeleton />
      ) : (
        <ul className="divide-y divide-border rounded-lg border border-border/60">
          {toggles.map((item) => {
            const id = `pref-${item.key}`;
            const checked = preferences[item.key];
            const disabled =
              saving ||
              (Boolean(item.requiresTray) && !preferences.enableMinimizeToTray);
            return (
              <li
                key={item.key}
                className="flex items-start justify-between gap-4 px-4 py-3"
              >
                <div className="min-w-0 space-y-1">
                  <Label htmlFor={id} className="font-medium">
                    {t.settings.application[item.labelKey]}
                  </Label>
                  <p className="text-sm text-muted-foreground text-pretty">
                    {t.settings.application[item.hintKey]}
                  </p>
                </div>
                <Switch
                  id={id}
                  className="mt-1"
                  checked={checked}
                  disabled={disabled}
                  onCheckedChange={(next) => void onToggle(item.key, next)}
                />
              </li>
            );
          })}
        </ul>
      )}
    </motion.section>
  );
}
