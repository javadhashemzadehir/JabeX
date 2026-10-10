import { describe, expect, it } from "vitest";
import { RICH_TEXT_DOCUMENT_VERSION } from "@jabex/types";
import {
  emptyRichTextDocument,
  isRichTextBlank,
  normalizeRichTextInput,
  parseRichTextJson,
  plainTextFromDocument,
  richTextFromPlainText,
  toJabexJsonFace,
  toRichTextDocument,
  toTiptapContent,
} from "./rich-text";

describe("rich-text helpers", () => {
  it("builds and extracts plain text", () => {
    const doc = richTextFromPlainText("  سلام  ");
    expect(plainTextFromDocument(doc)).toBe("سلام");
    expect(isRichTextBlank(doc)).toBe(false);
    expect(isRichTextBlank(emptyRichTextDocument())).toBe(true);
  });

  it("round-trips TipTap content at current version", () => {
    const doc = richTextFromPlainText("hello");
    const tip = toTiptapContent(doc);
    const back = toRichTextDocument(tip);
    expect(back.version).toBe(RICH_TEXT_DOCUMENT_VERSION);
    expect(back.type).toBe("doc");
    expect(plainTextFromDocument(back)).toBe("hello");
  });

  it("normalizes legacy strings and v1 docs to write version", () => {
    const fromString = normalizeRichTextInput("legacy");
    expect(plainTextFromDocument(fromString)).toBe("legacy");
    expect(fromString.version).toBe(2);

    const fromV1 = normalizeRichTextInput({
      version: 1,
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "v1" }],
        },
      ],
    });
    expect(fromV1.version).toBe(2);
    expect(plainTextFromDocument(fromV1)).toBe("v1");
  });

  it("treats math-only documents as non-blank", () => {
    const doc = toRichTextDocument({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "mathInline", attrs: { latex: "E=mc^2" } }],
        },
      ],
    });
    expect(plainTextFromDocument(doc)).toBe("E=mc^2");
    expect(isRichTextBlank(doc)).toBe(false);
  });

  it("parses jabex-json face envelopes", () => {
    const face = toJabexJsonFace(richTextFromPlainText("face"));
    expect(face.format).toBe("jabex-json");
    const parsed = parseRichTextJson(JSON.stringify(face));
    expect(parsed).not.toBeNull();
    expect(plainTextFromDocument(parsed!)).toBe("face");
  });
});
