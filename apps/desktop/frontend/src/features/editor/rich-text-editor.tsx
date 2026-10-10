"use client";

import { useEffect, type ReactNode } from "react";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import {
  Bold,
  Code,
  Code2,
  Heading1,
  Heading2,
  Heading3,
  Italic,
  Link2,
  Link2Off,
  List,
  ListOrdered,
  Minus,
  Quote,
  Redo2,
  RemoveFormatting,
  Strikethrough,
  Underline,
  Undo2,
} from "lucide-react";
import type { RichTextDocument } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Toggle } from "@/components/ui/toggle";
import { useTranslations } from "@/hooks/use-locale";
import {
  isRichTextBlank,
  toRichTextDocument,
  toTiptapContent,
} from "@/lib/rich-text";
import { cn } from "@/lib/utils";
import { createCardEditorExtensions } from "./extensions";

export type EditorLayoutWidth = "medium" | "large";

type RichTextEditorProps = {
  value: RichTextDocument;
  onChange: (doc: RichTextDocument) => void;
  placeholder?: string;
  disabled?: boolean;
  autoFocus?: boolean;
  id?: string;
  className?: string;
  "aria-label"?: string;
  layoutWidth?: EditorLayoutWidth;
};

function MarkToggle({
  pressed,
  onPressedChange,
  label,
  disabled,
  children,
}: {
  pressed: boolean;
  onPressedChange: () => void;
  label: string;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <Toggle
      size="sm"
      pressed={pressed}
      onPressedChange={() => onPressedChange()}
      disabled={disabled}
      aria-label={label}
      title={label}
    >
      {children}
    </Toggle>
  );
}

function ActionButton({
  onClick,
  label,
  disabled,
  pressed,
  children,
}: {
  onClick: () => void;
  label: string;
  disabled?: boolean;
  pressed?: boolean;
  children: ReactNode;
}) {
  return (
    <Button
      type="button"
      variant={pressed ? "secondary" : "ghost"}
      size="icon"
      className="size-8"
      disabled={disabled}
      aria-label={label}
      title={label}
      aria-pressed={pressed}
      onClick={onClick}
    >
      {children}
    </Button>
  );
}

function promptForLinkUrl(current: string, message: string): string | null {
  const next = window.prompt(message, current);
  if (next === null) return null;
  return next.trim();
}

