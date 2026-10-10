import type { InkDocument, InkLayer, InkStroke, InkTool } from "@jabex/types";

export type { InkDocument, InkLayer, InkStroke, InkTool };

export type InkViewport = {
  scale: number;
  offsetX: number;
  offsetY: number;
};

export type InkRenderer = {
  drawStroke: (stroke: InkStroke) => void;
  eraseStroke: (strokeId: string) => void;
  setLayerVisibility: (layerId: string, visible: boolean) => void;
  setViewport: (viewport: InkViewport) => void;
  clear: () => void;
  destroy: () => void;
};
