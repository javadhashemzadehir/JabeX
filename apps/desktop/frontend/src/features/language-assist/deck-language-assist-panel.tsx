"use client";

import { useState } from "react";
import { toast } from "sonner";
import type { AssistToggle, Deck, DeckSettings, LanguageAssistOverrides } from "@jabex/types";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { api } from "@/lib/tauri";

const DEFAULT_OVERRIDES: LanguageAssistOverrides = {
  translation: "inherit",
  pronunciation: "inherit",
};

type Props = {
  deck: Deck;
  onUpdated: (deck: Deck) => void;
};

export function DeckLanguageAssistPanel({ deck, onUpdated }: Props) {
  return (
    <DeckLanguageAssistPanelInner
      key={`${deck.id}:${deck.revision}:lang`}
      deck={deck}
      onUpdated={onUpdated}
    />
  );
}

function DeckLanguageAssistPanelInner({ deck, onUpdated }: Props) {
  const t = useTranslations();
  const initial = deck.settings?.languageAssist ?? DEFAULT_OVERRIDES;
  const [overrides, setOverrides] = useState<LanguageAssistOverrides>({
    translation: initial.translation ?? "inherit",
    pronunciation: initial.pronunciation ?? "inherit",
  });
  const [busy, setBusy] = useState(false);

  async function persist(next: LanguageAssistOverrides) {
    setBusy(true);
    setOverrides(next);
    try {
      const settings: DeckSettings = {
        ...deck.settings,
        languageAssist: next,
      };
      const updated = await api.updateDeckSettings(deck.id, settings);
      onUpdated(updated);
      toast.success(t.languageAssist.deckSaved);
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : t.languageAssist.deckSaveError,
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="space-y-3 rounded-lg border border-border/60 p-4">
      <div className="space-y-1">
        <h3 className="text-sm font-semibold tracking-tight">
          {t.languageAssist.deckTitle}
        </h3>
        <p className="text-xs text-muted-foreground text-pretty">
          {t.languageAssist.deckHint}
        </p>
      </div>

      <ToggleRow
        id="deck-lang-translation"
        label={t.languageAssist.autoTranslation}
        hint={t.languageAssist.deckTranslationHint}
        value={overrides.translation}
        disabled={busy}
        onChange={(translation) => void persist({ ...overrides, translation })}
        labels={t.languageAssist.toggles}
      />

      <ToggleRow
        id="deck-lang-pronunciation"
        label={t.languageAssist.pronunciation}
        hint={t.languageAssist.deckPronunciationHint}
        value={overrides.pronunciation}
        disabled={busy}
        onChange={(pronunciation) =>
          void persist({ ...overrides, pronunciation })
        }
        labels={t.languageAssist.toggles}
      />

      {busy ? (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Spinner className="size-3.5" />
          {t.settings.loading}
        </div>
      ) : null}
    </section>
  );
}

function ToggleRow({
  id,
  label,
  hint,
  value,
  disabled,
  onChange,
  labels,
}: {
  id: string;
  label: string;
  hint: string;
  value: AssistToggle;
  disabled: boolean;
  onChange: (value: AssistToggle) => void;
  labels: { inherit: string; enabled: string; disabled: string };
}) {
  return (
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0 flex-1 space-y-0.5">
        <Label htmlFor={id}>{label}</Label>
        <p className="text-xs text-muted-foreground text-pretty">{hint}</p>
      </div>
      <select
        id={id}
        className="h-9 rounded-md border border-input bg-background px-2 text-sm"
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value as AssistToggle)}
      >
        <option value="inherit">{labels.inherit}</option>
        <option value="enabled">{labels.enabled}</option>
        <option value="disabled">{labels.disabled}</option>
      </select>
    </div>
  );
}
