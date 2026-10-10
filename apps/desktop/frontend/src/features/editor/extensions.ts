import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import Placeholder from "@tiptap/extension-placeholder";
import { Table } from "@tiptap/extension-table";
import TableCell from "@tiptap/extension-table-cell";
import TableHeader from "@tiptap/extension-table-header";
import TableRow from "@tiptap/extension-table-row";
import Underline from "@tiptap/extension-underline";
import StarterKit from "@tiptap/starter-kit";
import type { Extensions } from "@tiptap/react";
import { common, createLowlight } from "lowlight";
import { MathDisplay, MathInline } from "./extensions/math";
import { Spoiler, SpoilerBlock } from "./extensions/spoiler";

const lowlight = createLowlight(common);

export type CardEditorExtensionsOptions = {
  placeholder?: string;
  /** When false, skip Placeholder (readonly viewer). Default true. */
  editable?: boolean;
};

/** Shared TipTap extension set for card faces (edit + readonly). */
export function createCardEditorExtensions(
  options: CardEditorExtensionsOptions | string = {},
): Extensions {
  const opts =
    typeof options === "string"
      ? { placeholder: options, editable: true }
      : { editable: true, ...options };

  const extensions: Extensions = [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      codeBlock: false,
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
    Underline,
    Spoiler,
    SpoilerBlock,
    MathInline,
    MathDisplay,
    Table.configure({
      resizable: true,
      allowTableNodeSelection: true,
      HTMLAttributes: {
        class: "jabex-table",
        dir: "ltr",
      },
    }),
    TableRow,
    TableHeader,
    TableCell,
    CodeBlockLowlight.configure({
      lowlight,
      defaultLanguage: "plaintext",
      HTMLAttributes: {
        class: "jabex-code-block",
        dir: "ltr",
        spellcheck: "false",
      },
    }),
  ];

  if (opts.editable !== false) {
    extensions.push(
      Placeholder.configure({
        placeholder: opts.placeholder ?? "",
        emptyEditorClass: "is-editor-empty",
      }),
    );
  }

  return extensions;
}
