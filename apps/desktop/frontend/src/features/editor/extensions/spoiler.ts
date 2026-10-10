import { Mark, Node, mergeAttributes } from "@tiptap/core";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    spoiler: {
      toggleSpoiler: () => ReturnType;
      setSpoiler: () => ReturnType;
      unsetSpoiler: () => ReturnType;
    };
    spoilerBlock: {
      toggleSpoilerBlock: () => ReturnType;
      setSpoilerBlock: () => ReturnType;
    };
  }
}

/** Telegram-style inline spoiler mark. */
export const Spoiler = Mark.create({
  name: "spoiler",
  inclusive: false,
  excludes: "",
  spanning: true,

  parseHTML() {
    return [{ tag: "span[data-spoiler]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-spoiler": "",
        class: "jabex-spoiler",
        role: "button",
        tabindex: "0",
        "aria-expanded": "false",
      }),
      0,
    ];
  },

  addCommands() {
    return {
      toggleSpoiler:
        () =>
        ({ commands }) =>
          commands.toggleMark(this.name),
      setSpoiler:
        () =>
        ({ commands }) =>
          commands.setMark(this.name),
      unsetSpoiler:
        () =>
        ({ commands }) =>
          commands.unsetMark(this.name),
    };
  },

  addKeyboardShortcuts() {
    return {
      "Mod-Shift-s": () => this.editor.commands.toggleSpoiler(),
    };
  },
});

/** Multi-line spoiler container (paragraphs, lists, code, tables, math). */
export const SpoilerBlock = Node.create({
  name: "spoilerBlock",
  group: "block",
  content: "block+",
  defining: true,
  isolating: true,

  parseHTML() {
    return [{ tag: "div[data-spoiler-block]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "div",
      mergeAttributes(HTMLAttributes, {
        "data-spoiler-block": "",
        class: "jabex-spoiler-block",
        role: "button",
        tabindex: "0",
        "aria-expanded": "false",
      }),
      0,
    ];
  },

  addCommands() {
    return {
      setSpoilerBlock:
        () =>
        ({ commands }) =>
          commands.wrapIn(this.name),
      toggleSpoilerBlock:
        () =>
        ({ commands }) =>
          commands.toggleWrap(this.name),
    };
  },
});
