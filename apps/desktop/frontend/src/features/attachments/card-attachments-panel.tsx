"use client";

import { useCallback, useEffect, useState } from "react";
import type { Attachment } from "@jabex/types";
import { toast } from "sonner";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";
import { AttachmentList } from "./attachment-list";
import { AttachmentPicker } from "./attachment-picker";
import { MediaViewer } from "./media-viewer";
import { isAllowedAttachmentMime } from "./mime";
import { VoiceRecorder } from "./voice-recorder";

type CardAttachmentsPanelProps = {
  cardId: string;
};

export function CardAttachmentsPanel({ cardId }: CardAttachmentsPanelProps) {
  const t = useTranslations();
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [objectUrls, setObjectUrls] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [viewer, setViewer] = useState<Attachment | null>(null);

  const revokeAll = useCallback((urls: Record<string, string>) => {
    for (const url of Object.values(urls)) URL.revokeObjectURL(url);
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const list = await api.listAttachments("card", cardId);
      setAttachments(list);
      const nextUrls: Record<string, string> = {};
      for (const item of list) {
        if (!item.mimeType.startsWith("image/")) continue;
        try {
          const bytes = await api.readAttachmentBytes(item.id);
          const blob = new Blob([new Uint8Array(bytes)], {
            type: item.mimeType,
          });
          nextUrls[item.id] = URL.createObjectURL(blob);
        } catch {
          // Preview is optional.
        }
      }
      setObjectUrls((prev) => {
        revokeAll(prev);
        return nextUrls;
      });
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.attachments.error);
    } finally {
      setLoading(false);
    }
  }, [cardId, revokeAll, t.attachments.error]);

  useEffect(() => {
    void load();
    return () => {
      setObjectUrls((prev) => {
        revokeAll(prev);
        return {};
      });
    };
  }, [load, revokeAll]);

  async function handleFiles(files: File[]): Promise<boolean> {
    setUploading(true);
    try {
      for (const file of files) {
        const mime = file.type || "application/octet-stream";
        if (!isAllowedAttachmentMime(mime)) {
          toast.error(t.attachments.unsupportedType);
          continue;
        }
        const buffer = new Uint8Array(await file.arrayBuffer());
        await api.createAttachment({
          ownerType: "card",
          ownerId: cardId,
          fileName: file.name,
          mimeType: mime,
          bytes: Array.from(buffer),
        });
      }
      toast.success(t.attachments.added);
      await load();
      return true;
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.attachments.error);
      return false;
    } finally {
      setUploading(false);
    }
  }

  async function handleRemove(id: string) {
    setBusyId(id);
    try {
      await api.deleteAttachment(id);
      toast.success(t.attachments.removed);
      await load();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.attachments.error);
    } finally {
      setBusyId(null);
    }
  }

  async function handleOpen(attachment: Attachment) {
    if (!objectUrls[attachment.id]) {
      try {
        const bytes = await api.readAttachmentBytes(attachment.id);
        const blob = new Blob([new Uint8Array(bytes)], {
          type: attachment.mimeType,
        });
        const url = URL.createObjectURL(blob);
        setObjectUrls((prev) => ({ ...prev, [attachment.id]: url }));
      } catch (err) {
        toast.error(err instanceof Error ? err.message : t.attachments.error);
        return;
      }
    }
    setViewer(attachment);
  }

  return (
    <section className="space-y-3" aria-label={t.attachments.title}>
      <div>
        <h3 className="text-sm font-medium">{t.attachments.title}</h3>
        <p className="text-xs text-muted-foreground">{t.attachments.hint}</p>
      </div>
      <AttachmentPicker
        loading={uploading}
        onFiles={async (files) => {
          await handleFiles(files);
        }}
      />
      <VoiceRecorder
        disabled={loading}
        saving={uploading}
        onSave={async (file) => {
          const ok = await handleFiles([file]);
          if (!ok) throw new Error("voice-save-failed");
        }}
      />
      {loading ? (
        <div className="flex justify-center py-4">
          <Spinner className="size-5" />
        </div>
      ) : (
        <AttachmentList
          attachments={attachments}
          objectUrls={objectUrls}
          onRemove={handleRemove}
          onOpen={(item) => void handleOpen(item)}
          busyId={busyId}
        />
      )}
      <MediaViewer
        open={!!viewer}
        onOpenChange={(open) => {
          if (!open) setViewer(null);
        }}
        attachment={viewer}
        objectUrl={viewer ? objectUrls[viewer.id] : null}
      />
    </section>
  );
}
