"use client";

import type { AlgorithmMode, LeitnerPresetId } from "@jabex/types";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { useTranslations } from "@/hooks/use-locale";
import {
  intervalsLabel,
  LEITNER_BOX_COUNT,
  PRESET_IDS,
  PRESET_INTERVALS,
  type AlgorithmDraft,
} from "./leitner";

type Props = {
  value: AlgorithmDraft;
  onChange: (next: AlgorithmDraft) => void;
  disabled?: boolean;
  idPrefix: string;
};

export function AlgorithmEditor({
  value,
  onChange,
  disabled = false,
  idPrefix,
}: Props) {
  const t = useTranslations();

  function setMode(mode: AlgorithmMode) {
    if (mode === "preset") {
      onChange({
        ...value,
        mode,
        intervalsDays: [...PRESET_INTERVALS[value.presetId]],
      });
      return;
    }
    onChange({
      ...value,
      mode,
      intervalsDays: normalizeFromCurrent(value),
    });
  }

  function setPreset(presetId: LeitnerPresetId) {
    onChange({
      ...value,
      mode: "preset",
      presetId,
      intervalsDays: [...PRESET_INTERVALS[presetId]],
    });
  }

  function setIntervalAt(index: number, raw: string) {
    const next = [...value.intervalsDays] as AlgorithmDraft["intervalsDays"];
    const parsed = Number(raw);
    next[index] = Number.isFinite(parsed) ? parsed : next[index];
    onChange({ ...value, mode: "custom", intervalsDays: next });
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>{t.algorithm.modeLabel}</Label>
        <RadioGroup
          value={value.mode}
          onValueChange={(next) => setMode(next as AlgorithmMode)}
          disabled={disabled}
          className="flex flex-wrap gap-4"
        >
          <div className="flex items-center gap-2">
            <RadioGroupItem value="preset" id={`${idPrefix}-mode-preset`} />
            <Label htmlFor={`${idPrefix}-mode-preset`} className="font-normal">
              {t.algorithm.modePreset}
            </Label>
          </div>
          <div className="flex items-center gap-2">
            <RadioGroupItem value="custom" id={`${idPrefix}-mode-custom`} />
            <Label htmlFor={`${idPrefix}-mode-custom`} className="font-normal">
              {t.algorithm.modeCustom}
            </Label>
          </div>
        </RadioGroup>
      </div>

      {value.mode === "preset" ? (
        <div className="space-y-2">
          <Label>{t.algorithm.presetLabel}</Label>
          <RadioGroup
            value={value.presetId}
            onValueChange={(next) => setPreset(next as LeitnerPresetId)}
            disabled={disabled}
            className="gap-2"
          >
            {PRESET_IDS.map((id) => {
              const intervals = PRESET_INTERVALS[id];
              const itemId = `${idPrefix}-preset-${id}`;
              return (
                <Label
                  key={id}
                  htmlFor={itemId}
                  className="flex cursor-pointer items-start gap-3 rounded-md border border-border/60 px-3 py-2 font-normal has-data-[state=checked]:border-ring"
                >
                  <RadioGroupItem value={id} id={itemId} className="mt-1" />
                  <span className="min-w-0 space-y-0.5">
                    <span className="block text-sm font-medium">
                      {t.algorithm.presets[id].name}
                    </span>
                    <span className="block text-xs text-muted-foreground text-pretty">
                      {t.algorithm.presets[id].hint}
                    </span>
                    <span className="block text-xs text-muted-foreground">
                      {intervalsLabel(intervals, t.algorithm.dayUnit)}
                    </span>
                  </span>
                </Label>
              );
            })}
          </RadioGroup>
        </div>
      ) : (
        <div className="space-y-3">
          <div className="space-y-1">
            <Label>{t.algorithm.customIntervals}</Label>
            <p className="text-xs text-muted-foreground text-pretty">
              {t.algorithm.customHint}
            </p>
          </div>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
            {Array.from({ length: LEITNER_BOX_COUNT }, (_, index) => {
              const box = index + 1;
              const inputId = `${idPrefix}-box-${box}`;
              return (
                <div key={box} className="space-y-1.5">
                  <Label htmlFor={inputId}>
                    {t.algorithm.boxDays.replace("{box}", String(box))}
                  </Label>
                  <Input
                    id={inputId}
                    type="number"
                    min={1}
                    max={3650}
                    step={1}
                    disabled={disabled}
                    value={value.intervalsDays[index]}
                    onChange={(event) =>
                      setIntervalAt(index, event.target.value)
                    }
                  />
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

function normalizeFromCurrent(
  value: AlgorithmDraft,
): AlgorithmDraft["intervalsDays"] {
  if (value.mode === "preset") {
    return [...PRESET_INTERVALS[value.presetId]];
  }
  return [...value.intervalsDays];
}
