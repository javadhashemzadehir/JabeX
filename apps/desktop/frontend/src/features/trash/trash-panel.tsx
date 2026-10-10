"use client";

import { useEffect, useState } from "react";
import type { TrashEntityKind, TrashItem } from "@jabex/types";
import {
  FileImage,
  Layers,
  PenLine,
  RectangleHorizontal,
  RotateCcw,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { ConfirmDeleteDialog } from "@/components/ui/confirm-delete-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useTrashStore } from "./trash-store";

function kindIcon(kind: TrashEntityKind) {
  switch (kind) {
    case "deck":
      return Layers;
    case "card":
      return RectangleHorizontal;
    case "attachment":
      return FileImage;
    case "ink_document":
      return PenLine;
  }
}

export function TrashPanel() {
  const t = useTranslations();
  const { locale } = useLocale();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const { items, loading, busy, error, desktop, hydrate, restore, permanentlyDelete } =
    useTrashStore();
  const [pending, setPending] = useState<TrashItem | null>(null);

  useEffect(() => {
    void hydrate();
  }, [hydrate, activeProfileId]);

  async function handleRestore(item: TrashItem) {
    try {
      await restore(item.entityKind, item.id);
      toast.success(t.trash.restored);
    } catch {
      toast.error(t.trash.error);
    }
  }

  async function handlePermanentDelete() {
    if (!pending) return;
    try {
      await permanentlyDelete(pending.entityKind, pending.id);
      toast.success(t.trash.deletedForever);
      setPending(null);
    } catch {
      toast.error(t.trash.error);
    }
  }

  function formatDeletedAt(iso: string) {
    try {
      return new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  }

  function kindLabel(kind: TrashEntityKind) {
    return t.trash.kinds[kind];
  }

  return (
    <section className="flex flex-col gap-6">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight">{t.trash.title}</h1>
        <p className="max-w-2xl text-sm text-muted-foreground text-pretty">
          {t.trash.description}
        </p>
      </header>

      {!desktop ? (
        <p className="text-sm text-muted-foreground">{t.trash.desktopOnly}</p>
      ) : null}

      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}

      {loading ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Spinner className="size-4" />
          <span>{t.trash.loading}</span>
        </div>
      ) : null}

      {!loading && desktop && items.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border px-6 py-12 text-center">
          <Trash2 className="mx-auto mb-3 size-8 text-muted-foreground" aria-hidden />
          <p className="text-sm font-medium">{t.trash.emptyTitle}</p>
          <p className="mt-1 text-sm text-muted-foreground">{t.trash.emptyHint}</p>
        </div>
      ) : null}

      {items.length > 0 ? (
        <ul className="divide-y divide-border rounded-lg border border-border">
          {items.map((item) => {
            const Icon = kindIcon(item.entityKind);
            return (
              <li
                key={`${item.entityKind}:${item.id}`}
                className="flex flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
              >
                <div className="flex min-w-0 items-start gap-3">
                  <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden />
                  <div className="min-w-0 space-y-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="truncate text-sm font-medium">{item.title}</p>
                      <Badge variant="secondary">{kindLabel(item.entityKind)}</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {t.trash.deletedAt}: {formatDeletedAt(item.deletedAt)}
                    </p>
                  </div>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={busy}
                    onClick={() => void handleRestore(item)}
                  >
                    <RotateCcw className="size-4" aria-hidden />
                    {t.trash.restore}
                  </Button>
                  <Button
                    type="button"
                    variant="destructive"
                    size="sm"
                    disabled={busy}
                    onClick={() => setPending(item)}
                  >
                    <Trash2 className="size-4" aria-hidden />
                    {t.trash.deleteForever}
                  </Button>
                </div>
              </li>
            );
          })}
        </ul>
      ) : null}

      <ConfirmDeleteDialog
        open={Boolean(pending)}
        onOpenChange={(open) => {
          if (!open && !busy) setPending(null);
        }}
        title={t.trash.confirmTitle}
        description={t.trash.confirmDescription}
        cancelLabel={t.trash.cancel}
        deleteLabel={t.trash.deleteForever}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handlePermanentDelete()}
      />
    </section>
  );
}
