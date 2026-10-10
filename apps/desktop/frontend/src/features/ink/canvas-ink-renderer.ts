import type { InkDocument, InkStroke } from "@jabex/types";
import type { InkRenderer, InkViewport } from "./types";

function strokeStyle(stroke: InkStroke): {
  color: string;
  width: number;
  alpha: number;
} {
  if (stroke.tool === "highlighter") {
    return { color: stroke.color, width: stroke.width * 2, alpha: 0.35 };
  }
  if (stroke.tool === "eraser") {
    return { color: "#000000", width: stroke.width * 3, alpha: 1 };
  }
  return { color: stroke.color, width: stroke.width, alpha: 1 };
}

export function createCanvasInkRenderer(
  canvas: HTMLCanvasElement,
): InkRenderer & { renderDocument: (doc: InkDocument) => void } {
  const maybeCtx = canvas.getContext("2d");
  if (!maybeCtx) {
    throw new Error("2d context unavailable");
  }
  const ctx: CanvasRenderingContext2D = maybeCtx;

  let viewport: InkViewport = { scale: 1, offsetX: 0, offsetY: 0 };
  let document: InkDocument | null = null;

  function clearCanvas() {
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "transparent";
  }

  function applyViewport() {
    ctx.setTransform(
      viewport.scale,
      0,
      0,
      viewport.scale,
      viewport.offsetX,
      viewport.offsetY,
    );
  }

  function paintStroke(stroke: InkStroke) {
    if (stroke.points.length < 2) return;
    const style = strokeStyle(stroke);
    ctx.save();
    if (stroke.tool === "eraser") {
      ctx.globalCompositeOperation = "destination-out";
    } else {
      ctx.globalCompositeOperation = "source-over";
      ctx.globalAlpha = style.alpha;
    }
    ctx.strokeStyle = style.color;
    ctx.lineWidth = style.width;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();
    ctx.moveTo(stroke.points[0]!.x, stroke.points[0]!.y);
    for (let i = 1; i < stroke.points.length; i += 1) {
      const point = stroke.points[i]!;
      ctx.lineWidth = style.width * (0.5 + point.pressure * 0.5);
      ctx.lineTo(point.x, point.y);
    }
    ctx.stroke();
    ctx.restore();
  }

  function renderDocument(doc: InkDocument) {
    document = doc;
    clearCanvas();
    applyViewport();
    const layers = [...doc.layers].sort((a, b) => a.order - b.order);
    for (const layer of layers) {
      if (!layer.visible) continue;
      ctx.globalAlpha = layer.opacity;
      for (const stroke of layer.strokes) {
        paintStroke(stroke);
      }
    }
    ctx.globalAlpha = 1;
  }

  return {
    renderDocument,
    drawStroke(stroke) {
      if (!document) return;
      applyViewport();
      paintStroke(stroke);
    },
    eraseStroke(strokeId) {
      if (!document) return;
      document = {
        ...document,
        layers: document.layers.map((layer) => ({
          ...layer,
          strokes: layer.strokes.filter((s) => s.id !== strokeId),
        })),
      };
      renderDocument(document);
    },
    setLayerVisibility(layerId, visible) {
      if (!document) return;
      document = {
        ...document,
        layers: document.layers.map((layer) =>
          layer.id === layerId ? { ...layer, visible } : layer,
        ),
      };
      renderDocument(document);
    },
    setViewport(next) {
      viewport = next;
      if (document) renderDocument(document);
    },
    clear() {
      clearCanvas();
    },
    destroy() {
      document = null;
      clearCanvas();
    },
  };
}
