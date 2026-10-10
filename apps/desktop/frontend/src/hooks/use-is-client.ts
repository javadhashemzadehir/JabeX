"use client";

import { useSyncExternalStore } from "react";

const emptySubscribe = () => () => undefined;

/** True only after client hydration (SSR-safe). */
export function useIsClient() {
  return useSyncExternalStore(
    emptySubscribe,
    () => true,
    () => false,
  );
}
