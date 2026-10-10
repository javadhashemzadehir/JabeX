"use client";

import { motion } from "framer-motion";
import { BackupPanel } from "@/features/profiles/backup-panel";
import { useTranslations } from "@/hooks/use-locale";
import { ApplicationSettingsPanel } from "./application-settings-panel";

export function SettingsPanel() {
  const t = useTranslations();

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-10"
    >
      <div className="space-y-1">
        <h2 className="text-2xl font-semibold tracking-tight">
          {t.settings.title}
        </h2>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.settings.description}
        </p>
      </div>

      <ApplicationSettingsPanel />

      <section id="backup" className="space-y-4 scroll-mt-6">
        <div className="space-y-1">
          <h3 className="text-lg font-semibold tracking-tight">
            {t.backup.title}
          </h3>
          <p className="text-sm text-muted-foreground text-pretty">
            {t.app.privacyNote}
          </p>
        </div>
        <BackupPanel />
      </section>
    </motion.div>
  );
}
