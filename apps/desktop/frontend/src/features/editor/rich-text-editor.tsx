"use client";

import { useEffect, useRef, useState } from "react";
import { EditorContent, useEditor, type Editor } from "@tiptap/react";
import { Copy } from "lucide-react";
import type { RichTextDocument } from "@jabex/types";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useTranslations } from "@/hooks/use-locale";
import {
  documentToMarkdown,
  markdownToDocument,
} from "@/lib/rich-text-markdown";
import { createPasteHandler, type JsonPasteChoice } from "@/lib/rich-text-paste";
import {
  isRichTextBlank,
  parseRichTextJson,
  toRichTextDocument,
  toTiptapContent,
} from "@/lib/rich-text";
import { cn } from "@/lib/utils";
import { createCardEditorExtensions } from "./extensions";
import { EditorToolbar } from "./editor-toolbar";
import { RichTextViewer } from "./rich-text-viewer";
import { bindSpoilerInteractions } from "./spoiler-interaction";

export type EditorLayoutWidth = "medium" | "large";
export type EditorViewMode = "edit" | "preview" | "json" | "markdown";

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

type JsonPromptState = {
  text: string;
  asDocument: RichTextDocument | null;
  resolve: (choice: JsonPasteChoice) => void;
};

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
  const t = useTranslations();
  const [mode, setMode] = useState<EditorViewMode>("edit");
  const [markdownDraft, setMarkdownDraft] = useState("");
  const [jsonDraft, setJsonDraft] = useState("");
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [jsonPrompt, setJsonPrompt] = useState<JsonPromptState | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Editor | null>(null);
  const askJsonRef = useRef<
    (payload: {
      text: string;
      asDocument: RichTextDocument | null;
    }) => Promise<JsonPasteChoice>
  >(async () => "cancel");

  askJsonRef.current = ({ text, asDocument }) =>
    new Promise<JsonPasteChoice>((resolve) => {
      setJsonPrompt({ text, asDocument, resolve });
    });

  const editor = useEditor({
    immediatelyRender: false,
    shouldRerenderOnTransaction: true,
    extensions: createCardEditorExtensions({ placeholder }),
    content: toTiptapContent(value),
    editable: !disabled,
    autofocus: autoFocus ? "end" : false,
    editorProps: {
      attributes: {
        id: id ?? "",
        class: "jabex-editor-content",
        ...(ariaLabel ? { "aria-label": ariaLabel } : {}),
      },
      handlePaste: (_view, event) => {
        const current = editorRef.current;
        if (!current) return false;
        return createPasteHandler({
          onAskJson: (payload) => askJsonRef.current(payload),
        })({ editor: current, event: event as ClipboardEvent });
      },
    },
    onUpdate: ({ editor: next }) => {
      onChange(toRichTextDocument(next.getJSON()));
    },
  });

  useEffect(() => {
    editorRef.current = editor;
  }, [editor]);

  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!disabled && mode === "edit");
  }, [disabled, editor, mode]);

  useEffect(() => {
    if (!editor) return;
    const current = toRichTextDocument(editor.getJSON());
    const nextJson = JSON.stringify(toTiptapContent(value));
    const currentJson = JSON.stringify(toTiptapContent(current));
    if (nextJson === currentJson) return;
    editor.commands.setContent(toTiptapContent(value), { emitUpdate: false });
  }, [editor, value]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    return bindSpoilerInteractions(root);
  }, []);

  const prevModeRef = useRef<EditorViewMode>(mode);
  useEffect(() => {
    const entered = prevModeRef.current !== mode;
    prevModeRef.current = mode;
    if (!entered) return;
    if (mode === "markdown") {
      setMarkdownDraft(documentToMarkdown(value));
    }
    if (mode === "json") {
      setJsonDraft(JSON.stringify(value, null, 2));
      setJsonError(null);
    }
  }, [mode, value]);

  function switchMode(next: EditorViewMode) {
    if (mode === "markdown" && next !== "markdown") {
      onChange(markdownToDocument(markdownDraft));
    }
    if (mode === "json" && next !== "json") {
      const parsed = parseRichTextJson(jsonDraft);
      if (parsed) onChange(parsed);
    }
    setMode(next);
  }

  function copyText(text: string) {
    void navigator.clipboard.writeText(text);
  }

  const modes: { id: EditorViewMode; label: string }[] = [
    { id: "edit", label: t.editor.modeEdit },
    { id: "preview", label: t.editor.modePreview },
    { id: "json", label: t.editor.modeJson },
    { id: "markdown", label: t.editor.modeMarkdown },
  ];

  return (
    <div
      ref={rootRef}
      className={cn(
        "overflow-hidden rounded-md border border-input bg-transparent shadow-sm focus-within:ring-1 focus-within:ring-ring",
        layoutWidth === "large" && "jabex-editor--large",
        disabled && "opacity-50",
        className,
      )}
      data-blank={isRichTextBlank(value) ? "true" : "false"}
      data-layout={layoutWidth}
      data-mode={mode}
    >
      <div
        className="flex flex-wrap items-center gap-1 border-b border-border/60 bg-muted/20 px-1.5 py-1"
        role="tablist"
        aria-label={t.editor.modes}
      >
        {modes.map((item) => (
          <Button
            key={item.id}
            type="button"
            size="sm"
            variant={mode === item.id ? "secondary" : "ghost"}
            className="h-7 px-2.5 text-xs"
            role="tab"
            aria-selected={mode === item.id}
            disabled={disabled && item.id === "edit"}
            onClick={() => switchMode(item.id)}
          >
            {item.label}
          </Button>
        ))}
      </div>

      {mode === "edit" && <EditorToolbar editor={editor} />}

      {mode === "edit" && (
        <EditorContent editor={editor} className="jabex-editor" />
      )}

      {mode === "preview" && (
        <div className="p-3">
          <RichTextViewer document={value} />
        </div>
      )}

      {mode === "json" && (
        <div className="space-y-2 p-2">
          <div className="flex justify-end">
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="h-7 gap-1.5 text-xs"
              onClick={() => copyText(jsonDraft)}
            >
              <Copy className="size-3.5" />
              {t.editor.copy}
            </Button>
          </div>
          <textarea
            className="jabex-editor-raw min-h-40 w-full resize-y rounded-md border border-border/60 bg-muted/30 p-2 font-mono text-xs leading-relaxed"
            dir="ltr"
            spellCheck={false}
            value={jsonDraft}
            aria-label={t.editor.modeJson}
            disabled={disabled}
            onChange={(e) => {
              const next = e.target.value;
              setJsonDraft(next);
              const parsed = parseRichTextJson(next);
              if (!parsed) {
                setJsonError(t.editor.jsonInvalid);
                return;
              }
              setJsonError(null);
              onChange(parsed);
            }}
          />
          {jsonError ? (
            <p className="text-xs text-destructive" role="alert">
              {jsonError}
            </p>
          ) : null}
        </div>
      )}

      {mode === "markdown" && (
        <div className="space-y-2 p-2">
          <div className="flex justify-end">
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="h-7 gap-1.5 text-xs"
              onClick={() => copyText(markdownDraft)}
            >
              <Copy className="size-3.5" />
              {t.editor.copy}
            </Button>
          </div>
          <textarea
            className="jabex-editor-raw min-h-40 w-full resize-y rounded-md border border-border/60 bg-muted/30 p-2 font-mono text-xs leading-relaxed"
            dir="auto"
            spellCheck={false}
            value={markdownDraft}
            aria-label={t.editor.modeMarkdown}
            disabled={disabled}
            onChange={(e) => setMarkdownDraft(e.target.value)}
            onBlur={() => onChange(markdownToDocument(markdownDraft))}
          />
        </div>
      )}

      <Dialog
        open={jsonPrompt !== null}
        onOpenChange={(open) => {
          if (!open && jsonPrompt) {
            jsonPrompt.resolve("cancel");
            setJsonPrompt(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t.editor.jsonPasteTitle}</DialogTitle>
            <DialogDescription>{t.editor.jsonPasteDescription}</DialogDescription>
          </DialogHeader>
          <DialogFooter className="flex-col gap-2 sm:flex-row">
            {jsonPrompt?.asDocument ? (
              <Button
                type="button"
                onClick={() => {
                  jsonPrompt.resolve("document");
                  setJsonPrompt(null);
                }}
              >
                {t.editor.jsonPasteAsDocument}
              </Button>
            ) : null}
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                jsonPrompt?.resolve("code");
                setJsonPrompt(null);
              }}
            >
              {t.editor.jsonPasteAsCode}
            </Button>
            <Button
              type="button"
              variant="ghost"
              onClick={() => {
                jsonPrompt?.resolve("cancel");
                setJsonPrompt(null);
              }}
            >
              {t.editor.cancel}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
