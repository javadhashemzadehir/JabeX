import { describe, expect, it } from "vitest";
import {
  documentToMarkdown,
  looksLikeMarkdown,
  markdownToDocument,
} from "./rich-text-markdown";
import { plainTextFromDocument, toRichTextDocument } from "./rich-text";

describe("rich-text markdown", () => {
  it("converts headings, lists, and bold", () => {
    const md = "## Title\n\n- item 1\n- item 2\n\n**bold**";
    const doc = markdownToDocument(md);
    const back = documentToMarkdown(doc);
    expect(back).toContain("## Title");
    expect(back).toContain("- item 1");
    expect(back).toContain("**bold**");
    expect(looksLikeMarkdown(md)).toBe(true);
  });

  it("round-trips tables", () => {
    const md = "| Word | Meaning |\n| --- | --- |\n| Apple | Fruit |";
    const doc = markdownToDocument(md);
    expect(doc.content.some((n) => n.type === "table")).toBe(true);
    const back = documentToMarkdown(doc);
    expect(back).toContain("| Word | Meaning |");
    expect(back).toContain("| Apple | Fruit |");
  });

  it("round-trips code fences with language", () => {
    const md = "```javascript\nconst x = 10;\n```";
    const doc = markdownToDocument(md);
    const block = doc.content.find((n) => n.type === "codeBlock") as {
      attrs?: { language?: string };
      content?: Array<{ text?: string }>;
    };
    expect(block?.attrs?.language).toBe("javascript");
    expect(block?.content?.[0]?.text).toContain("const x = 10");
    expect(documentToMarkdown(doc)).toContain("```javascript");
  });

  it("round-trips inline spoilers and math", () => {
    const md = "Hidden ||answer|| and $E=mc^2$";
    const doc = markdownToDocument(md);
    const back = documentToMarkdown(doc);
    expect(back).toContain("||answer||");
    expect(back).toContain("$E=mc^2$");
  });

  it("round-trips display math and spoiler blocks", () => {
    const md = "$$\n\\frac{a}{b}\n$$\n\n:::spoiler\nsecret\n:::";
    const doc = markdownToDocument(md);
    expect(doc.content.some((n) => n.type === "mathDisplay")).toBe(true);
    expect(doc.content.some((n) => n.type === "spoilerBlock")).toBe(true);
    const back = documentToMarkdown(doc);
    expect(back).toContain("\\frac{a}{b}");
    expect(back).toContain(":::spoiler");
  });

  it("preserves plain paragraphs from structured docs", () => {
    const doc = toRichTextDocument({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "سلام" }],
        },
      ],
    });
    expect(plainTextFromDocument(markdownToDocument(documentToMarkdown(doc)))).toBe(
      "سلام",
    );
  });
});
