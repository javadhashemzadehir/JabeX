import type {
  AlgorithmMode,
  DeckAlgorithmSettings,
  LeitnerPresetId,
  ProfileLeitnerSettings,
} from "@jabex/types";

export const LEITNER_BOX_COUNT = 5;

export const PRESET_INTERVALS: Record<
  LeitnerPresetId,
  [number, number, number, number, number]
> = {
  recommended: [1, 3, 7, 14, 30],
  intensive: [1, 2, 4, 7, 14],
  relaxed: [2, 5, 12, 25, 60],
  longTerm: [1, 4, 10, 30, 90],
};

export const PRESET_IDS: LeitnerPresetId[] = [
  "recommended",
  "intensive",
  "relaxed",
  "longTerm",
];

export function defaultProfileAlgorithm(): ProfileLeitnerSettings {
  return {
    mode: "preset",
    presetId: "recommended",
    intervalsDays: [...PRESET_INTERVALS.recommended],
  };
}

export function defaultDeckAlgorithm(): DeckAlgorithmSettings {
  return {
    inherit: true,
    mode: "preset",
    presetId: "recommended",
    intervalsDays: [...PRESET_INTERVALS.recommended],
  };
}

export function normalizeIntervals(
  values: Array<number | string | null | undefined> | null | undefined,
  fallback: LeitnerPresetId = "recommended",
): [number, number, number, number, number] {
  const base = PRESET_INTERVALS[fallback];
  const out = [...base] as [number, number, number, number, number];
  if (!values) return out;
  for (let i = 0; i < LEITNER_BOX_COUNT; i += 1) {
    const raw = values[i];
    const n = typeof raw === "string" ? Number(raw) : raw;
    if (typeof n === "number" && Number.isFinite(n) && n >= 1) {
      out[i] = Math.min(3650, Math.floor(n));
    }
  }
  return out;
}

export function intervalsLabel(
  intervals: readonly number[],
  dayUnit: string,
): string {
  return intervals.map((d) => `${d}${dayUnit}`).join(" · ");
}

export function draftFromProfile(
  settings: ProfileLeitnerSettings | null | undefined,
): ProfileLeitnerSettings {
  const base = defaultProfileAlgorithm();
  if (!settings) return base;
  return {
    mode: settings.mode ?? "preset",
    presetId: settings.presetId ?? "recommended",
    intervalsDays: normalizeIntervals(
      settings.intervalsDays,
      settings.presetId ?? "recommended",
    ),
  };
}

export function draftFromDeck(
  settings: DeckAlgorithmSettings | null | undefined,
): DeckAlgorithmSettings {
  const base = defaultDeckAlgorithm();
  if (!settings) return base;
  return {
    inherit: settings.inherit ?? true,
    mode: settings.mode ?? "preset",
    presetId: settings.presetId ?? "recommended",
    intervalsDays: normalizeIntervals(
      settings.intervalsDays,
      settings.presetId ?? "recommended",
    ),
  };
}

export function toProfilePayload(
  draft: ProfileLeitnerSettings,
): ProfileLeitnerSettings {
  if (draft.mode === "custom") {
    return {
      mode: "custom",
      presetId: draft.presetId,
      intervalsDays: normalizeIntervals(draft.intervalsDays, draft.presetId),
    };
  }
  return {
    mode: "preset",
    presetId: draft.presetId,
    intervalsDays: null,
  };
}

export function toDeckPayload(draft: DeckAlgorithmSettings): DeckAlgorithmSettings {
  if (draft.inherit) {
    return {
      inherit: true,
      mode: "preset",
      presetId: "recommended",
      intervalsDays: null,
    };
  }
  if (draft.mode === "custom") {
    return {
      inherit: false,
      mode: "custom",
      presetId: draft.presetId,
      intervalsDays: normalizeIntervals(draft.intervalsDays, draft.presetId),
    };
  }
  return {
    inherit: false,
    mode: "preset",
    presetId: draft.presetId,
    intervalsDays: null,
  };
}

export type AlgorithmDraft = {
  mode: AlgorithmMode;
  presetId: LeitnerPresetId;
  intervalsDays: [number, number, number, number, number];
};
