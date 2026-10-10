"use client";

import { useEffect, useEffectEvent } from "react";

import { useAppPreferencesStore } from "@/features/settings/app-preferences-store";

import { matchesChord } from "./chord";
import {
  DEFAULT_KEYBOARD_SHORTCUTS,
  type ShortcutActionId,
} from "./defaults";

function isTypingTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable
  );
}

/**
 * Invoke `handler` when the configured chord for `action` is pressed.
 * Global actions (e.g. open search) ignore typing targets unless `ignoreWhenTyping` is false.
 */
export function useShortcut(
  action: ShortcutActionId,
  handler: (event: KeyboardEvent) => void,
  options?: { ignoreWhenTyping?: boolean; enabled?: boolean },
) {
  const chord =
    useAppPreferencesStore(
      (s) => s.preferences.keyboardShortcuts?.[action],
    ) ?? DEFAULT_KEYBOARD_SHORTCUTS[action];
  const ignoreWhenTyping = options?.ignoreWhenTyping ?? false;
  const enabled = options?.enabled ?? true;

  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    if (!enabled) return;
    if (ignoreWhenTyping && isTypingTarget(event.target)) return;
    if (!matchesChord(event, chord)) return;
    event.preventDefault();
    handler(event);
  });

  useEffect(() => {
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onKeyDown, chord, enabled, ignoreWhenTyping]);
}

export function useShortcutChord(action: ShortcutActionId): string {
  return (
    useAppPreferencesStore(
      (s) => s.preferences.keyboardShortcuts?.[action],
    ) ?? DEFAULT_KEYBOARD_SHORTCUTS[action]
  );
}

export { isTypingTarget };
