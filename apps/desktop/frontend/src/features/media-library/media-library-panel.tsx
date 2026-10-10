"use client";

import { useEffect, useMemo, useState } from "react";
import type { Attachment, AttachmentType, MediaLibraryItem } from "@jabex/types";
import {
  AlertTriangle,
  HardDrive,
  RotateCcw,
  Search,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { ConfirmDeleteDialog } from "@/components/ui/confirm-delete-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { Switch } from "@/components/ui/switch";
import { AttachmentPreview } from "@/features/attachments/attachment-preview";
import { MediaViewer } from "@/features/attachments/media-viewer";
import { useLocale, useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { api } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { formatBytes } from "./format-bytes";
import {
  type MediaTypeFilter,
  useMediaLibraryStore,
} from "./media-library-store";

const TYPE_FILTERS: MediaTypeFilter[] = [
  "all",
  "image",
  "audio",
  "video",
  "pdf",
];

function asAttachment(item: MediaLibraryItem): Attachment {
  const {
    attachmentType: _attachmentType,
    fileMissing: _fileMissing,
    ...attachment
  } = item;
  return attachment;
}

export function MediaLibraryPanel() {
  const t = useTranslations();
  const { locale } = useLocale();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const {
    items,
    stats,
    integrity,
    loading,
    busy,
    error,
    desktop,
    query,
    typeFilter,
    includeDeleted,
    setQuery,
    setTypeFilter,
    setIncludeDeleted,
    hydrate,
    softDelete,
    restore,
    purgeOne,
    purgeAllDeleted,
    removeOrphans,
    verify,
  } = useMediaLibraryStore();

  const [objectUrls, setObjectUrls] = useState<Record<string, string>>({});
  const [viewer, setViewer] = useState<MediaLibraryItem | null>(null);
  const [purgeAllOpen, setPurgeAllOpen] = useState(false);
  const [purgeOneId, setPurgeOneId] = useState<string | null>(null);

  useEffect(() => {
    void hydrate();
  }, [hydrate, activeProfileId, query, typeFilter, includeDeleted]);

  useEffect(() => {
    let cancelled = false;
    const urls: Record<string, string> = {};

    async function loadPreviews() {
      for (const item of items) {
        if (!item.mimeType.startsWith("image/") || item.fileMissing) continue;
        if (item.deletedAt) continue;
        try {
          const bytes = await api.readAttachmentBytes(item.id);
          if (cancelled) return;
          const blob = new Blob([new Uint8Array(bytes)], {
            type: item.mimeType,
          });
          urls[item.id] = URL.createObjectURL(blob);
        } catch {
          // Preview is optional.
        }
      }
      if (!cancelled) {
        setObjectUrls((prev) => {
          for (const url of Object.values(prev)) URL.revokeObjectURL(url);
          return urls;
        });
      }
    }

    void loadPreviews();
    return () => {
      cancelled = true;
      for (const url of Object.values(urls)) URL.revokeObjectURL(url);
    };
  }, [items]);

  const purgeTarget = useMemo(
    () => items.find((item) => item.id === purgeOneId) ?? null,
    [items, purgeOneId],
  );

  if (!desktop) {
    return (
      <section className="flex flex-col gap-6">
        <header className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight">
            {t.mediaLibrary.title}
          </h1>
          <p className="max-w-2xl text-sm text-muted-foreground text-pretty">
            {t.mediaLibrary.description}
          </p>
        </header>
        <p className="text-sm text-muted-foreground">
          {t.mediaLibrary.desktopOnly}
        </p>
      </section>
    );
  }

  async function handleOpen(item: MediaLibraryItem) {
    if (item.fileMissing) {
      toast.error(t.mediaLibrary.fileMissing);
      return;
    }
    if (!objectUrls[item.id]) {
      try {
        const bytes = await api.readAttachmentBytes(item.id);
        const blob = new Blob([new Uint8Array(bytes)], {
          type: item.mimeType,
        });
        const url = URL.createObjectURL(blob);
        setObjectUrls((prev) => ({ ...prev, [item.id]: url }));
      } catch (err) {
        toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
        return;
      }
    }
    setViewer(item);
  }

  async function handleSoftDelete(id: string) {
    try {
      await softDelete(id);
      toast.success(t.mediaLibrary.movedToTrash);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  async function handleRestore(id: string) {
    try {
      await restore(id);
      toast.success(t.mediaLibrary.restored);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  async function handlePurgeOne() {
    if (!purgeOneId) return;
    try {
      const result = await purgeOne(purgeOneId);
      toast.success(
        t.mediaLibrary.purgedOne.replace(
          "{size}",
          formatBytes(result.freedBytes, locale),
        ),
      );
      setPurgeOneId(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  async function handlePurgeAll() {
    try {
      const result = await purgeAllDeleted();
      toast.success(
        t.mediaLibrary.purgedAll
          .replace("{count}", String(result.purgedCount))
          .replace("{size}", formatBytes(result.freedBytes, locale)),
      );
      setPurgeAllOpen(false);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  async function handleVerify() {
    try {
      const report = await verify();
      const issues =
        report.missingFiles.length +
        report.checksumMismatches.length +
        report.orphanDirs.length;
      if (issues === 0) {
        toast.success(t.mediaLibrary.verifyOk);
      } else {
        toast.message(
          t.mediaLibrary.verifyIssues.replace("{count}", String(issues)),
        );
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  async function handleRemoveOrphans() {
    try {
      const removed = await removeOrphans();
      toast.success(
        t.mediaLibrary.orphansRemoved.replace("{count}", String(removed)),
      );
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.mediaLibrary.error);
    }
  }

  return (
    <section className="flex flex-col gap-6" aria-busy={loading || busy}>
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight">
          {t.mediaLibrary.title}
        </h1>
        <p className="max-w-2xl text-sm text-muted-foreground text-pretty">
          {t.mediaLibrary.description}
        </p>
      </header>

      {stats ? (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <StatMetric
            label={t.mediaLibrary.stats.files}
            value={String(stats.totalCount)}
          />
          <StatMetric
            label={t.mediaLibrary.stats.size}
            value={formatBytes(stats.totalBytes, locale)}
          />
          <StatMetric
            label={t.mediaLibrary.stats.trash}
            value={`${stats.deletedCount} · ${formatBytes(stats.deletedBytes, locale)}`}
          />
          <StatMetric
            label={t.mediaLibrary.stats.issues}
            value={String(stats.missingFileCount + stats.orphanDirCount)}
            warn={stats.missingFileCount + stats.orphanDirCount > 0}
          />
        </div>
      ) : null}

      {stats && stats.totalCount > 0 ? (
        <div className="flex flex-wrap gap-2">
          <TypeChip
            label={t.attachments.types.image}
            count={stats.imageCount}
          />
          <TypeChip
            label={t.attachments.types.audio}
            count={stats.audioCount}
          />
          <TypeChip
            label={t.attachments.types.video}
            count={stats.videoCount}
          />
          <TypeChip label={t.attachments.types.pdf} count={stats.pdfCount} />
        </div>
      ) : null}

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
        <div className="relative min-w-0 flex-1">
          <Search
            className="pointer-events-none absolute inset-s-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t.mediaLibrary.searchPlaceholder}
            className="ps-9"
            aria-label={t.mediaLibrary.searchPlaceholder}
          />
        </div>
        <div className="flex flex-wrap gap-1">
          {TYPE_FILTERS.map((type) => (
            <Button
              key={type}
              type="button"
              size="sm"
              variant={typeFilter === type ? "default" : "outline"}
              onClick={() => setTypeFilter(type)}
            >
              {type === "all"
                ? t.mediaLibrary.filterAll
                : t.attachments.types[type as AttachmentType]}
            </Button>
          ))}
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <label className="flex items-center gap-2 text-sm">
          <Switch
            checked={includeDeleted}
            onCheckedChange={setIncludeDeleted}
          />
          <span>{t.mediaLibrary.showTrash}</span>
        </label>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => void handleVerify()}
          >
            <ShieldCheck className="size-4" aria-hidden />
            {t.mediaLibrary.verify}
          </Button>
          {stats && stats.orphanDirCount > 0 ? (
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={busy}
              onClick={() => void handleRemoveOrphans()}
            >
              <HardDrive className="size-4" aria-hidden />
              {t.mediaLibrary.removeOrphans}
            </Button>
          ) : null}
          {stats && stats.deletedCount > 0 ? (
            <Button
              type="button"
              variant="destructive"
              size="sm"
              disabled={busy}
              onClick={() => setPurgeAllOpen(true)}
            >
              <Trash2 className="size-4" aria-hidden />
              {t.mediaLibrary.emptyTrash}
            </Button>
          ) : null}
        </div>
      </div>

      {integrity &&
      integrity.missingFiles.length +
        integrity.checksumMismatches.length +
        integrity.orphanDirs.length >
        0 ? (
        <div
          className="flex gap-2 rounded-md border border-border bg-muted/30 px-3 py-2 text-sm"
          role="status"
        >
          <AlertTriangle
            className="mt-0.5 size-4 shrink-0 text-amber-600 dark:text-amber-400"
            aria-hidden
          />
          <p className="text-muted-foreground text-pretty">
            {t.mediaLibrary.verifyIssues.replace(
              "{count}",
              String(
                integrity.missingFiles.length +
                  integrity.checksumMismatches.length +
                  integrity.orphanDirs.length,
              ),
            )}
          </p>
        </div>
      ) : null}

      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}

      {loading && items.length === 0 ? (
        <div className="flex justify-center py-8">
          <Spinner className="size-5" />
        </div>
      ) : items.length === 0 ? (
        <div className="rounded-md border border-border bg-muted/20 px-3 py-8 text-center">
          <p className="text-sm font-medium">{t.mediaLibrary.emptyTitle}</p>
          <p className="mt-1 text-xs text-muted-foreground text-pretty">
            {t.mediaLibrary.emptyHint}
          </p>
        </div>
      ) : (
        <ul className="flex flex-col gap-2">
          {items.map((item) => {
            const inTrash = Boolean(item.deletedAt);
            return (
              <li
                key={item.id}
                className={cn(
                  "flex flex-col gap-2 rounded-md border border-border p-2 sm:flex-row sm:items-center",
                  inTrash && "border-dashed bg-muted/20",
                )}
              >
                <AttachmentPreview
                  attachment={asAttachment(item)}
                  objectUrl={objectUrls[item.id]}
                  onOpen={() => void handleOpen(item)}
                  className="flex-1 border-0 bg-transparent p-0 hover:bg-transparent"
                />
                <div className="flex flex-wrap items-center gap-1 sm:justify-end">
                  {item.fileMissing ? (
                    <Badge variant="destructive">
                      {t.mediaLibrary.fileMissing}
                    </Badge>
                  ) : null}
                  {inTrash ? (
                    <Badge variant="secondary">{t.mediaLibrary.inTrash}</Badge>
                  ) : null}
                  {inTrash ? (
                    <>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        disabled={busy}
                        onClick={() => void handleRestore(item.id)}
                      >
                        <RotateCcw className="size-4" aria-hidden />
                        {t.mediaLibrary.restore}
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="destructive"
                        disabled={busy}
                        onClick={() => setPurgeOneId(item.id)}
                      >
                        <Trash2 className="size-4" aria-hidden />
                        {t.mediaLibrary.purge}
                      </Button>
                    </>
                  ) : (
                    <Button
                      type="button"
                      size="sm"
                      variant="ghost"
                      disabled={busy}
                      aria-label={t.mediaLibrary.moveToTrash}
                      onClick={() => void handleSoftDelete(item.id)}
                    >
                      <Trash2 className="size-4" aria-hidden />
                    </Button>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      )}

      <MediaViewer
        open={!!viewer}
        onOpenChange={(open) => {
          if (!open) setViewer(null);
        }}
        attachment={viewer ? asAttachment(viewer) : null}
        objectUrl={viewer ? objectUrls[viewer.id] : null}
      />

      <ConfirmDeleteDialog
        open={purgeAllOpen}
        onOpenChange={setPurgeAllOpen}
        title={t.mediaLibrary.purgeAllTitle}
        description={t.mediaLibrary.purgeAllHint}
        cancelLabel={t.decks.cancel}
        deleteLabel={t.mediaLibrary.emptyTrash}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handlePurgeAll()}
      />

      <ConfirmDeleteDialog
        open={!!purgeOneId}
        onOpenChange={(open) => {
          if (!open) setPurgeOneId(null);
        }}
        title={t.mediaLibrary.purgeOneTitle}
        description={t.mediaLibrary.purgeOneHint.replace(
          "{name}",
          purgeTarget?.fileName ?? "",
        )}
        cancelLabel={t.decks.cancel}
        deleteLabel={t.mediaLibrary.purge}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handlePurgeOne()}
      />
    </section>
  );
}

function StatMetric({
  label,
  value,
  warn,
}: {
  label: string;
  value: string;
  warn?: boolean;
}) {
  return (
    <div className="rounded-md border border-border bg-muted/20 px-3 py-2">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p
        className={cn(
          "mt-0.5 text-lg font-semibold tracking-tight",
          warn && "text-amber-700 dark:text-amber-400",
        )}
      >
        {value}
      </p>
    </div>
  );
}

function TypeChip({ label, count }: { label: string; count: number }) {
  return (
    <Badge variant="outline">
      {label}: {count}
    </Badge>
  );
}
