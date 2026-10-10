import { describe, expect, it } from "vitest";
import { InkHistory } from "./history";
import {
  clearLayer,
  createStroke,
  parseInk,
  serializeInk,
  toggleLayerVisibility,
} from "./serialize";
import type { InkDocument } from "@jabex/types";

function sampleDoc(): InkDocument {
  return {
    version: 1,
    id: "aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa",
    width: 400,
    height: 300,
    layers: [
      {
        id: "bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb",
        order: 0,
        visible: true,
        opacity: 1,
        strokes: [],
      },
    ],
  };
}

describe("ink serialization", () => {
  it("roundtrips JSON", () => {
    const doc = sampleDoc();
    const stroke = createStroke("pen", "#111827", 2);
    stroke.points.push({ x: 1, y: 2, pressure: 1, timestamp: 0 });
    doc.layers[0]!.strokes.push(stroke);
    const again = parseInk(serializeInk(doc));
    expect(again.layers[0]!.strokes).toHaveLength(1);
    expect(again.layers[0]!.strokes[0]!.points[0]!.x).toBe(1);
  });
});

describe("ink undo/redo", () => {
  it("undoes and redoes commits", () => {
    const history = new InkHistory(sampleDoc());
    const layerId = history.document.layers[0]!.id;
    history.commit(clearLayer(history.document, layerId));
    const withStroke = structuredClone(history.document);
    withStroke.layers[0]!.strokes.push(createStroke("pen", "#000", 1));
    history.commit(withStroke);
    expect(history.document.layers[0]!.strokes).toHaveLength(1);
    history.undo();
    expect(history.document.layers[0]!.strokes).toHaveLength(0);
    history.redo();
    expect(history.document.layers[0]!.strokes).toHaveLength(1);
  });

  it("toggles layer visibility", () => {
    const doc = sampleDoc();
    const next = toggleLayerVisibility(doc, doc.layers[0]!.id);
    expect(next.layers[0]!.visible).toBe(false);
  });
});
