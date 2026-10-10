"use client";

import type { Attachment } from "@jabex/types";
import { FileText, ImageIcon, Music, Video } from "lucide-react";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";
import { attachmentTypeFromMime, formatBytes } from "./mime";

type AttachmentPreviewProps = {
  attachment: Attachment;
  objectUrl?: string | null;
  className?: string;
  onOpen?: () => void;
};

export function AttachmentPreview({
  attachment,
  objectUrl,
  className,
  onOpen,
}: AttachmentPreviewProps) {
  const t = useTranslations();
  const kind = attachmentTypeFromMime(attachment.mimeType);

  return (
    <button
      type="button"
      onClick={onOpen}
      className={cn(
        "flex w-full items-center gap-3 rounded-md border border-border bg-background p-2 text-start transition-colors hover:bg-muted/50",
        className,
      )}
    >
      <div className="flex size-12 shrink-0 items-center justify-center overflow-hidden rounded bg-muted">
        {kind === "image" && objectUrl ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            src={objectUrl}
            alt=""
            className="size-full object-cover"
          />
        ) : (
          <KindIcon kind={kind} />
        )}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium">{attachment.fileName}</p>
        <p className="text-xs text-muted-foreground">
          {kind ? t.attachments.types[kind] : attachment.mimeType} ·{" "}
          {formatBytes(attachment.sizeBytes)}
        </p>
      </div>
    </button>
  );
}

function KindIcon({ kind }: { kind: ReturnType<typeof attachmentTypeFromMime> }) {
  const className = "size-5 text-muted-foreground";
  switch (kind) {
    case "image":
      return <ImageIcon className={className} aria-hidden />;
    case "audio":
      return <Music className={className} aria-hidden />;
    case "video":
      return <Video className={className} aria-hidden />;
    case "pdf":
      return <FileText className={className} aria-hidden />;
    default:
      return <FileText className={className} aria-hidden />;
  }
}
