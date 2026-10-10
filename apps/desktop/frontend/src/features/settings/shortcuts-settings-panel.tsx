"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { SettingsSkeleton } from "@/components/feedback/settings-skeleton";
import { chordFromEvent, chordsConflict, formatChordLabel } from "@/features/shortcuts/chord";
import {
  DEFAULT_KEYBOARD_SHORTCUTS,
  SHORTCUT_ACTION_ORDER,
  type ShortcutActionId,
} from "@/features/shortcuts/defaults";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";

import { useAppPreferencesStore } from "./app-preferences-store";

export function ShortcutsSettingsPanel() {
  const t = useTranslations();
  const tauri = useIsTauri();
  const { preferences, loading, saving, desktop, hydrate, setShortcut, resetShortcuts } =
    useAppPreferencesStore();
  const [listening, setListening] = useState<ShortcutActionId | null>(null);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  useEffect(() => {
    if (!listening) return;

    function onKeyDown(event: KeyboardEvent) {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        setListening(null);
        return;
      }
      const chord = chordFromEvent(event);
      if (!chord) return;

      const conflict = SHORTCUT_ACTION_ORDER.find((action) => {
        if (action === listening) return false;
        return chordsConflict(
          preferences.keyboardShortcuts[action],
          chord,
        );
      });
      if (conflict) {
        toast.error(
          t.settings.shortcuts.conflict.replace(
            "{action}",
            t.settings.shortcuts.actions[conflict],
          ),
        );
        setListening(null);
        return;
      }

      void setShortcut(listening!, chord)
        .then(() => {
          toast.success(t.settings.saved);
          setListening(null);
        })
        .catch((err) => {
          toast.error(err instanceof Error ? err.message : t.settings.error);
          setListening(null);
        });
    }

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [listening, preferences.keyboardShortcuts, setShortcut, t]);

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.settings.desktopOnly}</p>
    );
  }

  async function onReset() {
    try {
      await resetShortcuts();
      toast.success(t.settings.saved);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.settings.error);
    }
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className="space-y-6"
    >
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div className="space-y-1">
          <h3 className="text-lg font-semibold tracking-tight">
            {t.settings.shortcuts.title}
          </h3>
          <p className="text-sm text-muted-foreground text-pretty">
            {t.settings.shortcuts.description}
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={saving || loading}
          onClick={() => void onReset()}
        >
          {t.settings.shortcuts.resetAll}
        </Button>
      </div>

      {loading ? (
        <SettingsSkeleton />
      ) : (
        <ul className="divide-y divide-border rounded-lg border border-border/60">
          {SHORTCUT_ACTION_ORDER.map((action) => {
            const chord =
              preferences.keyboardShortcuts[action] ??
              DEFAULT_KEYBOARD_SHORTCUTS[action];
            const isListening = listening === action;
            return (
              <li
                key={action}
                className="flex items-center justify-between gap-4 px-4 py-3"
              >
                <div className="min-w-0 space-y-1">
                  <p className="font-medium">
                    {t.settings.shortcuts.actions[action]}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {isListening
                      ? t.settings.shortcuts.pressKeys
                      : formatChordLabel(chord)}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <kbd className="hidden rounded border bg-muted px-2 py-1 font-mono text-xs text-muted-foreground sm:inline-flex">
                    {formatChordLabel(chord)}
                  </kbd>
                  <Button
                    type="button"
                    variant={isListening ? "default" : "outline"}
                    size="sm"
                    disabled={saving}
                    onClick={() =>
                      setListening((prev) => (prev === action ? null : action))
                    }
                  >
                    {isListening
                      ? t.settings.shortcuts.cancel
                      : t.settings.shortcuts.change}
                  </Button>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </motion.section>
  );
}
