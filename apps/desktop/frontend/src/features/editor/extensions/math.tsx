"use client";

import { Node, mergeAttributes } from "@tiptap/core";
import {
  NodeViewWrapper,
  ReactNodeViewRenderer,
  type NodeViewProps,
} from "@tiptap/react";
import katex from "katex";
import { useMemo, type MouseEvent } from "react";
import { cn } from "@/lib/utils";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    mathInline: {
      setMathInline: (latex: string) => ReturnType;
    };
    mathDisplay: {
      setMathDisplay: (latex: string) => ReturnType;
    };
  }
}

function renderKatex(latex: string, displayMode: boolean): {
  html: string;
  error: string | null;
} {
  try {
    return {
      html: katex.renderToString(latex || "", {
        displayMode,
        throwOnError: true,
        strict: "ignore",
        trust: false,
      }),
      error: null,
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Invalid formula";
    return { html: "", error: message };
  }
}

function MathNodeView({
  node,
  selected,
  updateAttributes,
  editor,
  displayMode,
}: NodeViewProps & { displayMode: boolean }) {
  const latex = String(node.attrs.latex ?? "");
  const { html, error } = useMemo(
    () => renderKatex(latex, displayMode),
    [latex, displayMode],
  );

  function editLatex() {
    if (!editor.isEditable) return;
    const next = window.prompt(
      displayMode ? "Display math (LaTeX)" : "Inline math (LaTeX)",
      latex,
    );
    if (next === null) return;
    updateAttributes({ latex: next.trim() });
  }

  return (
    <NodeViewWrapper
      as={displayMode ? "div" : "span"}
      className={cn(
        "jabex-math",
        displayMode ? "jabex-math--display" : "jabex-math--inline",
        selected && "jabex-math--selected",
        error && "jabex-math--error",
      )}
      dir="ltr"
      data-latex={latex}
      contentEditable={false}
      onClick={(e: MouseEvent) => {
        if (!editor.isEditable) return;
        e.preventDefault();
        editLatex();
      }}
      onDoubleClick={(e: MouseEvent) => {
        e.preventDefault();
        editLatex();
      }}
      title={error ?? latex}
    >
      {error ? (
        <span className="jabex-math__fallback">{latex || "?"}</span>
      ) : (
        <span
          className="jabex-math__render"
          // KaTeX output is generated locally from user LaTeX with trust:false.
          dangerouslySetInnerHTML={{ __html: html }}
        />
      )}
    </NodeViewWrapper>
  );
}

function InlineMathView(props: NodeViewProps) {
  return <MathNodeView {...props} displayMode={false} />;
}

function DisplayMathView(props: NodeViewProps) {
  return <MathNodeView {...props} displayMode />;
}

export const MathInline = Node.create({
  name: "mathInline",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      latex: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-latex") ?? "",
        renderHTML: (attributes) => ({
          "data-latex": attributes.latex as string,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-math-inline]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-math-inline": "",
        class: "jabex-math jabex-math--inline",
        dir: "ltr",
      }),
    ];
  },

  addNodeView() {
    return ReactNodeViewRenderer(InlineMathView);
  },

  addCommands() {
    return {
      setMathInline:
        (latex: string) =>
        ({ commands }) =>
          commands.insertContent({
            type: this.name,
            attrs: { latex },
          }),
    };
  },
});

export const MathDisplay = Node.create({
  name: "mathDisplay",
  group: "block",
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      latex: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-latex") ?? "",
        renderHTML: (attributes) => ({
          "data-latex": attributes.latex as string,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-math-display]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "div",
      mergeAttributes(HTMLAttributes, {
        "data-math-display": "",
        class: "jabex-math jabex-math--display",
        dir: "ltr",
      }),
    ];
  },

  addNodeView() {
    return ReactNodeViewRenderer(DisplayMathView);
  },

  addCommands() {
    return {
      setMathDisplay:
        (latex: string) =>
        ({ commands }) =>
          commands.insertContent({
            type: this.name,
            attrs: { latex },
          }),
    };
  },
});
