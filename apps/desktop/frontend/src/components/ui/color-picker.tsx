"use client";

import * as React from "react";
import { Pipette } from "lucide-react";
import { RgbaColorPicker, type RgbaColor } from "react-colorful";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Slider } from "@/components/ui/slider";
import { cn } from "@/lib/utils";

type ColorPickerProps = {
  value: string;
  onChange: (value: string) => void;
  /** Optional trigger; defaults to a swatch button. */
  children?: React.ReactNode;
  className?: string;
  disabled?: boolean;
  /** Accessible label for the trigger. */
  "aria-label"?: string;
  labels?: {
    title?: string;
    hex?: string;
    alpha?: string;
  };
};

function clampByte(n: number) {
  return Math.max(0, Math.min(255, Math.round(n)));
}

function toHex(n: number) {
  return clampByte(n).toString(16).padStart(2, "0");
}

export function rgbaToHex(r: number, g: number, b: number, a = 1) {
  const alpha = Number.isFinite(a) ? Math.round(a * 255) : 255;
  const base = `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  return alpha === 255 ? base : `${base}${toHex(alpha)}`;
}

export function hexToRgba(hex: string): RgbaColor | null {
  if (!hex) return null;
  let value = hex.replace(/^#/, "");
  if (value.length === 3) {
    value = value
      .split("")
      .map((char) => char + char)
      .join("");
  }
  if (value.length !== 6 && value.length !== 8) return null;
  const r = Number.parseInt(value.slice(0, 2), 16);
  const g = Number.parseInt(value.slice(2, 4), 16);
  const b = Number.parseInt(value.slice(4, 6), 16);
  const a =
    value.length === 8 ? Number.parseInt(value.slice(6, 8), 16) / 255 : 1;
  if ([r, g, b, a].some((n) => Number.isNaN(n))) return null;
  return { r, g, b, a };
}

/**
 * Popover color picker (RGBA + HEX), adapted from the shadcn color-picker
 * pattern used by [Fnz11/shadcn-color-picker](https://github.com/Fnz11/shadcn-color-picker).
 */
export function ColorPicker({
  value,
  onChange,
  children,
  className,
  disabled,
  "aria-label": ariaLabel,
  labels,
}: ColorPickerProps) {
  const rgba = hexToRgba(value) ?? { r: 17, g: 24, b: 39, a: 1 };
  const solidHex = rgbaToHex(rgba.r, rgba.g, rgba.b, 1);

  const handlePickerChange = React.useCallback(
    (next: RgbaColor) => {
      onChange(rgbaToHex(next.r, next.g, next.b, next.a));
    },
    [onChange],
  );

  return (
    <Popover>
      <PopoverTrigger asChild disabled={disabled}>
        {children ?? (
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={disabled}
            aria-label={ariaLabel}
            className={cn("ms-2 size-8 shrink-0 overflow-hidden p-0", className)}
          >
            <span
              className="size-full"
              style={{
                backgroundColor: value || "#111827",
              }}
            />
          </Button>
        )}
      </PopoverTrigger>
      <PopoverContent className="w-auto space-y-3 p-3" align="start">
        <div className="flex items-center gap-2 text-sm font-medium">
          <Pipette className="size-4 text-muted-foreground" aria-hidden />
          <span>{labels?.title ?? "Color"}</span>
        </div>

        <RgbaColorPicker
          color={rgba}
          onChange={handlePickerChange}
          className="w-56!"
        />

        <div className="grid gap-2">
          <div className="grid gap-1.5">
            <Label htmlFor="color-picker-hex">{labels?.hex ?? "HEX"}</Label>
            <div className="flex items-center gap-2">
              <span
                className="size-8 shrink-0 rounded-md border border-border"
                style={{ backgroundColor: value }}
                aria-hidden
              />
              <Input
                id="color-picker-hex"
                value={solidHex.toUpperCase()}
                onChange={(e) => {
                  const next = e.target.value.trim();
                  const parsed = hexToRgba(next);
                  if (!parsed) return;
                  onChange(rgbaToHex(parsed.r, parsed.g, parsed.b, rgba.a));
                }}
                spellCheck={false}
                className="font-mono uppercase"
              />
            </div>
          </div>

          <div className="grid gap-1.5">
            <div className="flex items-center justify-between gap-2">
              <Label htmlFor="color-picker-alpha">
                {labels?.alpha ?? "Alpha"}
              </Label>
              <span className="text-xs tabular-nums text-muted-foreground">
                {Math.round(rgba.a * 100)}%
              </span>
            </div>
            <Slider
              id="color-picker-alpha"
              min={0}
              max={1}
              step={0.01}
              value={[rgba.a]}
              onValueChange={([alpha]) => {
                onChange(rgbaToHex(rgba.r, rgba.g, rgba.b, alpha ?? 1));
              }}
              aria-label={labels?.alpha ?? "Alpha"}
            />
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
