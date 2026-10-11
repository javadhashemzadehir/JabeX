import type { KeyboardShortcuts } from "@jabex/types";

export const DEFAULT_KEYBOARD_SHORTCUTS: KeyboardShortcuts = {
  openSearch: "mod+k",
  reviewFlipOrGood: "space",
  reviewFlipOrGoodAlt: "enter",
  reviewGradeAgain: "1",
  reviewGradeHard: "2",
  reviewGradeGood: "3",
  reviewGradeEasy: "4",
  reviewPlayAudio: "p",
};

export type ShortcutActionId = keyof KeyboardShortcuts;

export const SHORTCUT_ACTION_ORDER: ShortcutActionId[] = [
  "openSearch",
  "reviewFlipOrGood",
  "reviewFlipOrGoodAlt",
  "reviewGradeAgain",
  "reviewGradeHard",
  "reviewGradeGood",
  "reviewGradeEasy",
  "reviewPlayAudio",
];
