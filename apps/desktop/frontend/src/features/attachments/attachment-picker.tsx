"use client";

import { useCallback, useRef, useState } from "react";
import { Paperclip, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";
import { ATTACHMENT_ACCEPT, isAllowedAttachmentMime } from "./mime";

export type PendingAttachment = {
  id: string;
  file: File;
  previewUrl?: string;
};

type AttachmentPickerProps = {
  disabled?: boolean;
  loading?: boolean;
  onFiles: (files: File[]) => void | Promise<void>;
  className?: string;
};

function filterFiles(list: FileList | File[]): File[] {
  return Array.from(list).filter((file) =>
    isAllowedAttachmentMime(file.type || guessMime(file.name)),
  );
}

function guessMime(name: string): string {
  const lower = name.toLowerCase();
  if (lower.endsWith(".png")) return "image/png";
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) return "image/jpeg";
  if (lower.endsWith(".webp")) return "image/webp";
  if (lower.endsWith(".gif")) return "image/gif";
  if (lower.endsWith(".pdf")) return "application/pdf";
  if (lower.endsWith(".mp3")) return "audio/mpeg";
  if (lower.endsWith(".wav")) return "audio/wav";
  if (lower.endsWith(".mp4")) return "video/mp4";
  if (lower.endsWith(".webm")) return "video/webm";
  return "";
}

export function AttachmentPicker({
  disabled,
  loading,
  onFiles,
  className,
}: AttachmentPickerProps) {
  const t = useTranslations();
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const handleFiles = useCallback(
    async (files: File[]) => {
      if (!files.length || disabled || loading) return;
      await onFiles(files);
    },
    [disabled, loading, onFiles],
  );

  return (
    <div
      className={cn(
        "rounded-md border border-dashed border-border bg-muted/30 p-4 transition-colors",
        dragOver && "border-primary bg-primary/5",
        className,
      )}
      onDragEnter={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragOver(false);
        void handleFiles(filterFiles(e.dataTransfer.files));
      }}
      onPaste={(e) => {
        const items = Array.from(e.clipboardData.items);
        const files = items
          .filter((item) => item.kind === "file")
          .map((item) => item.getAsFile())
          .filter((f): f is File => !!f);
        if (files.length) {
          e.preventDefault();
          void handleFiles(filterFiles(files));
        }
      }}
      tabIndex={0}
      role="group"
      aria-label={t.attachments.dropZone}
    >
      <div className="flex flex-col items-center gap-2 text-center">
        <Upload className="size-5 text-muted-foreground" aria-hidden />
        <p className="text-sm text-muted-foreground">{t.attachments.dropZone}</p>
        <p className="text-xs text-muted-foreground">{t.attachments.pasteHint}</p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          disabled={disabled || loading}
          onClick={() => inputRef.current?.click()}
        >
          {loading ? <Spinner className="size-4" /> : <Paperclip className="size-4" />}
          {t.attachments.browse}
        </Button>
        <input
          ref={inputRef}
          type="file"
          className="sr-only"
          accept={ATTACHMENT_ACCEPT}
          multiple
          disabled={disabled || loading}
          onChange={(e) => {
            if (e.target.files) {
              void handleFiles(filterFiles(e.target.files));
              e.target.value = "";
            }
          }}
        />
      </div>
    </div>
  );
}
