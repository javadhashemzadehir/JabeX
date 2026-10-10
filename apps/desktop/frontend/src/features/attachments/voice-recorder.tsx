"use client";

import { useEffect, useRef, useState } from "react";
import { Mic, Square, Trash2, Check } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";
import {
  buildVoiceFileName,
  formatElapsed,
  normalizeVoiceMime,
  pickSupportedVoiceMime,
  VOICE_MAX_DURATION_MS,
} from "./voice-recorder-utils";

type VoiceRecorderProps = {
  disabled?: boolean;
  saving?: boolean;
  onSave: (file: File) => void | Promise<void>;
  className?: string;
};

type Phase = "idle" | "recording" | "preview";

export function VoiceRecorder({
  disabled,
  saving,
  onSave,
  className,
}: VoiceRecorderProps) {
  const t = useTranslations();
  const v = t.attachments.voice;

  const [phase, setPhase] = useState<Phase>("idle");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);

  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<BlobPart[]>([]);
  const mimeRef = useRef("audio/webm");
  const startedAtRef = useRef(0);
  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const maxTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const previewUrlRef = useRef<string | null>(null);
  const blobRef = useRef<Blob | null>(null);

  function clearTimers() {
    if (tickRef.current) {
      clearInterval(tickRef.current);
      tickRef.current = null;
    }
    if (maxTimerRef.current) {
      clearTimeout(maxTimerRef.current);
      maxTimerRef.current = null;
    }
  }

  function stopTracks() {
    streamRef.current?.getTracks().forEach((track) => track.stop());
    streamRef.current = null;
  }

  function revokePreview() {
    if (previewUrlRef.current) {
      URL.revokeObjectURL(previewUrlRef.current);
      previewUrlRef.current = null;
    }
    setPreviewUrl(null);
    blobRef.current = null;
  }

  function resetToIdle() {
    clearTimers();
    stopTracks();
    recorderRef.current = null;
    chunksRef.current = [];
    revokePreview();
    setElapsedMs(0);
    setPhase("idle");
  }

  useEffect(() => {
    return () => {
      clearTimers();
      stopTracks();
      if (recorderRef.current?.state === "recording") {
        try {
          recorderRef.current.stop();
        } catch {
          // ignore
        }
      }
      if (previewUrlRef.current) {
        URL.revokeObjectURL(previewUrlRef.current);
        previewUrlRef.current = null;
      }
    };
  }, []);

  function finishRecording(hitMax: boolean) {
    clearTimers();
    const recorder = recorderRef.current;
    if (!recorder || recorder.state === "inactive") {
      stopTracks();
      return;
    }
    recorder.onstop = () => {
      stopTracks();
      const mime = normalizeVoiceMime(mimeRef.current);
      const blob = new Blob(chunksRef.current, { type: mime });
      chunksRef.current = [];
      blobRef.current = blob;
      const url = URL.createObjectURL(blob);
      previewUrlRef.current = url;
      setPreviewUrl(url);
      setPhase("preview");
      if (hitMax) {
        toast.message(v.maxDuration);
      }
    };
    recorder.stop();
  }

  async function handleStart() {
    if (disabled || saving) return;
    if (
      typeof navigator === "undefined" ||
      !navigator.mediaDevices?.getUserMedia ||
      typeof MediaRecorder === "undefined"
    ) {
      toast.error(v.unsupported);
      return;
    }

    const mime = pickSupportedVoiceMime();
    mimeRef.current = mime || "audio/webm";

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;
      chunksRef.current = [];
      revokePreview();

      const options = mime ? { mimeType: mime } : undefined;
      const recorder = options
        ? new MediaRecorder(stream, options)
        : new MediaRecorder(stream);
      mimeRef.current = recorder.mimeType || mimeRef.current;
      recorderRef.current = recorder;

      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) chunksRef.current.push(event.data);
      };

      startedAtRef.current = Date.now();
      setElapsedMs(0);
      setPhase("recording");
      recorder.start(250);

      tickRef.current = setInterval(() => {
        setElapsedMs(Date.now() - startedAtRef.current);
      }, 200);

      maxTimerRef.current = setTimeout(() => {
        finishRecording(true);
      }, VOICE_MAX_DURATION_MS);
    } catch (err) {
      stopTracks();
      const name =
        err && typeof err === "object" && "name" in err
          ? String((err as { name: string }).name)
          : "";
      if (
        name === "NotAllowedError" ||
        name === "PermissionDeniedError" ||
        name === "SecurityError"
      ) {
        toast.error(v.permissionDenied);
      } else {
        toast.error(v.unsupported);
      }
      setPhase("idle");
    }
  }

  function handleStop() {
    if (phase !== "recording") return;
    finishRecording(false);
  }

  function handleDiscard() {
    resetToIdle();
  }

  async function handleSave() {
    const blob = blobRef.current;
    if (!blob || saving || disabled) return;
    const mime = normalizeVoiceMime(blob.type || mimeRef.current);
    const file = new File([blob], buildVoiceFileName(new Date(), mime), {
      type: mime,
    });
    try {
      await onSave(file);
      resetToIdle();
    } catch {
      // Keep preview; parent surfaces the error toast.
    }
  }

  const busy = disabled || saving;

  return (
    <div
      className={cn(
        "rounded-md border border-border bg-muted/20 px-3 py-3",
        className,
      )}
      role="group"
      aria-label={v.title}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Mic className="size-4 shrink-0 text-muted-foreground" aria-hidden />
        <p className="text-sm font-medium">{v.title}</p>
        {(phase === "recording" || phase === "preview") && (
          <span
            className={cn(
              "ms-auto font-mono text-xs tabular-nums text-muted-foreground",
              phase === "recording" && "text-destructive",
            )}
            aria-live="polite"
          >
            {phase === "recording" ? v.recording : v.preview}{" "}
            {formatElapsed(elapsedMs)}
          </span>
        )}
      </div>

      <div className="mt-2 flex flex-wrap items-center gap-2">
        {phase === "idle" && (
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={busy}
            onClick={() => void handleStart()}
          >
            <Mic className="size-4" />
            {v.start}
          </Button>
        )}

        {phase === "recording" && (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            disabled={busy}
            onClick={handleStop}
          >
            <Square className="size-4" />
            {v.stop}
          </Button>
        )}

        {phase === "preview" && (
          <>
            <Button
              type="button"
              variant="default"
              size="sm"
              disabled={busy}
              onClick={() => void handleSave()}
            >
              {saving ? <Spinner className="size-4" /> : <Check className="size-4" />}
              {v.save}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={busy}
              onClick={handleDiscard}
            >
              <Trash2 className="size-4" />
              {v.discard}
            </Button>
          </>
        )}
      </div>

      {phase === "preview" && previewUrl && (
        <audio
          controls
          src={previewUrl}
          className="mt-3 w-full"
          preload="metadata"
        >
          {v.preview}
        </audio>
      )}
    </div>
  );
}
