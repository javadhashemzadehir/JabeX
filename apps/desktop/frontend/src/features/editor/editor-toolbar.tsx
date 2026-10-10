"use client";

import type { ReactNode } from "react";
import type { Editor } from "@tiptap/react";
import {
  Bold,
  Code,
  Code2,
  EyeOff,
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
  Sigma,
  Strikethrough,
  Table2,
  Underline,
  Undo2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { Toggle } from "@/components/ui/toggle";
import { useTranslations } from "@/hooks/use-locale";
import { cn } from "@/lib/utils";

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
      className="shrink-0"
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
      className="size-8 shrink-0"
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

export function EditorToolbar({
  editor,
  className,
}: {
  editor: Editor | null;
  className?: string;
}) {
  const t = useTranslations();
  if (!editor) return null;

  const ed = editor;
  const locked = !ed.isEditable;
  const canUndo = ed.can().undo();
  const canRedo = ed.can().redo();
  const linkActive = ed.isActive("link");
  const inTable = ed.isActive("table");

  function setLink() {
    const previous = ed.getAttributes("link").href as string | undefined;
    const url = promptForLinkUrl(previous ?? "https://", t.editor.linkPrompt);
    if (url === null) return;
    if (url === "") {
      ed.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }
    ed.chain().focus().extendMarkRange("link").setLink({ href: url }).run();
  }

  function insertMath(display: boolean) {
    const latex = window.prompt(
      display ? t.editor.mathDisplayPrompt : t.editor.mathInlinePrompt,
      display ? "\\int_0^1 x^2\\,dx" : "E=mc^2",
    );
    if (latex === null || !latex.trim()) return;
    if (display) {
      ed.chain().focus().setMathDisplay(latex.trim()).run();
    } else {
      ed.chain().focus().setMathInline(latex.trim()).run();
    }
  }

  return (
    <div
      className={cn(
        "flex items-center gap-0.5 overflow-x-auto border-b border-border/60 bg-muted/30 px-1.5 py-1 scrollbar-thin",
        className,
      )}
      role="toolbar"
      aria-label={t.editor.toolbar}
    >
      <ActionButton
        disabled={locked || !canUndo}
        onClick={() => ed.chain().focus().undo().run()}
        label={t.editor.undo}
      >
        <Undo2 className="size-4" />
      </ActionButton>
      <ActionButton
        disabled={locked || !canRedo}
        onClick={() => ed.chain().focus().redo().run()}
        label={t.editor.redo}
      >
        <Redo2 className="size-4" />
      </ActionButton>
      <Separator orientation="vertical" className="mx-1 h-6 shrink-0" />

      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("bold")}
        onPressedChange={() => ed.chain().focus().toggleBold().run()}
        label={t.editor.bold}
      >
        <Bold />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("italic")}
        onPressedChange={() => ed.chain().focus().toggleItalic().run()}
        label={t.editor.italic}
      >
        <Italic />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("underline")}
        onPressedChange={() => ed.chain().focus().toggleUnderline().run()}
        label={t.editor.underline}
      >
        <Underline />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("strike")}
        onPressedChange={() => ed.chain().focus().toggleStrike().run()}
        label={t.editor.strike}
      >
        <Strikethrough />
      </MarkToggle>
      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("code")}
        onPressedChange={() => ed.chain().focus().toggleCode().run()}
        label={t.editor.code}
      >
        <Code />
      </MarkToggle>
      <Separator orientation="vertical" className="mx-1 h-6 shrink-0" />

      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("spoiler")}
        onPressedChange={() => ed.chain().focus().toggleSpoiler().run()}
        label={t.editor.spoiler}
      >
        <EyeOff />
      </MarkToggle>
      <ActionButton
        disabled={locked}
        pressed={ed.isActive("spoilerBlock")}
        onClick={() => ed.chain().focus().toggleSpoilerBlock().run()}
        label={t.editor.spoilerBlock}
      >
        <EyeOff className="size-4 opacity-70" />
      </ActionButton>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="size-8 shrink-0"
            disabled={locked}
            aria-label={t.editor.math}
            title={t.editor.math}
          >
            <Sigma className="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem onSelect={() => insertMath(false)}>
            {t.editor.mathInline}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => insertMath(true)}>
            {t.editor.mathDisplay}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant={inTable ? "secondary" : "ghost"}
            size="icon"
            className="size-8 shrink-0"
            disabled={locked}
            aria-label={t.editor.table}
            title={t.editor.table}
          >
            <Table2 className="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem
            onSelect={() =>
              ed
                .chain()
                .focus()
                .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
                .run()
            }
          >
            {t.editor.tableInsert}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().addRowBefore().run()}
          >
            {t.editor.tableAddRowBefore}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().addRowAfter().run()}
          >
            {t.editor.tableAddRowAfter}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().deleteRow().run()}
          >
            {t.editor.tableDeleteRow}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().addColumnBefore().run()}
          >
            {t.editor.tableAddColBefore}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().addColumnAfter().run()}
          >
            {t.editor.tableAddColAfter}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().deleteColumn().run()}
          >
            {t.editor.tableDeleteCol}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().mergeCells().run()}
          >
            {t.editor.tableMerge}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().splitCell().run()}
          >
            {t.editor.tableSplit}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!inTable}
            onSelect={() => ed.chain().focus().deleteTable().run()}
          >
            {t.editor.tableDelete}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <MarkToggle
        disabled={locked}
        pressed={ed.isActive("codeBlock")}
        onPressedChange={() => ed.chain().focus().toggleCodeBlock().run()}
        label={t.editor.codeBlock}
      >
        <Code2 />
      </MarkToggle>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-8 shrink-0 px-2 text-xs"
            disabled={locked}
            aria-label={t.editor.more}
          >
            {t.editor.more}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-44">
          <DropdownMenuItem
            onSelect={() =>
              ed.chain().focus().toggleHeading({ level: 1 }).run()
            }
          >
            <Heading1 className="size-4" />
            {t.editor.heading1}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() =>
              ed.chain().focus().toggleHeading({ level: 2 }).run()
            }
          >
            <Heading2 className="size-4" />
            {t.editor.heading2}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() =>
              ed.chain().focus().toggleHeading({ level: 3 }).run()
            }
          >
            <Heading3 className="size-4" />
            {t.editor.heading3}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onSelect={() => ed.chain().focus().toggleBulletList().run()}
          >
            <List className="size-4" />
            {t.editor.bulletList}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() => ed.chain().focus().toggleOrderedList().run()}
          >
            <ListOrdered className="size-4" />
            {t.editor.orderedList}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() => ed.chain().focus().toggleBlockquote().run()}
          >
            <Quote className="size-4" />
            {t.editor.blockquote}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() => ed.chain().focus().setHorizontalRule().run()}
          >
            <Minus className="size-4" />
            {t.editor.horizontalRule}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={setLink}>
            <Link2 className="size-4" />
            {t.editor.link}
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={!linkActive}
            onSelect={() =>
              ed.chain().focus().extendMarkRange("link").unsetLink().run()
            }
          >
            <Link2Off className="size-4" />
            {t.editor.unlink}
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() =>
              ed.chain().focus().unsetAllMarks().clearNodes().run()
            }
          >
            <RemoveFormatting className="size-4" />
            {t.editor.clearFormat}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
