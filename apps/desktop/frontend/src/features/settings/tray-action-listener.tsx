"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import type { TrayActionPayload } from "@jabex/types";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useProfileSwitcherUi } from "@/features/profiles/profile-switcher-ui";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { isTauri } from "@/lib/tauri";

const TRAY_ACTION_EVENT = "tray://action";

/**
 * Listens for tray menu actions that need UI (settings, restore, profile).
 * Backup create is handled in Rust; we only surface toast feedback here.
 * Window show/focus for tray actions is done in Rust before the event is emitted.
 */
export function TrayActionListener() {
  const tauri = useIsTauri();
  const router = useRouter();
  const t = useTranslations();

  useEffect(() => {
    if (!tauri || !isTauri()) return;

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void listen<TrayActionPayload>(TRAY_ACTION_EVENT, (event) => {
      const payload = event.payload;
      switch (payload.action) {
        case "settings":
          router.push("/settings");
          break;
        case "restoreBackup":
          router.push("/settings#backup");
          break;
        case "switch-profile": {
          // Reuse header ProfileSwitcher — refresh list, then open the existing menu.
          void useProfileStore.getState().hydrate();
          // Defer one frame so focus lands after Rust restore/show.
          requestAnimationFrame(() => {
            useProfileSwitcherUi.getState().openMenu();
          });
          break;
        }
        case "backupCreated":
          toast.success(
            t.backup.exported.replace("{path}", payload.path ?? ""),
          );
          break;
        case "backupFailed":
          toast.error(payload.message || t.backup.error);
          break;
        default:
          break;
      }
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [tauri, router, t]);

  return null;
}
