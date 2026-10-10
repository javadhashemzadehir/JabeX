import type { JSONContent } from "@tiptap/react";
import {
  RICH_TEXT_DOCUMENT_VERSION,
  type RichTextDocument,
  type RichTextNode,
} from "@jabex/types";
import { z } from "zod";

const richTextNodeSchema: z.ZodType<RichTextNode> = z.record(z.string(), z.unknown());

export const richTextDocumentSchema: z.ZodType<RichTextDocument> = z.object({
  version: z.literal(RICH_TEXT_DOCUMENT_VERSION),
  type: z.literal("doc"),
  content: z.array(richTextNodeSchema),
});

export function emptyRichTextDocument(): RichTextDocument {
  return {
    version: RICH_TEXT_DOCUMENT_VERSION,
    type: "doc",
    content: [{ type: "paragraph" }],
  };
}

export function richTextFromPlainText(text: string): RichTextDocument {
  const trimmed = text.trim();
  if (!trimmed) return emptyRichTextDocument();
  return {
    version: RICH_TEXT_DOCUMENT_VERSION,
    type: "doc",
    content: [
      {
        type: "paragraph",
        content: [{ type: "text", text: trimmed }],
      },
    ],
  };
}

/** Normalize TipTap editor JSON into a versioned JabeX document. */
export function toRichTextDocument(json: JSONContent | null | undefined): RichTextDocument {
  const content = Array.isArray(json?.content)
    ? (json.content as RichTextNode[])
    : [{ type: "paragraph" }];
  return {
    version: RICH_TEXT_DOCUMENT_VERSION,
    type: "doc",
    content,
  };
}

/** Strip the version wrapper for TipTap `content` prop. */
export function toTiptapContent(doc: RichTextDocument): JSONContent {
  return {
    type: "doc",
    content: doc.content as JSONContent["content"],
  };
}

function appendPlainText(node: unknown, out: string[]): void {
  if (!node || typeof node !== "object") return;
  const record = node as Record<string, unknown>;
  if (typeof record.text === "string") {
    out.push(record.text);
  }
  if (Array.isArray(record.content)) {
    for (const child of record.content) {
      appendPlainText(child, out);
    }
  }
}

export function plainTextFromDocument(doc: RichTextDocument | null | undefined): string {
  if (!doc) return "";
  const parts: string[] = [];
  for (const node of doc.content) {
    const before = parts.length;
    appendPlainText(node, parts);
    if (parts.length > before) {
      parts.push("\n");
    }
  }
  return parts.join("").replace(/\n+$/, "").trim();
}

export function isRichTextBlank(doc: RichTextDocument | null | undefined): boolean {
  return plainTextFromDocument(doc).length === 0;
}

/** Accept legacy plain strings (seed / older clients) or versioned documents. */
export function normalizeRichTextInput(
  value: RichTextDocument | string | null | undefined,
): RichTextDocument {
  if (!value) return emptyRichTextDocument();
  if (typeof value === "string") return richTextFromPlainText(value);
  const parsed = richTextDocumentSchema.safeParse(value);
  if (parsed.success) return parsed.data;
  return emptyRichTextDocument();
}
