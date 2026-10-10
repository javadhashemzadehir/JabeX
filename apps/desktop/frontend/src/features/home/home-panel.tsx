"use client";

import { motion } from "framer-motion";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { BackupPanel } from "@/features/profiles/backup-panel";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useTranslations } from "@/hooks/use-locale";

export function HomePanel() {
  const t = useTranslations();
  const active = useProfileStore((s) => s.active);
  const profiles = useProfileStore((s) => s.profiles);
  const desktop = useProfileStore((s) => s.desktop);

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-4"
    >
      <Card className="overflow-hidden border-border/60 bg-gradient-to-br from-background via-muted/40 to-background shadow-none">
        <CardHeader>
          <CardDescription>{t.app.tagline}</CardDescription>
          <CardTitle className="text-3xl tracking-tight text-balance">
            {t.home.welcome}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm leading-7 text-muted-foreground text-pretty">
            {t.home.description}
          </p>
          {desktop && active ? (
            <p className="text-sm font-medium text-foreground/90">
              {t.home.profiles}: {active.displayName}
              {profiles.length > 1 ? ` (${profiles.length})` : ""}
            </p>
          ) : (
            <p className="text-sm font-medium text-foreground/90">{t.home.cta}</p>
          )}
        </CardContent>
      </Card>

      <Card className="border-border/60 shadow-none">
        <CardHeader>
          <CardTitle className="text-lg">{t.backup.title}</CardTitle>
          <CardDescription>{t.app.privacyNote}</CardDescription>
        </CardHeader>
        <CardContent>
          <BackupPanel />
        </CardContent>
      </Card>
    </motion.section>
  );
}
