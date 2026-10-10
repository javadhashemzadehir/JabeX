import type { InkDocument, InkLayer, InkStroke, InkTool } from "@jabex/types";

export function cloneInkDocument(doc: InkDocument): InkDocument {
  return structuredClone(doc);
}

export function createStroke(
  tool: InkTool,
  color: string,
  width: number,
): InkStroke {
  return {
    id: crypto.randomUUID(),
    tool,
    color,
    width,
    points: [],
  };
}

export function clearLayer(doc: InkDocument, layerId: string): InkDocument {
  const next = cloneInkDocument(doc);
  const layer = next.layers.find((l) => l.id === layerId);
  if (layer) layer.strokes = [];
  return next;
}

export function toggleLayerVisibility(
  doc: InkDocument,
  layerId: string,
): InkDocument {
  const next = cloneInkDocument(doc);
  const layer = next.layers.find((l) => l.id === layerId);
  if (layer) layer.visible = !layer.visible;
  return next;
}

export function activeLayer(doc: InkDocument): InkLayer {
  return [...doc.layers].sort((a, b) => a.order - b.order)[0]!;
}

export function serializeInk(doc: InkDocument): string {
  return JSON.stringify(doc);
}

export function parseInk(raw: string): InkDocument {
  const parsed = JSON.parse(raw) as InkDocument;
  if (parsed.version !== 1 || !Array.isArray(parsed.layers)) {
    throw new Error("invalid ink document");
  }
  return parsed;
}
