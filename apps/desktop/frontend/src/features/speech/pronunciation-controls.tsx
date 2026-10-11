"use client";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { useTranslations } from "@/hooks/use-locale";
import type { PlaybackSpeed } from "./audio-player";

const SPEEDS: PlaybackSpeed[] = [0.5, 0.75, 1, 1.25];

type PronunciationControlsProps = {
  speed: PlaybackSpeed;
  onSpeedChange: (speed: PlaybackSpeed) => void;
  repeat: boolean;
  onRepeatChange: (value: boolean) => void;
  shadowing: boolean;
  onShadowingChange: (value: boolean) => void;
  disabled?: boolean;
};

export function PronunciationControls({
  speed,
  onSpeedChange,
  repeat,
  onRepeatChange,
  shadowing,
  onShadowingChange,
  disabled,
}: PronunciationControlsProps) {
  const t = useTranslations();

  return (
    <div className="flex flex-wrap items-end gap-3">
      <div className="space-y-1">
        <Label htmlFor="speech-speed" className="text-xs">
          {t.speech.speed}
        </Label>
        <select
          id="speech-speed"
          className="h-9 rounded-md border border-input bg-background px-2 text-sm"
          value={String(speed)}
          disabled={disabled}
          onChange={(e) => onSpeedChange(Number(e.target.value) as PlaybackSpeed)}
        >
          {SPEEDS.map((s) => (
            <option key={s} value={s}>
              {s}x
            </option>
          ))}
        </select>
      </div>
      <Button
        type="button"
        size="sm"
        variant={repeat ? "default" : "outline"}
        disabled={disabled}
        aria-pressed={repeat}
        onClick={() => onRepeatChange(!repeat)}
      >
        {t.speech.repeat}
      </Button>
      <Button
        type="button"
        size="sm"
        variant={shadowing ? "default" : "outline"}
        disabled={disabled}
        aria-pressed={shadowing}
        onClick={() => onShadowingChange(!shadowing)}
        title={t.speech.shadowingHint}
      >
        {t.speech.shadowing}
      </Button>
    </div>
  );
}
