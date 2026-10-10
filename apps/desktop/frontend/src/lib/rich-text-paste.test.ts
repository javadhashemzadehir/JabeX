import { describe, expect, it } from "vitest";
import {
  analyzePaste,
  detectJsonPaste,
} from "./rich-text-paste";
import { looksLikeMarkdown } from "./rich-text-markdown";

function fakeClipboardEvent(data: {
  text?: string;
  html?: string;
}): ClipboardEvent {
  return {
    clipboardData: {
      getData: (type: string) => {
        if (type === "text/plain") return data.text ?? "";
        if (type === "text/html") return data.html ?? "";
        return "";
      },
    },
    preventDefault: () => undefined,
  } as unknown as ClipboardEvent;
}

describe("rich-text paste", () => {
  it("detects JSON objects and tipTap documents", () => {
    const tipTap = JSON.stringify({
      version: 2,
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: "x" }] }],
    });
    const detected = detectJsonPaste(tipTap);
    expect(detected.isJson).toBe(true);
    expect(detected.asDocument).not.toBeNull();
  });

  it("asks on pure JSON paste without HTML", () => {
    const decision = analyzePaste(
      fakeClipboardEvent({ text: '{"a":1}', html: "" }),
    );
    expect(decision.kind).toBe("ask-json");
  });

  it("handles markdown-looking plain text", () => {
    const md = "## Title\n\n- one\n- two";
    expect(looksLikeMarkdown(md)).toBe(true);
    const decision = analyzePaste(fakeClipboardEvent({ text: md, html: "" }));
    expect(decision.kind).toBe("handled");
  });

  it("defers ChatGPT-like HTML to TipTap schema parse", () => {
    const decision = analyzePaste(
      fakeClipboardEvent({
        text: "Title",
        html: '<div class="markdown"><h2>Title</h2><ul><li>a</li></ul></div>',
      }),
    );
    expect(decision.kind).toBe("default");
  });

  it("detects GFM tables as markdown", () => {
    const table = "| A | B |\n| --- | --- |\n| 1 | 2 |";
    expect(looksLikeMarkdown(table)).toBe(true);
  });
});
