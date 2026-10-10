"use client";

import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Download, Upload } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";
import { useProfileStore } from "./profile-store";

const JBX_FILTER = [{ name: "JabeX backup", extensions: ["jbx"] }];

export function BackupPanel() {
  const t = useTranslations();
  const desktop = useProfileStore((s) => s.desktop);
  const hydrate = useProfileStore((s) => s.hydrate);
  const active = useProfileStore((s) => s.active);
  const tauri = useIsTauri();
  const [busy, setBusy] = useState(false);

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.backup.desktopOnly}</p>
    );
  }

  async function handleExport() {
    setBusy(true);
    try {
      const stamp = new Date().toISOString().replace(/[:.]/g, "").slice(0, 15);
      const profileSlug = (active?.displayName ?? "profile")
        .trim()
        .replace(/[^\w\-]+/g, "-")
        .replace(/^-+|-+$/g, "")
        .slice(0, 40);
      const suggested = `jabex-${profileSlug || "profile"}-${stamp}.jbx`;

      const outputPath = await save({
        title: t.backup.exportDialogTitle,
        defaultPath: suggested,
        filters: JBX_FILTER,
      });
      if (!outputPath) return;

      const path = await api.exportBackup(outputPath);
      toast.success(t.backup.exported.replace("{path}", path));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.backup.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleImport() {
    setBusy(true);
    try {
      const selected = await open({
        title: t.backup.importDialogTitle,
        multiple: false,
        directory: false,
        filters: JBX_FILTER,
      });
      if (!selected || Array.isArray(selected)) return;

      const result = await api.importBackup(selected, "create_profile");
      await hydrate();
      toast.success(
        t.backup.imported
          .replace("{decks}", String(result.decks))
          .replace("{cards}", String(result.cards)),
      );
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.backup.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap gap-2">
        <Button onClick={() => void handleExport()} disabled={busy}>
          {busy ? <Spinner /> : <Download className="size-4" />}
          {t.backup.export}
        </Button>
        <Button
          variant="secondary"
          onClick={() => void handleImport()}
          disabled={busy}
        >
          {busy ? <Spinner /> : <Upload className="size-4" />}
          {t.backup.import}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">{t.backup.importHint}</p>
    </div>
  );
}
