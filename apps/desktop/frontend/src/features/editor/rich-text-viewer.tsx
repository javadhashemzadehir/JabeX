"use client";

import { useEffect } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import type { RichTextDocument } from "@jabex/types";
import { toTiptapContent } from "@/lib/rich-text";
import { cn } from "@/lib/utils";
import { createCardEditorExtensions } from "./extensions";

type RichTextViewerProps = {
  document: RichTextDocument;
  className?: string;
  contentClassName?: string;
};

/** Read-only TipTap renderer — never injects raw HTML from user input. */
export function RichTextViewer({
  document,
  className,
  contentClassName,
}: RichTextViewerProps) {
  const editor = useEditor({
    immediatelyRender: false,
    editable: false,
    extensions: createCardEditorExtensions(),
    content: toTiptapContent(document),
    editorProps: {
      attributes: {
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

  return (
    <div className={cn("jabex-editor jabex-editor--readonly", className)}>
      <EditorContent editor={editor} />
    </div>
  );
}
