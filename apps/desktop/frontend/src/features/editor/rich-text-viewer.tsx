"use client";

import { useEffect, useRef } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import type { RichTextDocument } from "@jabex/types";
import { toTiptapContent } from "@/lib/rich-text";
import { cn } from "@/lib/utils";
import { createCardEditorExtensions } from "./extensions";
import { bindSpoilerInteractions } from "./spoiler-interaction";

type RichTextViewerProps = {
  document: RichTextDocument;
  className?: string;
  contentClassName?: string;
};

/** Read-only TipTap renderer — never injects raw HTML from user input as storage. */
export function RichTextViewer({
  document,
  className,
  contentClassName,
}: RichTextViewerProps) {
  const rootRef = useRef<HTMLDivElement>(null);

  const editor = useEditor({
    immediatelyRender: false,
    editable: false,
    extensions: createCardEditorExtensions({ editable: false }),
    content: toTiptapContent(document),
    editorProps: {
      attributes: {
        // Auto direction so LTR English (and trailing ?) stays correct inside RTL chrome.
        dir: "auto",
        class: cn("jabex-editor-content", contentClassName),
      },
    },
  });

  useEffect(() => {
    if (!editor) return;
    editor.commands.setContent(toTiptapContent(document), {
      emitUpdate: false,
    });
  }, [document, editor]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    return bindSpoilerInteractions(root);
  }, []);

  return (
    <div
      ref={rootRef}
      className={cn("jabex-editor jabex-editor--readonly", className)}
    >
      <EditorContent editor={editor} />
    </div>
  );
}
