import type { AttachmentType } from "@jabex/types";

const IMAGE = new Set([
  "image/png",
  "image/jpeg",
  "image/jpg",
  "image/webp",
  "image/gif",
  "image/svg+xml",
]);
const AUDIO = new Set([
  "audio/mpeg",
  "audio/mp3",
  "audio/wav",
  "audio/ogg",
  "audio/webm",
  "audio/mp4",
]);
const VIDEO = new Set([
  "video/mp4",
  "video/webm",
  "video/ogg",
  "video/quicktime",
]);

export function attachmentTypeFromMime(mime: string): AttachmentType | null {
  const value = mime.toLowerCase();
  if (value === "application/pdf") return "pdf";
  if (IMAGE.has(value)) return "image";
  if (AUDIO.has(value)) return "audio";
  if (VIDEO.has(value)) return "video";
  return null;
}

export function isAllowedAttachmentMime(mime: string): boolean {
  return attachmentTypeFromMime(mime) !== null;
}

export function formatBytes(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

export const ATTACHMENT_ACCEPT =
  "image/*,audio/*,video/*,application/pdf,.pdf";