function EditorToolbar({ editor }: { editor: Editor | null }) {
  const t = useTranslations();
  if (!editor) return null;

  const locked = !editor.isEditable;
  const canUndo = editor.can().undo();
  const canRedo = editor.can().redo();
  const linkActive = editor.isActive("link");

  function setLink() {
    if (!editor) return;
    const previous = editor.getAttributes("link").href as string | undefined;
    const url = promptForLinkUrl(previous ?? "https://", t.editor.linkPrompt);
    if (url === null) return;
    if (url === "") {
      editor.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }
    editor
      .chain()
      .focus()
      .extendMarkRange("link")
      .setLink({ href: url })
      .run();
  }

  return (
    <div
      className="flex flex-wrap items-center gap-0.5 border-b border-border/60 bg-muted/30 px-1.5 py-1"
      role="toolbar"
      aria-label={t.editor.toolbar}
    >
      <ActionButton
        disabled={locked || !canUndo}
        onClick={() => editor.chain().focus().undo().run()}
        label={t.editor.undo}
      >
        <Undo2 className="size-4" />
      </ActionButton>
      <ActionButton
        disabled={locked || !canRedo}
        onClick={() => editor.chain().focus().redo().run()}
        label={t.editor.redo}
      >
        <Redo2 className="size-4" />
      </ActionButton>
      <Separator orientation="vertical" className="mx-1 h-6" />
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("bold")}
        onPressedChange={() => editor.chain().focus().toggleBold().run()}
        label={t.editor.bold}
      >
        <Bold />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("italic")}
        onPressedChange={() => editor.chain().focus().toggleItalic().run()}
        label={t.editor.italic}
      >
        <Italic />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("underline")}
        onPressedChange={() => editor.chain().focus().toggleUnderline().run()}
        label={t.editor.underline}
      >
        <Underline />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("strike")}
        onPressedChange={() => editor.chain().focus().toggleStrike().run()}
        label={t.editor.strike}
      >
        <Strikethrough />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("code")}
        onPressedChange={() => editor.chain().focus().toggleCode().run()}
        label={t.editor.code}
      >
        <Code />
      </MarkToggle>
      <Separator orientation="vertical" className="mx-1 h-6" />
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("heading", { level: 1 })}
        onPressedChange={() =>
          editor.chain().focus().toggleHeading({ level: 1 }).run()
        }
        label={t.editor.heading1}
      >
        <Heading1 />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("heading", { level: 2 })}
        onPressedChange={() =>
          editor.chain().focus().toggleHeading({ level: 2 }).run()
        }
        label={t.editor.heading2}
      >
        <Heading2 />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("heading", { level: 3 })}
        onPressedChange={() =>
          editor.chain().focus().toggleHeading({ level: 3 }).run()
        }
        label={t.editor.heading3}
      >
        <Heading3 />
      </MarkToggle>
      <Separator orientation="vertical" className="mx-1 h-6" />
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("bulletList")}
        onPressedChange={() => editor.chain().focus().toggleBulletList().run()}
        label={t.editor.bulletList}
      >
        <List />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("orderedList")}
        onPressedChange={() => editor.chain().focus().toggleOrderedList().run()}
        label={t.editor.orderedList}
      >
        <ListOrdered />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("blockquote")}
        onPressedChange={() => editor.chain().focus().toggleBlockquote().run()}
        label={t.editor.blockquote}
      >
        <Quote />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={editor.isActive("codeBlock")}
        onPressedChange={() => editor.chain().focus().toggleCodeBlock().run()}
        label={t.editor.codeBlock}
      >
        <Code2 />
      </MarkToggle>
      <ActionButton
        disabled={locked}
        onClick={() => editor.chain().focus().setHorizontalRule().run()}
        label={t.editor.horizontalRule}
      >
        <Minus className="size-4" />
      </ActionButton>
      <Separator orientation="vertical" className="mx-1 h-6" />
      <MarkToggle
        disabled={locked}
        pressed={linkActive}
        onPressedChange={setLink}
        label={t.editor.link}
      >
        <Link2 />
      </MarkToggle>
      <ActionButton
        disabled={locked || !linkActive}
        onClick={() =>
          editor.chain().focus().extendMarkRange("link").unsetLink().run()
        }
        label={t.editor.unlink}
      >
        <Link2Off className="size-4" />
      </ActionButton>
      <ActionButton
        disabled={locked}
        onClick={() =>
          editor.chain().focus().unsetAllMarks().clearNodes().run()
        }
        label={t.editor.clearFormat}
      >
        <RemoveFormatting className="size-4" />
      </ActionButton>
    </div>
  );
}

export function RichTextEditor({
  value,
  onChange,
  placeholder,
  disabled = false,
  autoFocus = false,
  id,
  className,
  "aria-label": ariaLabel,
  layoutWidth = "medium",
}: RichTextEditorProps) {
  const editor = useEditor({
    immediatelyRender: false,
    shouldRerenderOnTransaction: true,
    extensions: createCardEditorExtensions(placeholder),
    content: toTiptapContent(value),
    editable: !disabled,
    autofocus: autoFocus ? "end" : false,
    editorProps: {
      attributes: {
        id: id ?? "",
        class: "jabex-editor-content",
        ...(ariaLabel ? { "aria-label": ariaLabel } : {}),
      },
    },
    onUpdate: ({ editor: next }) => {
      onChange(toRichTextDocument(next.getJSON()));
    },
  });

  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!disabled);
  }, [disabled, editor]);

  useEffect(() => {
    if (!editor) return;
    const current = toRichTextDocument(editor.getJSON());
    const nextJson = JSON.stringify(toTiptapContent(value));
    const currentJson = JSON.stringify(toTiptapContent(current));
    if (nextJson === currentJson) return;
    editor.commands.setContent(toTiptapContent(value), { emitUpdate: false });
  }, [editor, value]);

  return (
    <div
      className={cn(
        "overflow-hidden rounded-md border border-input bg-transparent shadow-sm focus-within:ring-1 focus-within:ring-ring",
        layoutWidth === "large" && "jabex-editor--large",
        disabled && "opacity-50",
        className,
      )}
      data-blank={isRichTextBlank(value) ? "true" : "false"}
      data-layout={layoutWidth}
    >
      <EditorToolbar editor={editor} />
      <EditorContent editor={editor} className="jabex-editor" />
    </div>
  );
}
