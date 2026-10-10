"use client";

import type { Attachment } from "@jabex/types";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useTranslations } from "@/hooks/use-locale";
import { attachmentTypeFromMime } from "./mime";

type MediaViewerProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  attachment: Attachment | null;
  objectUrl?: string | null;
};

export function MediaViewer({
  open,
  onOpenChange,
  attachment,
  objectUrl,
}: MediaViewerProps) {
  const t = useTranslations();
  const kind = attachment
    ? attachmentTypeFromMime(attachment.mimeType)
    : null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>
            {attachment?.fileName ?? t.attachments.viewerTitle}
          </DialogTitle>
        </DialogHeader>
        <div className="flex min-h-48 items-center justify-center overflow-auto rounded-md bg-muted/40 p-2">
          {!attachment || !objectUrl ? (
            <p className="text-sm text-muted-foreground">
              {t.attachments.previewUnavailable}
            </p>
          ) : kind === "image" ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={objectUrl}
              alt={attachment.fileName}
              className="max-h-[70vh] max-w-full object-contain"
            />
          ) : kind === "audio" ? (
            <audio controls src={objectUrl} className="w-full" />
          ) : kind === "video" ? (
            <video
              controls
              src={objectUrl}
              className="max-h-[70vh] max-w-full"
            />
          ) : kind === "pdf" ? (
            <iframe
              title={attachment.fileName}
              src={objectUrl}
              className="h-[70vh] w-full rounded border border-border"
            />
          ) : (
            <p className="text-sm text-muted-foreground">
              {t.attachments.previewUnavailable}
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
