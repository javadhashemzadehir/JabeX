import type { Editor } from "@tiptap/react";
import type { RichTextDocument } from "@jabex/types";
import {
  looksLikeMarkdown,
  markdownToDocument,
} from "./rich-text-markdown";
import {
  fromJabexJsonFace,
  parseRichTextJson,
  toTiptapContent,
} from "./rich-text";

export type JsonPasteChoice = "document" | "code" | "cancel";

export type PasteDecision =
  | { kind: "handled" }
  | { kind: "ask-json"; text: string; parsed: RichTextDocument | null; isArrayOrObject: boolean }
  | { kind: "default" };

function clipboardText(event: ClipboardEvent): string {
  return event.clipboardData?.getData("text/plain") ?? "";
}

function clipboardHtml(event: ClipboardEvent): string {
  return event.clipboardData?.getData("text/html") ?? "";
}

function isLikelyChatGptHtml(html: string): boolean {
  return (
    html.includes("data-message-author-role") ||
    html.includes("chatgpt") ||
    html.includes('class="markdown"') ||
    html.includes("prose")
  );
}

/** Detect standalone JSON paste candidates (object/array). */
export function detectJsonPaste(text: string): {
  isJson: boolean;
  asDocument: RichTextDocument | null;
  pretty: string | null;
} {
  const trimmed = text.trim();
  if (!trimmed) return { isJson: false, asDocument: null, pretty: null };
  if (!(trimmed.startsWith("{") || trimmed.startsWith("["))) {
    return { isJson: false, asDocument: null, pretty: null };
  }
  try {
    const value = JSON.parse(trimmed) as unknown;
    if (value === null || typeof value !== "object") {
      return { isJson: false, asDocument: null, pretty: null };
    }
    const pretty = JSON.stringify(value, null, 2);
    const asDocument =
      fromJabexJsonFace(value) ??
      parseRichTextJson(pretty) ??
      (value &&
      typeof value === "object" &&
      (value as { type?: unknown }).type === "doc"
        ? parseRichTextJson(pretty)
        : null);
    return { isJson: true, asDocument, pretty };
  } catch {
    return { isJson: false, asDocument: null, pretty: null };
  }
}

export function applyJsonPasteChoice(
  editor: Editor,
  text: string,
  choice: JsonPasteChoice,
  asDocument: RichTextDocument | null,
): void {
  if (choice === "cancel") return;
  if (choice === "document" && asDocument) {
    editor.commands.insertContent(toTiptapContent(asDocument));
    return;
  }
  const pretty = (() => {
    try {
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      return text;
    }
  })();
  editor.commands.insertContent({
    type: "codeBlock",
    attrs: { language: "json" },
    content: pretty ? [{ type: "text", text: pretty }] : [],
  });
}

/**
 * Analyze a paste event. Returns whether the caller should preventDefault
 * and apply a custom path (including asking the user about JSON).
 */
export function analyzePaste(event: ClipboardEvent): PasteDecision {
  const text = clipboardText(event);
  const html = clipboardHtml(event);

  const json = detectJsonPaste(text);
  if (json.isJson && !html.trim()) {
    return {
      kind: "ask-json",
      text,
      parsed: json.asDocument,
      isArrayOrObject: true,
    };
  }

  // Prefer TipTap HTML parsing for rich clipboard (ChatGPT, browsers).
  // Schema strips unknown tags — safe default path.
  if (html.trim() && (isLikelyChatGptHtml(html) || /<(table|ul|ol|pre|h[1-3]|p)\b/i.test(html))) {
    return { kind: "default" };
  }

  if (text && looksLikeMarkdown(text)) {
    return { kind: "handled" };
  }

  return { kind: "default" };
}

/** Insert Markdown plain text as structured content. */
export function pasteMarkdown(editor: Editor, text: string): void {
  const doc = markdownToDocument(text);
  editor.commands.insertContent(toTiptapContent(doc).content ?? []);
}

/**
 * TipTap handlePaste factory. `onAskJson` is invoked when pasted text is JSON;
 * return a promise that resolves to the user's choice.
 */
export function createPasteHandler(options: {
  onAskJson: (payload: {
    text: string;
    asDocument: RichTextDocument | null;
  }) => Promise<JsonPasteChoice>;
}) {
  return ({ editor, event }: { editor: Editor; event: ClipboardEvent }) => {
    const decision = analyzePaste(event);

    if (decision.kind === "ask-json") {
      event.preventDefault();
      void options
        .onAskJson({ text: decision.text, asDocument: decision.parsed })
        .then((choice) => {
          applyJsonPasteChoice(editor, decision.text, choice, decision.parsed);
        });
      return true;
    }

    if (decision.kind === "handled") {
      const text = clipboardText(event);
      event.preventDefault();
      pasteMarkdown(editor, text);
      return true;
    }

    // Let TipTap parse HTML / plain text against the schema (XSS-safe).
    return false;
  };
}
