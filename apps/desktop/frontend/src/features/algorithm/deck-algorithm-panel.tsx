"use client";

import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { Deck, DeckSettings } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";
import { AlgorithmEditor } from "./algorithm-editor";
import {
  draftFromDeck,
  intervalsLabel,
  normalizeIntervals,
  PRESET_INTERVALS,
  toDeckPayload,
  type AlgorithmDraft,
} from "./leitner";

type Props = {
  deck: Deck;
  onUpdated: (deck: Deck) => void;
};

export function DeckAlgorithmPanel({ deck, onUpdated }: Props) {
  return (
    <DeckAlgorithmPanelInner
      key={`${deck.id}:${deck.revision}`}
      deck={deck}
      onUpdated={onUpdated}
    />
  );
}

function DeckAlgorithmPanelInner({ deck, onUpdated }: Props) {
  const t = useTranslations();
  const initial = draftFromDeck(deck.settings?.algorithm);
  const [inherit, setInherit] = useState(initial.inherit);
  const [draft, setDraft] = useState<AlgorithmDraft>(() => ({
    mode: initial.mode,
    presetId: initial.presetId,
    intervalsDays: normalizeIntervals(initial.intervalsDays, initial.presetId),
  }));
  const [profileSummary, setProfileSummary] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void api
      .getProfileAlgorithm()
      .then((profile) => {
        if (cancelled) return;
        const intervals =
          profile.mode === "custom"
            ? normalizeIntervals(profile.intervalsDays, profile.presetId)
            : PRESET_INTERVALS[profile.presetId];
        const label =
          profile.mode === "custom"
            ? t.algorithm.modeCustom
            : t.algorithm.presets[profile.presetId].name;
        setProfileSummary(
          `${label} — ${intervalsLabel(intervals, t.algorithm.dayUnit)}`,
        );
      })
      .catch(() => {
        if (!cancelled) setProfileSummary("");
      });
    return () => {
      cancelled = true;
    };
  }, [t.algorithm.dayUnit, t.algorithm.modeCustom, t.algorithm.presets]);

  async function handleSave() {
    setBusy(true);
    try {
      const algorithm = toDeckPayload({
        inherit,
        mode: draft.mode,
        presetId: draft.presetId,
        intervalsDays: draft.intervalsDays,
      });
      const settings: DeckSettings = {
        ...deck.settings,
        algorithm,
      };
      const updated = await api.updateDeckSettings(deck.id, settings);
      onUpdated(updated);
      toast.success(t.algorithm.saved);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.algorithm.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="space-y-4 rounded-lg border border-border/60 px-4 py-4">
      <div className="space-y-1">
        <h3 className="text-base font-semibold tracking-tight">
          {t.algorithm.deckTitle}
        </h3>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.algorithm.deckDescription}
        </p>
      </div>

      <div className="flex items-start gap-3">
        <Checkbox
          id={`deck-inherit-${deck.id}`}
          className="mt-1"
          checked={inherit}
          disabled={busy}
          onCheckedChange={(checked) => setInherit(checked === true)}
        />
        <div className="space-y-0.5">
          <Label
            htmlFor={`deck-inherit-${deck.id}`}
            className="text-sm font-medium"
          >
            {t.algorithm.inheritProfile}
          </Label>
          <p className="text-xs text-muted-foreground text-pretty">
            {profileSummary
              ? t.algorithm.inheritHint.replace("{summary}", profileSummary)
              : t.algorithm.inheritHintFallback}
          </p>
        </div>
      </div>

      {!inherit ? (
        <AlgorithmEditor
          idPrefix={`deck-algorithm-${deck.id}`}
          value={draft}
          onChange={setDraft}
          disabled={busy}
        />
      ) : null}

      <Button type="button" onClick={() => void handleSave()} disabled={busy}>
        {busy ? <Spinner /> : null}
        {t.algorithm.saveDeck}
      </Button>
    </section>
  );
}
