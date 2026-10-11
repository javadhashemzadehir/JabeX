/**
 * Shared HTMLAudioElement helpers for cached pronunciation WAV/MP3.
 */

export type PlaybackSpeed = 0.5 | 0.75 | 1 | 1.25;

export async function playAudioBytes(
  bytes: ArrayBuffer | Uint8Array | number[],
  options?: {
    mimeType?: string;
    playbackRate?: number;
    signal?: AbortSignal;
  },
): Promise<void> {
  const mime = options?.mimeType ?? sniffMime(bytes);
  const copy = Uint8Array.from(
    bytes instanceof ArrayBuffer
      ? new Uint8Array(bytes)
      : bytes instanceof Uint8Array
        ? bytes
        : bytes,
  );
  const blob = new Blob([copy], { type: mime });
  const url = URL.createObjectURL(blob);
  try {
    const audio = new Audio(url);
    audio.playbackRate = options?.playbackRate ?? 1;
    await new Promise<void>((resolve, reject) => {
      const onAbort = () => {
        audio.pause();
        reject(new DOMException("Aborted", "AbortError"));
      };
      options?.signal?.addEventListener("abort", onAbort, { once: true });
      audio.onended = () => resolve();
      audio.onerror = () => reject(new Error("Audio playback failed"));
      void audio.play().catch(reject);
    });
  } finally {
    URL.revokeObjectURL(url);
  }
}

function sniffMime(bytes: ArrayBuffer | Uint8Array | number[]): string {
  const arr =
    bytes instanceof ArrayBuffer
      ? new Uint8Array(bytes)
      : bytes instanceof Uint8Array
        ? bytes
        : new Uint8Array(bytes);
  if (arr.length >= 4 && arr[0] === 0x52 && arr[1] === 0x49 && arr[2] === 0x46 && arr[3] === 0x46) {
    return "audio/wav";
  }
  return "audio/mpeg";
}

export async function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  if (ms <= 0) return;
  await new Promise<void>((resolve, reject) => {
    const timer = window.setTimeout(() => resolve(), ms);
    signal?.addEventListener(
      "abort",
      () => {
        window.clearTimeout(timer);
        reject(new DOMException("Aborted", "AbortError"));
      },
      { once: true },
    );
  });
}
