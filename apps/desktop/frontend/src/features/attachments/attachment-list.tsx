"use client";

import type { Attachment } from "@jabex/types";
import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTranslations } from "@/hooks/use-locale";
import { AttachmentPreview } from "./attachment-preview";

type AttachmentListProps = {
  attachments: Attachment[];
  objectUrls?: Record<string, string>;
  empty?: boolean;
  onRemove?: (id: string) => void;
  onOpen?: (attachment: Attachment) => void;
  busyId?: string | null;
};

export function AttachmentList({
  attachments,
  objectUrls = {},
  empty,
  onRemove,
  onOpen,
  busyId,
}: AttachmentListProps) {
  const t = useTranslations();

  if (empty || attachments.length === 0) {
    return (
      <div className="rounded-md border border-border bg-muted/20 px-3 py-6 text-center">
        <p className="text-sm font-medium">{t.attachments.emptyTitle}</p>
        <p className="mt-1 text-xs text-muted-foreground">
          {t.attachments.emptyHint}
        </p>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-2">
      {attachments.map((attachment) => (
        <li key={attachment.id} className="flex items-center gap-2">
          <AttachmentPreview
            attachment={attachment}
            objectUrl={objectUrls[attachment.id]}
            onOpen={() => onOpen?.(attachment)}
            className="flex-1"
          />
          {onRemove ? (
            <Button
              type="button"
              variant="ghost"
              size="icon"
              disabled={busyId === attachment.id}
              aria-label={t.attachments.remove}
              onClick={() => onRemove(attachment.id)}
            >
              <Trash2 className="size-4" />
            </Button>
          ) : null}
        </li>
      ))}
    </ul>
  );
}
