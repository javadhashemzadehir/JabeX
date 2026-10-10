import { describe, expect, it } from "vitest";

import { chordFromEvent, formatChordLabel, matchesChord } from "./chord";

function keyEvent(
  key: string,
  mods: Partial<{
    ctrlKey: boolean;
    metaKey: boolean;
    altKey: boolean;
    shiftKey: boolean;
  }> = {},
): KeyboardEvent {
  return {
    key,
    ctrlKey: mods.ctrlKey ?? false,
    metaKey: mods.metaKey ?? false,
    altKey: mods.altKey ?? false,
    shiftKey: mods.shiftKey ?? false,
  } as KeyboardEvent;
}

describe("matchesChord", () => {
  it("matches mod+k with ctrl on non-apple", () => {
    expect(matchesChord(keyEvent("k", { ctrlKey: true }), "mod+k")).toBe(true);
    expect(matchesChord(keyEvent("k", { metaKey: true }), "mod+k")).toBe(true);
    expect(matchesChord(keyEvent("k"), "mod+k")).toBe(false);
  });

  it("matches space and enter", () => {
    expect(matchesChord(keyEvent(" "), "space")).toBe(true);
    expect(matchesChord(keyEvent("Enter"), "enter")).toBe(true);
    expect(matchesChord(keyEvent("1"), "1")).toBe(true);
  });

  it("rejects wrong modifiers", () => {
    expect(matchesChord(keyEvent("k", { ctrlKey: true, altKey: true }), "mod+k")).toBe(
      false,
    );
  });
});

describe("chordFromEvent", () => {
  it("builds space and digit chords", () => {
    expect(chordFromEvent(keyEvent(" "))).toBe("space");
    expect(chordFromEvent(keyEvent("3"))).toBe("3");
  });
});

describe("formatChordLabel", () => {
  it("formats common chords", () => {
    expect(formatChordLabel("space")).toBe("Space");
    expect(formatChordLabel("enter")).toBe("Enter");
    expect(formatChordLabel("1")).toBe("1");
  });
});
