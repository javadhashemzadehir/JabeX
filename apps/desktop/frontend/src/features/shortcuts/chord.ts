/** Normalize a KeyboardEvent key into a chord token. */
export function normalizeEventKey(event: KeyboardEvent): string | null {
  const key = event.key;
  if (key === " ") return "space";
  if (key === "Escape") return "escape";
  if (key.length === 1) return key.toLowerCase();
  const lower = key.toLowerCase();
  if (
    lower === "control" ||
    lower === "meta" ||
    lower === "alt" ||
    lower === "shift" ||
    lower === "os"
  ) {
    return null;
  }
  return lower;
}

function isApplePlatform(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Mac|iPhone|iPad|iPod/i.test(
    navigator.platform || navigator.userAgent,
  );
}

/** Build a chord string from a keydown event (e.g. `mod+k`, `space`). */
export function chordFromEvent(event: KeyboardEvent): string | null {
  const key = normalizeEventKey(event);
  if (!key) return null;

  const parts: string[] = [];
  const apple = isApplePlatform();
  const primaryMod = apple ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;

  if (primaryMod) {
    parts.push("mod");
  } else {
    if (event.ctrlKey) parts.push("ctrl");
    if (event.metaKey) parts.push("meta");
  }
  if (event.altKey) parts.push("alt");
  if (event.shiftKey && key !== "shift") parts.push("shift");
  parts.push(key);
  return parts.join("+");
}

function parseChord(chord: string): {
  key: string;
  mod: boolean;
  ctrl: boolean;
  meta: boolean;
  alt: boolean;
  shift: boolean;
} | null {
  const trimmed = chord.trim().toLowerCase();
  if (!trimmed) return null;
  const parts = trimmed.split("+").filter(Boolean);
  if (parts.length === 0) return null;
  const key = parts[parts.length - 1]!;
  return {
    key,
    mod: parts.includes("mod"),
    ctrl: parts.includes("ctrl"),
    meta: parts.includes("meta"),
    alt: parts.includes("alt"),
    shift: parts.includes("shift"),
  };
}

/** True when the event matches the stored chord. */
export function matchesChord(event: KeyboardEvent, chord: string): boolean {
  const parsed = parseChord(chord);
  if (!parsed) return false;

  const eventKey = normalizeEventKey(event);
  if (!eventKey || eventKey !== parsed.key) return false;

  const wantCtrl = parsed.mod || parsed.ctrl;
  const wantMeta = parsed.mod || parsed.meta;

  if (parsed.mod) {
    const hasMod = event.ctrlKey || event.metaKey;
    if (!hasMod) return false;
    // When chord is mod-only, allow either Ctrl or Meta (not Alt-required).
  } else {
    if (event.ctrlKey !== wantCtrl) return false;
    if (event.metaKey !== wantMeta) return false;
  }

  if (event.altKey !== parsed.alt) return false;
  if (event.shiftKey !== parsed.shift) return false;
  return true;
}

/** Human-readable label for UI (⌘K / Ctrl+K / Space). */
export function formatChordLabel(chord: string): string {
  const parsed = parseChord(chord);
  if (!parsed) return chord;
  const apple = isApplePlatform();
  const tokens: string[] = [];
  if (parsed.mod) {
    tokens.push(apple ? "⌘" : "Ctrl");
  } else {
    if (parsed.ctrl) tokens.push("Ctrl");
    if (parsed.meta) tokens.push(apple ? "⌘" : "Meta");
  }
  if (parsed.alt) tokens.push(apple ? "⌥" : "Alt");
  if (parsed.shift) tokens.push(apple ? "⇧" : "Shift");

  const keyLabel =
    parsed.key === "space"
      ? "Space"
      : parsed.key === "enter"
        ? "Enter"
        : parsed.key === "escape"
          ? "Esc"
          : parsed.key.length === 1
            ? parsed.key.toUpperCase()
            : parsed.key.charAt(0).toUpperCase() + parsed.key.slice(1);

  if (apple && (parsed.mod || parsed.meta || parsed.alt || parsed.shift)) {
    return `${tokens.join("")}${keyLabel}`;
  }
  tokens.push(keyLabel);
  return tokens.join("+");
}

export function chordsConflict(a: string, b: string): boolean {
  return a.trim().toLowerCase() === b.trim().toLowerCase();
}
