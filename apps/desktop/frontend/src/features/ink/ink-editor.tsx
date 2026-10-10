"use client";

import {
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import type { InkDocument, InkTool } from "@jabex/types";
import {
  Eraser,
  Eye,
  EyeOff,
  Highlighter,
  Pen,
  Redo2,
  Trash2,
  Undo2,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ColorPicker } from "@/components/ui/color-picker";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Spinner } from "@/components/ui/spinner";
import { Toggle } from "@/components/ui/toggle";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { createCanvasInkRenderer } from "./canvas-ink-renderer";
import { InkHistory } from "./history";
import {
  activeLayer,
  clearLayer,
  createStroke,
  toggleLayerVisibility,
} from "./serialize";

type InkEditorProps = {
  cardId: string;
  className?: string;
};

const SAVE_DEBOUNCE_MS = 600;

export function InkEditor({ cardId, className }: InkEditorProps) {
  const t = useTranslations();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<ReturnType<typeof createCanvasInkRenderer> | null>(
    null,
  );
  const historyRef = useRef<InkHistory | null>(null);
  const drawingRef = useRef(false);
  const strokeRef = useRef<ReturnType<typeof createStroke> | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const panRef = useRef<{ x: number; y: number } | null>(null);

  const [loading, setLoading] = useState(true);
  const [tool, setTool] = useState<InkTool>("pen");
  const [color, setColor] = useState("#111827");
  const [width, setWidth] = useState(2.5);
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [doc, setDoc] = useState<InkDocument | null>(null);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  function syncHistoryFlags() {
    const history = historyRef.current;
    setCanUndo(!!history?.canUndo());
    setCanRedo(!!history?.canRedo());
  }

  function scheduleSave(next: InkDocument) {
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      void api.saveInkDocument(next).catch((err) => {
        toast.error(err instanceof Error ? err.message : t.ink.error);
      });
    }, SAVE_DEBOUNCE_MS);
  }

  function applyDocument(next: InkDocument, commit: boolean) {
    if (commit) {
      historyRef.current?.commit(next);
    } else {
      historyRef.current?.replace(next);
    }
    setDoc(next);
    rendererRef.current?.renderDocument(next);
    syncHistoryFlags();
    scheduleSave(next);
  }

  useEffect(() => {
    let cancelled = false;
    async function boot() {
      setLoading(true);
      try {
        const existing = await api.listInkDocuments("card", cardId);
        let bundle =
          existing.length > 0
            ? await api.getInkDocument(existing[0]!.id)
            : await api.createInkDocument({
                ownerType: "card",
                ownerId: cardId,
                width: 800,
                height: 480,
              });
        if (cancelled) return;
        historyRef.current = new InkHistory(bundle.document);
        setDoc(bundle.document);
        const canvas = canvasRef.current;
        if (canvas) {
          canvas.width = bundle.document.width;
          canvas.height = bundle.document.height;
          rendererRef.current = createCanvasInkRenderer(canvas);
          rendererRef.current.renderDocument(bundle.document);
        }
        syncHistoryFlags();
      } catch (err) {
        toast.error(err instanceof Error ? err.message : t.ink.error);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void boot();
    return () => {
      cancelled = true;
      if (saveTimer.current) clearTimeout(saveTimer.current);
      rendererRef.current?.destroy();
      rendererRef.current = null;
    };
  }, [cardId, t.ink.error]);

  useEffect(() => {
    rendererRef.current?.setViewport({
      scale,
      offsetX: offset.x,
      offsetY: offset.y,
    });
  }, [scale, offset]);

  function pointerPos(e: ReactPointerEvent<HTMLCanvasElement>) {
    const canvas = canvasRef.current!;
    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - offset.x) / scale;
    const y = (e.clientY - rect.top - offset.y) / scale;
    return { x, y, pressure: e.pressure > 0 ? e.pressure : 1 };
  }

  function onPointerDown(e: ReactPointerEvent<HTMLCanvasElement>) {
    if (!doc || e.button === 1 || (e.button === 0 && e.altKey)) {
      panRef.current = { x: e.clientX - offset.x, y: e.clientY - offset.y };
      return;
    }
    const history = historyRef.current;
    if (!history || !doc) return;
    drawingRef.current = true;
    canvasRef.current?.setPointerCapture(e.pointerId);
    const pos = pointerPos(e);
    const stroke = createStroke(
      tool,
      tool === "highlighter" ? "#f59e0b" : color,
      width,
    );
    stroke.points.push({
      x: pos.x,
      y: pos.y,
      pressure: pos.pressure,
      timestamp: 0,
    });
    strokeRef.current = stroke;
  }

  function onPointerMove(e: ReactPointerEvent<HTMLCanvasElement>) {
    if (panRef.current) {
      setOffset({
        x: e.clientX - panRef.current.x,
        y: e.clientY - panRef.current.y,
      });
      return;
    }
    if (!drawingRef.current || !strokeRef.current || !doc) return;
    const pos = pointerPos(e);
    const start = strokeRef.current.points[0]?.timestamp ?? performance.now();
    strokeRef.current.points.push({
      x: pos.x,
      y: pos.y,
      pressure: pos.pressure,
      timestamp: Math.round(performance.now() - start),
    });
    rendererRef.current?.drawStroke(strokeRef.current);
  }

  function onPointerUp() {
    panRef.current = null;
    if (!drawingRef.current || !strokeRef.current || !doc) return;
    drawingRef.current = false;
    const stroke = strokeRef.current;
    strokeRef.current = null;
    if (stroke.points.length < 2) return;
    const layer = activeLayer(doc);
    const next: InkDocument = {
      ...doc,
      layers: doc.layers.map((l) =>
        l.id === layer.id ? { ...l, strokes: [...l.strokes, stroke] } : l,
      ),
    };
    applyDocument(next, true);
  }

  const layer = doc ? activeLayer(doc) : null;

  return (
    <section className={cn("space-y-3", className)} aria-label={t.ink.title}>
      <div>
        <h3 className="text-sm font-medium">{t.ink.title}</h3>
        <p className="text-xs text-muted-foreground">{t.ink.hint}</p>
      </div>

      <div className="flex flex-wrap items-center gap-1">
        <ToolToggle
          pressed={tool === "pen"}
          onPressedChange={() => setTool("pen")}
          label={t.ink.pen}
        >
          <Pen className="size-4" />
        </ToolToggle>
        <ToolToggle
          pressed={tool === "highlighter"}
          onPressedChange={() => setTool("highlighter")}
          label={t.ink.highlighter}
        >
          <Highlighter className="size-4" />
        </ToolToggle>
        <ToolToggle
          pressed={tool === "eraser"}
          onPressedChange={() => setTool("eraser")}
          label={t.ink.eraser}
        >
          <Eraser className="size-4" />
        </ToolToggle>
        <ColorPicker
          value={color}
          onChange={setColor}
          aria-label={t.ink.color}
          labels={{
            title: t.ink.colorPickerTitle,
            hex: t.ink.colorHex,
            alpha: t.ink.colorAlpha,
          }}
        />
        <div className="ms-2 flex min-w-36 items-center gap-2">
          <Label className="shrink-0 text-xs text-muted-foreground">
            {t.ink.width}
          </Label>
          <Slider
            min={1}
            max={16}
            step={0.5}
            value={[width]}
            onValueChange={([next]) => setWidth(next ?? 2.5)}
            aria-label={t.ink.width}
            className="w-28"
          />
          <span className="w-8 text-xs tabular-nums text-muted-foreground">
            {width.toFixed(1)}
          </span>
        </div>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          disabled={!canUndo}
          aria-label={t.ink.undo}
          onClick={() => {
            const prev = historyRef.current?.undo();
            if (!prev) return;
            setDoc(prev);
            rendererRef.current?.renderDocument(prev);
            syncHistoryFlags();
            scheduleSave(prev);
          }}
        >
          <Undo2 className="size-4" />
        </Button>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          disabled={!canRedo}
          aria-label={t.ink.redo}
          onClick={() => {
            const next = historyRef.current?.redo();
            if (!next) return;
            setDoc(next);
            rendererRef.current?.renderDocument(next);
            syncHistoryFlags();
            scheduleSave(next);
          }}
        >
          <Redo2 className="size-4" />
        </Button>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          disabled={!doc || !layer}
          aria-label={layer?.visible ? t.ink.hideLayer : t.ink.showLayer}
          onClick={() => {
            if (!doc || !layer) return;
            applyDocument(toggleLayerVisibility(doc, layer.id), true);
          }}
        >
          {layer?.visible ? (
            <Eye className="size-4" />
          ) : (
            <EyeOff className="size-4" />
          )}
        </Button>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          disabled={!doc || !layer}
          aria-label={t.ink.clearLayer}
          onClick={() => {
            if (!doc || !layer) return;
            applyDocument(clearLayer(doc, layer.id), true);
          }}
        >
          <Trash2 className="size-4" />
        </Button>
        <div className="ms-auto flex min-w-36 items-center gap-2">
          <Label className="shrink-0 text-xs text-muted-foreground">
            {t.ink.zoom}
          </Label>
          <Slider
            min={0.5}
            max={2}
            step={0.1}
            value={[scale]}
            onValueChange={([next]) => setScale(next ?? 1)}
            aria-label={t.ink.zoom}
            className="w-28"
          />
          <span className="w-10 text-xs tabular-nums text-muted-foreground">
            {Math.round(scale * 100)}%
          </span>
        </div>
      </div>

      <div className="relative overflow-hidden rounded-md border border-border bg-background">
        {loading ? (
          <div className="flex h-48 items-center justify-center">
            <Spinner className="size-5" />
          </div>
        ) : null}
        <canvas
          ref={canvasRef}
          className={cn(
            "touch-none max-w-full cursor-crosshair",
            loading && "hidden",
          )}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        />
      </div>
      <p className="text-xs text-muted-foreground">{t.ink.panHint}</p>
    </section>
  );
}

function ToolToggle({
  pressed,
  onPressedChange,
  label,
  children,
}: {
  pressed: boolean;
  onPressedChange: () => void;
  label: string;
  children: ReactNode;
}) {
  return (
    <Toggle
      pressed={pressed}
      onPressedChange={onPressedChange}
      aria-label={label}
      size="sm"
    >
      {children}
    </Toggle>
  );
}
