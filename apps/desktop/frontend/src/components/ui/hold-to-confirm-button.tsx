"use client";

import { useEffect, useRef, useState } from "react";
import { Button, type ButtonProps } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const DEFAULT_HOLD_MS = 1600;

export interface HoldToConfirmButtonProps
  extends Omit<
    ButtonProps,
    "onClick" | "onPointerDown" | "onPointerUp" | "onKeyDown" | "onKeyUp"
  > {
  holdMs?: number;
  holdLabel: string;
  confirmingLabel: string;
  onConfirm: () => void;
}

/**
 * Destructive confirmation that requires a sustained press (or Space/Enter hold).
 * Releases cancel the gesture before completion.
 */
export function HoldToConfirmButton({
  holdMs = DEFAULT_HOLD_MS,
  holdLabel,
  confirmingLabel,
  onConfirm,
  disabled,
  className,
  variant = "destructive",
  children,
  ...props
}: HoldToConfirmButtonProps) {
  const [progress, setProgress] = useState(0);
  const [holding, setHolding] = useState(false);
  const rafRef = useRef<number | null>(null);
  const startRef = useRef<number | null>(null);
  const confirmedRef = useRef(false);
  const onConfirmRef = useRef(onConfirm);
  const holdMsRef = useRef(holdMs);
  const disabledRef = useRef(!!disabled);

  useEffect(() => {
    onConfirmRef.current = onConfirm;
    holdMsRef.current = holdMs;
    disabledRef.current = !!disabled;
  }, [onConfirm, holdMs, disabled]);

  function clearRaf() {
    if (rafRef.current != null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
  }

  function stopHolding() {
    clearRaf();
    startRef.current = null;
    setHolding(false);
    if (!confirmedRef.current) {
      setProgress(0);
    }
  }

  function startHolding() {
    if (disabledRef.current || confirmedRef.current) return;
    confirmedRef.current = false;
    setHolding(true);
    setProgress(0);
    startRef.current = performance.now();

    const step = (now: number) => {
      if (startRef.current == null) return;
      if (disabledRef.current) {
        stopHolding();
        return;
      }
      const next = Math.min(1, (now - startRef.current) / holdMsRef.current);
      setProgress(next);
      if (next >= 1) {
        confirmedRef.current = true;
        clearRaf();
        startRef.current = null;
        setHolding(false);
        setProgress(1);
        onConfirmRef.current();
        return;
      }
      rafRef.current = requestAnimationFrame(step);
    };

    rafRef.current = requestAnimationFrame(step);
  }

  useEffect(() => {
    return () => clearRaf();
  }, []);

  return (
    <Button
      type="button"
      variant={variant}
      disabled={disabled}
      aria-label={holding ? confirmingLabel : holdLabel}
      className={cn("relative overflow-hidden select-none", className)}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        startHolding();
      }}
      onPointerUp={() => stopHolding()}
      onPointerCancel={() => stopHolding()}
      onLostPointerCapture={() => stopHolding()}
      onKeyDown={(event) => {
        if (event.key === " " || event.key === "Enter") {
          event.preventDefault();
          if (!holding) startHolding();
        }
      }}
      onKeyUp={(event) => {
        if (event.key === " " || event.key === "Enter") {
          event.preventDefault();
          stopHolding();
        }
      }}
      onContextMenu={(event) => event.preventDefault()}
      {...props}
    >
      <span
        aria-hidden
        className="pointer-events-none absolute inset-y-0 inset-s-0 bg-black/20 transition-[width] duration-75 dark:bg-white/20"
        style={{ width: `${progress * 100}%` }}
      />
      <span className="relative z-10">
        {holding || progress > 0 ? confirmingLabel : (children ?? holdLabel)}
      </span>
    </Button>
  );
}
