import { describe, expect, it } from "vitest";
import {
  buildVoiceFileName,
  extensionForVoiceMime,
  formatElapsed,
  normalizeVoiceMime,
  pickSupportedVoiceMime,
  VOICE_MIME_CANDIDATES,
} from "./voice-recorder-utils";

describe("voice-recorder-utils", () => {
  it("picks the first supported mime from the preference list", () => {
    const supported = new Set(["audio/ogg", "audio/mp4"]);
    expect(pickSupportedVoiceMime((m) => supported.has(m))).toBe("audio/ogg");
  });

  it("returns empty string when nothing is supported", () => {
    expect(pickSupportedVoiceMime(() => false)).toBe("");
  });

  it("exposes a non-empty candidate list", () => {
    expect(VOICE_MIME_CANDIDATES.length).toBeGreaterThan(0);
    expect(VOICE_MIME_CANDIDATES[0]).toContain("webm");
  });

  it("normalizes mime by stripping codecs", () => {
    expect(normalizeVoiceMime("audio/webm;codecs=opus")).toBe("audio/webm");
    expect(normalizeVoiceMime("audio/ogg;codecs=opus")).toBe("audio/ogg");
    expect(normalizeVoiceMime("audio/mp4")).toBe("audio/mp4");
    expect(normalizeVoiceMime("")).toBe("audio/webm");
  });

  it("maps mime to file extension", () => {
    expect(extensionForVoiceMime("audio/webm;codecs=opus")).toBe("webm");
    expect(extensionForVoiceMime("audio/ogg")).toBe("ogg");
    expect(extensionForVoiceMime("audio/mp4")).toBe("m4a");
  });

  it("builds a timestamped voice filename", () => {
    const date = new Date(2026, 9, 10, 18, 55, 3);
    expect(buildVoiceFileName(date, "audio/webm;codecs=opus")).toBe(
      "voice-20261010-185503.webm",
    );
    expect(buildVoiceFileName(date, "audio/ogg")).toBe(
      "voice-20261010-185503.ogg",
    );
  });

  it("formats elapsed mm:ss", () => {
    expect(formatElapsed(0)).toBe("00:00");
    expect(formatElapsed(1_500)).toBe("00:01");
    expect(formatElapsed(65_000)).toBe("01:05");
    expect(formatElapsed(10 * 60 * 1000)).toBe("10:00");
  });
});
