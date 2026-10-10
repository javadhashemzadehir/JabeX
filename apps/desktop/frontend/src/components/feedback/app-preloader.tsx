"use client";

import { motion } from "framer-motion";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";

/** Full-viewport splash while the desktop shell bootstraps. */
export function AppPreloader() {
  const t = useTranslations();

  return (
    <div
      className="flex h-svh w-full flex-col items-center justify-center gap-5 bg-background text-foreground"
      role="status"
      aria-live="polite"
      aria-busy="true"
      aria-label={t.app.loading}
    >
      <motion.img
        src="/brand/mark.png"
        alt=""
        width={56}
        height={56}
        initial={{ opacity: 0, scale: 0.92 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.35 }}
        className="size-14"
      />
      <div className="flex flex-col items-center gap-3">
        <p className="text-lg font-semibold tracking-tight">{t.app.name}</p>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Spinner className="size-4" />
          <span>{t.app.loading}</span>
        </div>
      </div>
    </div>
  );
}
