import { describe, expect, it } from "vitest";
import {
  emptyRichTextDocument,
  isRichTextBlank,
  normalizeRichTextInput,
  plainTextFromDocument,
  richTextFromPlainText,
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

  it("round-trips TipTap content", () => {
    const doc = richTextFromPlainText("hello");
    const tip = toTiptapContent(doc);
    const back = toRichTextDocument(tip);
    expect(back.version).toBe(1);
    expect(back.type).toBe("doc");
    expect(plainTextFromDocument(back)).toBe("hello");
  });

  it("normalizes legacy strings", () => {
    const doc = normalizeRichTextInput("legacy");
    expect(plainTextFromDocument(doc)).toBe("legacy");
  });
});
