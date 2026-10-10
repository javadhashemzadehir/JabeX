import type { JSONContent } from "@tiptap/react";
import {
  RICH_TEXT_DOCUMENT_VERSION,
  type JabexJsonFace,
  type RichTextDocument,
  type RichTextDocumentVersion,
  type RichTextNode,
} from "@jabex/types";
import { z } from "zod";

const richTextNodeSchema: z.ZodType<RichTextNode> = z.record(z.string(), z.unknown());

export const richTextDocumentSchema: z.ZodType<RichTextDocument> = z.object({
  version: z.union([z.literal(1), z.literal(2)]),
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

/** Normalize TipTap editor JSON into a versioned JabeX document (always current write version). */
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
  const nodeType = typeof record.type === "string" ? record.type : "";

  if (typeof record.text === "string") {
    out.push(record.text);
  }

  if (
    (nodeType === "mathInline" || nodeType === "mathDisplay") &&
    record.attrs &&
    typeof record.attrs === "object"
  ) {
    const latex = (record.attrs as { latex?: unknown }).latex;
    if (typeof latex === "string" && latex.trim()) {
      out.push(latex);
    }
    return;
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

function coerceVersion(version: unknown): RichTextDocumentVersion {
  if (version === 1 || version === 2) return version;
  return RICH_TEXT_DOCUMENT_VERSION;
}

/** Accept legacy plain strings / v1 docs; normalize writes toward current version. */
export function normalizeRichTextInput(
  value: RichTextDocument | string | null | undefined,
): RichTextDocument {
  if (!value) return emptyRichTextDocument();
  if (typeof value === "string") return richTextFromPlainText(value);
  const parsed = richTextDocumentSchema.safeParse(value);
  if (!parsed.success) return emptyRichTextDocument();
  return {
    version: RICH_TEXT_DOCUMENT_VERSION,
    type: "doc",
    content: parsed.data.content,
  };
}

/** Upgrade stored document version metadata without rewriting nodes. */
export function upgradeDocumentVersion(doc: RichTextDocument): RichTextDocument {
  return {
    ...doc,
    version: RICH_TEXT_DOCUMENT_VERSION,
  };
}

export function toJabexJsonFace(doc: RichTextDocument): JabexJsonFace {
  return {
    format: "jabex-json",
    version: RICH_TEXT_DOCUMENT_VERSION,
    content: upgradeDocumentVersion(doc),
  };
}

export function fromJabexJsonFace(face: unknown): RichTextDocument | null {
  if (!face || typeof face !== "object") return null;
  const record = face as Record<string, unknown>;
  if (record.format === "jabex-json" && record.content) {
    return normalizeRichTextInput(record.content as RichTextDocument);
  }
  const asDoc = richTextDocumentSchema.safeParse(face);
  if (asDoc.success) {
    return {
      version: RICH_TEXT_DOCUMENT_VERSION,
      type: "doc",
      content: asDoc.data.content,
    };
  }
  return null;
}

export function parseRichTextJson(raw: string): RichTextDocument | null {
  try {
    const value = JSON.parse(raw) as unknown;
    if (typeof value === "string") return richTextFromPlainText(value);
    const fromFace = fromJabexJsonFace(value);
    if (fromFace) return fromFace;
    const parsed = richTextDocumentSchema.safeParse(value);
    if (parsed.success) {
      return {
        version: RICH_TEXT_DOCUMENT_VERSION,
        type: "doc",
        content: parsed.data.content,
      };
    }
    // Bare TipTap doc without version
    if (
      value &&
      typeof value === "object" &&
      (value as { type?: unknown }).type === "doc" &&
      Array.isArray((value as { content?: unknown }).content)
    ) {
      return toRichTextDocument(value as JSONContent);
    }
    return null;
  } catch {
    return null;
  }
}

export { coerceVersion };
