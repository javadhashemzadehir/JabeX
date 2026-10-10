/** Preferred MediaRecorder MIME types (first supported wins). */
export const VOICE_MIME_CANDIDATES = [
  "audio/webm;codecs=opus",
  "audio/webm",
  "audio/ogg;codecs=opus",
  "audio/ogg",
  "audio/mp4",
] as const;

/** Soft cap — well under the 50 MiB audio attachment limit. */
export const VOICE_MAX_DURATION_MS = 10 * 60 * 1000;

export function pickSupportedVoiceMime(
  isTypeSupported: (mime: string) => boolean = (mime) =>
    typeof MediaRecorder !== "undefined" && MediaRecorder.isTypeSupported(mime),
): string {
  for (const mime of VOICE_MIME_CANDIDATES) {
    if (isTypeSupported(mime)) return mime;
  }
  return "";
}

/** Strip codecs / parameters for File.type and attachment whitelist checks. */
export function normalizeVoiceMime(mime: string): string {
  const base = mime.split(";")[0]?.trim().toLowerCase() ?? "";
  if (base === "audio/webm" || base === "audio/ogg" || base === "audio/mp4") {
    return base;
  }
  return "audio/webm";
}

export function extensionForVoiceMime(mime: string): string {
  switch (normalizeVoiceMime(mime)) {
    case "audio/ogg":
      return "ogg";
    case "audio/mp4":
      return "m4a";
    default:
      return "webm";
  }
}

function pad2(n: number): string {
  return String(n).padStart(2, "0");
}

/** Local timestamp filename: voice-YYYYMMDD-HHmmss.ext */
export function buildVoiceFileName(date: Date, mime: string): string {
  const y = date.getFullYear();
  const mo = pad2(date.getMonth() + 1);
  const d = pad2(date.getDate());
  const h = pad2(date.getHours());
  const mi = pad2(date.getMinutes());
  const s = pad2(date.getSeconds());
  return `voice-${y}${mo}${d}-${h}${mi}${s}.${extensionForVoiceMime(mime)}`;
}

export function formatElapsed(ms: number): string {
  const totalSec = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSec / 60);
  const seconds = totalSec % 60;
  return `${pad2(minutes)}:${pad2(seconds)}`;
}
