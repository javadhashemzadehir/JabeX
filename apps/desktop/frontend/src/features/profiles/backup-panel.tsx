"use client";

import { useState } from "react";
import { Download, Upload } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useTranslations } from "@/hooks/use-locale";
import { api, isTauri } from "@/lib/tauri";
import { useProfileStore } from "./profile-store";

export function BackupPanel() {
  const t = useTranslations();
  const desktop = useProfileStore((s) => s.desktop);
  const hydrate = useProfileStore((s) => s.hydrate);
  const [importPath, setImportPath] = useState("");
  const [busy, setBusy] = useState(false);

  if (!desktop && !isTauri()) {
    return (
      <p className="text-sm text-muted-foreground">{t.backup.desktopOnly}</p>
    );
  }

  async function handleExport() {
    setBusy(true);
    try {
      const path = await api.exportBackup();
      toast.success(t.backup.exported.replace("{path}", path));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.backup.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleImport() {
    const path = importPath.trim();
    if (!path) return;
    setBusy(true);
    try {
      const result = await api.importBackup(path, "create_profile");
      await hydrate();
      toast.success(
        t.backup.imported
          .replace("{decks}", String(result.decks))
          .replace("{cards}", String(result.cards)),
      );
      setImportPath("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.backup.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap gap-2">
        <Button onClick={() => void handleExport()} disabled={busy}>
          <Download className="size-4" />
          {t.backup.export}
        </Button>
      </div>
      <div className="space-y-2">
        <Label htmlFor="import-path">{t.backup.importPath}</Label>
        <div className="flex flex-col gap-2 sm:flex-row">
          <Input
            id="import-path"
            value={importPath}
            onChange={(event) => setImportPath(event.target.value)}
            placeholder={t.backup.importPlaceholder}
            disabled={busy}
          />
          <Button
            variant="secondary"
            onClick={() => void handleImport()}
            disabled={busy || !importPath.trim()}
          >
            <Upload className="size-4" />
            {t.backup.import}
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">{t.backup.importHint}</p>
      </div>
    </div>
  );
}
