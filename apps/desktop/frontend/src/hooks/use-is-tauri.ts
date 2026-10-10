"use client";

import { useSyncExternalStore } from "react";
import { isTauri } from "@/lib/tauri";

const emptySubscribe = () => () => undefined;

/**
 * SSR-safe Tauri detection: server + hydration snapshot is always false,
 * then the client re-renders with the real value (no hydration mismatch).
 */
export function useIsTauri(): boolean {
  return useSyncExternalStore(emptySubscribe, isTauri, () => false);
}
