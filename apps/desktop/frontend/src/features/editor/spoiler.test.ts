import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import { createCardEditorExtensions } from "./extensions";

describe("spoiler extensions", () => {
  it("toggles inline spoiler mark", () => {
    const editor = new Editor({
      extensions: createCardEditorExtensions({ placeholder: "" }),
      content: {
        type: "doc",
        content: [
          {
            type: "paragraph",
            content: [{ type: "text", text: "secret" }],
          },
        ],
      },
    });
    editor.commands.selectAll();
    editor.commands.toggleSpoiler();
    const json = editor.getJSON();
    const textNode = json.content?.[0]?.content?.[0];
    expect(textNode?.marks?.some((m) => m.type === "spoiler")).toBe(true);
    editor.destroy();
  });

  it("wraps selection in spoiler block", () => {
    const editor = new Editor({
      extensions: createCardEditorExtensions({ placeholder: "" }),
      content: {
        type: "doc",
        content: [
          {
            type: "paragraph",
            content: [{ type: "text", text: "hidden" }],
          },
        ],
      },
    });
    editor.commands.selectAll();
    editor.commands.setSpoilerBlock();
    expect(editor.getJSON().content?.[0]?.type).toBe("spoilerBlock");
    editor.destroy();
  });

  it("inserts math nodes", () => {
    const editor = new Editor({
      extensions: createCardEditorExtensions({ placeholder: "" }),
      content: { type: "doc", content: [{ type: "paragraph" }] },
    });
    editor.commands.setMathInline("a^2");
    const inline = JSON.stringify(editor.getJSON());
    expect(inline).toContain("mathInline");
    expect(inline).toContain("a^2");
    editor.commands.setMathDisplay("\\frac{1}{2}");
    expect(JSON.stringify(editor.getJSON())).toContain("mathDisplay");
    editor.destroy();
  });
});
