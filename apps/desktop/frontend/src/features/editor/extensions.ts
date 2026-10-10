import Placeholder from "@tiptap/extension-placeholder";
import StarterKit from "@tiptap/starter-kit";
import type { Extensions } from "@tiptap/react";

/** Shared TipTap extension set for card faces (edit + readonly). */
export function createCardEditorExtensions(placeholder?: string): Extensions {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      link: {
        openOnClick: false,
        autolink: true,
        defaultProtocol: "https",
        HTMLAttributes: {
          rel: "noopener noreferrer",
          class: "jabex-editor-link",
        },
      },
      dropcursor: { color: "var(--ring)", width: 2 },
    }),
    Placeholder.configure({
      placeholder: placeholder ?? "",
      emptyEditorClass: "is-editor-empty",
    }),
  ];
}
