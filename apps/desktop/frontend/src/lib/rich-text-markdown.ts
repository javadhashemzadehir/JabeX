import type { JSONContent } from "@tiptap/react";
import type { RichTextDocument, RichTextNode } from "@jabex/types";
import { marked } from "marked";
import {
  emptyRichTextDocument,
  toRichTextDocument,
} from "./rich-text";

type MarkName = "bold" | "italic" | "strike" | "code" | "underline" | "spoiler" | "link";

function escapeMd(text: string): string {
  return text.replace(/([\\`*_[\]{}()#+.!|-])/g, "\\$1");
}

function marksToWrappers(
  text: string,
  marks: Array<{ type: string; attrs?: Record<string, unknown> }> | undefined,
): string {
  if (!marks?.length) return escapeMd(text);
  let out = text;
  const order: MarkName[] = ["code", "spoiler", "bold", "italic", "strike", "underline", "link"];
  const sorted = [...marks].sort(
    (a, b) => order.indexOf(a.type as MarkName) - order.indexOf(b.type as MarkName),
  );
  for (const mark of sorted) {
    switch (mark.type) {
      case "bold":
        out = `**${out}**`;
        break;
      case "italic":
        out = `*${out}*`;
        break;
      case "strike":
        out = `~~${out}~~`;
        break;
      case "code":
        out = `\`${out}\``;
        break;
      case "underline":
        out = `<u>${out}</u>`;
        break;
      case "spoiler":
        out = `||${out}||`;
        break;
      case "link": {
        const href = String(mark.attrs?.href ?? "");
        out = `[${out}](${href})`;
        break;
      }
      default:
        break;
    }
  }
  return out;
}

function inlineToMarkdown(nodes: JSONContent[] | undefined): string {
  if (!nodes?.length) return "";
  return nodes
    .map((node) => {
      if (node.type === "text") {
        return marksToWrappers(
          node.text ?? "",
          node.marks as Array<{ type: string; attrs?: Record<string, unknown> }> | undefined,
        );
      }
      if (node.type === "hardBreak") return "  \n";
      if (node.type === "mathInline") {
        return `$${String(node.attrs?.latex ?? "")}$`;
      }
      return inlineToMarkdown(node.content);
    })
    .join("");
}

function cellText(cell: JSONContent): string {
  return inlineToMarkdown(cell.content)
    .replace(/\|/g, "\\|")
    .replace(/\n+/g, " ")
    .trim();
}

function tableToMarkdown(node: JSONContent): string {
  const rows = (node.content ?? []).filter((r) => r.type === "tableRow");
  if (!rows.length) return "";
  const matrix = rows.map((row) =>
    (row.content ?? []).map((cell) => cellText(cell)),
  );
  const width = Math.max(...matrix.map((r) => r.length), 0);
  const padded = matrix.map((r) => {
    const copy = [...r];
    while (copy.length < width) copy.push("");
    return copy;
  });
  const header = padded[0] ?? Array.from({ length: width }, () => "");
  const sep = header.map(() => "---");
  const body = padded.slice(1);
  const lines = [
    `| ${header.join(" | ")} |`,
    `| ${sep.join(" | ")} |`,
    ...body.map((r) => `| ${r.join(" | ")} |`),
  ];
  return lines.join("\n");
}

function blockToMarkdown(node: JSONContent, listIndent = 0): string {
  switch (node.type) {
    case "paragraph":
      return inlineToMarkdown(node.content);
    case "heading": {
      const level = Number(node.attrs?.level ?? 1);
      return `${"#".repeat(Math.min(Math.max(level, 1), 3))} ${inlineToMarkdown(node.content)}`;
    }
    case "bulletList":
      return (node.content ?? [])
        .map((item) => blockToMarkdown(item, listIndent))
        .join("\n");
    case "orderedList":
      return (node.content ?? [])
        .map((item, index) => {
          const indent = "  ".repeat(listIndent);
          const inner = (item.content ?? [])
            .map((child, childIndex) => {
              if (child.type === "paragraph") {
                const prefix = childIndex === 0 ? `${indent}${index + 1}. ` : `${indent}   `;
                return `${prefix}${inlineToMarkdown(child.content)}`;
              }
              return blockToMarkdown(child, listIndent + 1);
            })
            .join("\n");
          return inner;
        })
        .join("\n");
    case "listItem": {
      const indent = "  ".repeat(listIndent);
      return (node.content ?? [])
        .map((child, childIndex) => {
          if (child.type === "paragraph") {
            const prefix = childIndex === 0 ? `${indent}- ` : `${indent}  `;
            return `${prefix}${inlineToMarkdown(child.content)}`;
          }
          return blockToMarkdown(child, listIndent + 1);
        })
        .join("\n");
    }
    case "blockquote":
      return (node.content ?? [])
        .map((child) =>
          blockToMarkdown(child)
            .split("\n")
            .map((line) => `> ${line}`)
            .join("\n"),
        )
        .join("\n");
    case "codeBlock": {
      const lang = String(node.attrs?.language ?? "").replace(/[^a-zA-Z0-9_+#-]/g, "");
      const code = (node.content ?? [])
        .map((c) => c.text ?? "")
        .join("");
      return `\`\`\`${lang}\n${code}\n\`\`\``;
    }
    case "horizontalRule":
      return "---";
    case "mathDisplay":
      return `$$\n${String(node.attrs?.latex ?? "")}\n$$`;
    case "table":
      return tableToMarkdown(node);
    case "spoilerBlock": {
      const inner = (node.content ?? [])
        .map((child) => blockToMarkdown(child))
        .join("\n\n");
      return `:::spoiler\n${inner}\n:::`;
    }
    default:
      if (node.content) {
        return node.content.map((c) => blockToMarkdown(c, listIndent)).join("\n\n");
      }
      return "";
  }
}

/** Serialize a JabeX rich-text document to Markdown (spoilers, math, GFM tables). */
export function documentToMarkdown(doc: RichTextDocument): string {
  const blocks = (doc.content as JSONContent[]).map((node) => blockToMarkdown(node));
  return blocks.filter((b) => b.length > 0).join("\n\n").trim();
}

function textNode(
  text: string,
  marks?: Array<{ type: string; attrs?: Record<string, unknown> }>,
): JSONContent {
  return marks?.length ? { type: "text", text, marks } : { type: "text", text };
}

/** Parse Telegram-style ||spoiler|| and $math$ inside a plain string into inline nodes. */
export function parseInlineMarkdown(text: string): JSONContent[] {
  const nodes: JSONContent[] = [];
  const pattern =
    /(\|\|[\s\S]+?\|\|)|(\$\$[\s\S]+?\$\$)|(\$[^$\n]+?\$)|(\*\*[^*\n]+?\*\*)|(\*[^*\n]+?\*)|(~~[^~\n]+?~~)|(`[^`\n]+?`)|(\[[^\]]+?\]\([^)]+?\))/g;
  let last = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text)) !== null) {
    if (match.index > last) {
      nodes.push(textNode(text.slice(last, match.index)));
    }
    const token = match[0];
    if (token.startsWith("||") && token.endsWith("||")) {
      nodes.push(textNode(token.slice(2, -2), [{ type: "spoiler" }]));
    } else if (token.startsWith("$$") && token.endsWith("$$")) {
      nodes.push({
        type: "mathInline",
        attrs: { latex: token.slice(2, -2).trim() },
      });
    } else if (token.startsWith("$") && token.endsWith("$")) {
      nodes.push({
        type: "mathInline",
        attrs: { latex: token.slice(1, -1).trim() },
      });
    } else if (token.startsWith("**") && token.endsWith("**")) {
      nodes.push(textNode(token.slice(2, -2), [{ type: "bold" }]));
    } else if (token.startsWith("*") && token.endsWith("*")) {
      nodes.push(textNode(token.slice(1, -1), [{ type: "italic" }]));
    } else if (token.startsWith("~~") && token.endsWith("~~")) {
      nodes.push(textNode(token.slice(2, -2), [{ type: "strike" }]));
    } else if (token.startsWith("`") && token.endsWith("`")) {
      nodes.push(textNode(token.slice(1, -1), [{ type: "code" }]));
    } else if (token.startsWith("[")) {
      const m = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(token);
      if (m) {
        nodes.push(textNode(m[1], [{ type: "link", attrs: { href: m[2] } }]));
      } else {
        nodes.push(textNode(token));
      }
    }
    last = match.index + token.length;
  }
  if (last < text.length) {
    nodes.push(textNode(text.slice(last)));
  }
  return nodes.length ? nodes : [];
}

function paragraphFromText(text: string): JSONContent {
  const content = parseInlineMarkdown(text);
  return content.length ? { type: "paragraph", content } : { type: "paragraph" };
}

function parseTableBlock(lines: string[]): JSONContent | null {
  if (lines.length < 2) return null;
  const splitRow = (line: string) =>
    line
      .trim()
      .replace(/^\|/, "")
      .replace(/\|$/, "")
      .split("|")
      .map((c) => c.trim());
  const header = splitRow(lines[0]);
  const sep = splitRow(lines[1]);
  if (!sep.every((c) => /^:?-+:?$/.test(c))) return null;
  const body = lines.slice(2).map(splitRow);
  const toRow = (cells: string[], headerRow: boolean): JSONContent => ({
    type: "tableRow",
    content: cells.map((cell) => ({
      type: headerRow ? "tableHeader" : "tableCell",
      content: [paragraphFromText(cell)],
    })),
  });
  return {
    type: "table",
    content: [toRow(header, true), ...body.map((r) => toRow(r, false))],
  };
}

/**
 * Convert Markdown (GFM-ish + spoiler/math extensions) into a JabeX document.
 * Uses a line-oriented parser for reliable tables/fences; marked for HTML fallback.
 */
export function markdownToDocument(markdown: string): RichTextDocument {
  const trimmed = markdown.replace(/\r\n/g, "\n").trim();
  if (!trimmed) return emptyRichTextDocument();

  const lines = trimmed.split("\n");
  const blocks: JSONContent[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (!line.trim()) {
      i += 1;
      continue;
    }

    // Display math $$ ... $$
    if (line.trim() === "$$") {
      const body: string[] = [];
      i += 1;
      while (i < lines.length && lines[i].trim() !== "$$") {
        body.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) i += 1;
      blocks.push({ type: "mathDisplay", attrs: { latex: body.join("\n").trim() } });
      continue;
    }

    // Fenced code
    const fence = /^```([\w+-]*)\s*$/.exec(line.trim());
    if (fence) {
      const lang = fence[1] || "plaintext";
      const body: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        body.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) i += 1;
      blocks.push({
        type: "codeBlock",
        attrs: { language: lang },
        content: body.length ? [{ type: "text", text: body.join("\n") }] : [],
      });
      continue;
    }

    // Spoiler block :::spoiler ... :::
    if (line.trim() === ":::spoiler") {
      const body: string[] = [];
      i += 1;
      while (i < lines.length && lines[i].trim() !== ":::") {
        body.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) i += 1;
      const inner = markdownToDocument(body.join("\n"));
      blocks.push({
        type: "spoilerBlock",
        content: (inner.content as JSONContent[]).length
          ? (inner.content as JSONContent[])
          : [{ type: "paragraph" }],
      });
      continue;
    }

    // Table
    if (line.includes("|") && i + 1 < lines.length && /^\s*\|?\s*:?-+:?/.test(lines[i + 1])) {
      const tableLines: string[] = [];
      while (i < lines.length && lines[i].includes("|")) {
        tableLines.push(lines[i]);
        i += 1;
      }
      const table = parseTableBlock(tableLines);
      if (table) {
        blocks.push(table);
        continue;
      }
      // fall through if not a table
      i -= tableLines.length;
    }

    // Heading
    const heading = /^(#{1,3})\s+(.+)$/.exec(line);
    if (heading) {
      blocks.push({
        type: "heading",
        attrs: { level: heading[1].length },
        content: parseInlineMarkdown(heading[2]),
      });
      i += 1;
      continue;
    }

    // HR
    if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line.trim())) {
      blocks.push({ type: "horizontalRule" });
      i += 1;
      continue;
    }

    // Blockquote
    if (line.startsWith(">")) {
      const quoteLines: string[] = [];
      while (i < lines.length && lines[i].startsWith(">")) {
        quoteLines.push(lines[i].replace(/^>\s?/, ""));
        i += 1;
      }
      const inner = markdownToDocument(quoteLines.join("\n"));
      blocks.push({
        type: "blockquote",
        content: (inner.content as JSONContent[]).length
          ? (inner.content as JSONContent[])
          : [{ type: "paragraph" }],
      });
      continue;
    }

    // Lists
    if (/^\s*[-*+]\s+/.test(line) || /^\s*\d+\.\s+/.test(line)) {
      const ordered = /^\s*\d+\.\s+/.test(line);
      const items: JSONContent[] = [];
      while (
        i < lines.length &&
        (ordered ? /^\s*\d+\.\s+/.test(lines[i]) : /^\s*[-*+]\s+/.test(lines[i]))
      ) {
        const itemText = lines[i].replace(/^\s*(?:[-*+]|\d+\.)\s+/, "");
        items.push({
          type: "listItem",
          content: [paragraphFromText(itemText)],
        });
        i += 1;
      }
      blocks.push({
        type: ordered ? "orderedList" : "bulletList",
        content: items,
      });
      continue;
    }

    // Paragraph (consume until blank)
    const para: string[] = [line];
    i += 1;
    while (i < lines.length && lines[i].trim() && !isBlockStart(lines[i])) {
      para.push(lines[i]);
      i += 1;
    }
    blocks.push(paragraphFromText(para.join("\n")));
  }

  if (!blocks.length) {
    // Fallback: marked → strip to plain paragraphs if custom parser yielded nothing
    const html = marked.parse(trimmed, { async: false }) as string;
    if (html.trim()) {
      return toRichTextDocument({
        type: "doc",
        content: [paragraphFromText(trimmed.replace(/<[^>]+>/g, ""))],
      });
    }
    return emptyRichTextDocument();
  }

  return toRichTextDocument({ type: "doc", content: blocks as RichTextNode[] });
}

function isBlockStart(line: string): boolean {
  return (
    /^#{1,3}\s/.test(line) ||
    /^```/.test(line.trim()) ||
    line.trim() === "$$" ||
    line.trim() === ":::spoiler" ||
    line.startsWith(">") ||
    /^\s*[-*+]\s+/.test(line) ||
    /^\s*\d+\.\s+/.test(line) ||
    (line.includes("|") && true)
  );
}

/** Heuristic: does plain text look like Markdown worth converting? */
export function looksLikeMarkdown(text: string): boolean {
  const t = text.trim();
  if (!t) return false;
  if (/^#{1,3}\s/m.test(t)) return true;
  if (/```/.test(t)) return true;
  if (/^\s*[-*+]\s+\S/m.test(t)) return true;
  if (/^\s*\d+\.\s+\S/m.test(t)) return true;
  if (/\|.+\|/.test(t) && /---/.test(t)) return true;
  if (/\*\*[^*\n]+\*\*/.test(t)) return true;
  if (/\|\|[^|]+\|\|/.test(t)) return true;
  if (/\$\$[\s\S]+\$\$/.test(t) || /\$[^$\n]+\$/.test(t)) return true;
  if (/^>\s/m.test(t)) return true;
  return false;
}
