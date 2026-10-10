"use client";

import { useEffect, useState } from "react";
import { toast } from "sonner";
import { FormSkeleton } from "@/components/feedback/form-skeleton";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { api } from "@/lib/tauri";
import { AlgorithmEditor } from "./algorithm-editor";
import {
  draftFromProfile,
  normalizeIntervals,
  toProfilePayload,
  type AlgorithmDraft,
} from "./leitner";

export function ProfileAlgorithmPanel() {
  const t = useTranslations();
  const tauri = useIsTauri();
  const activeProfileId = useProfileStore((s) => s.active?.id);

  if (!tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.algorithm.desktopOnly}</p>
    );
  }

  return (
    <ProfileAlgorithmPanelInner
      key={activeProfileId ?? "none"}
      errorLabel={t.algorithm.error}
    />
  );
}

function ProfileAlgorithmPanelInner({ errorLabel }: { errorLabel: string }) {
  const t = useTranslations();
  const [draft, setDraft] = useState<AlgorithmDraft>(() => {
    const profile = draftFromProfile(null);
    return {
      mode: profile.mode,
      presetId: profile.presetId,
      intervalsDays: normalizeIntervals(profile.intervalsDays, profile.presetId),
    };
  });
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void api
      .getProfileAlgorithm()
      .then((settings) => {
        if (cancelled) return;
        const next = draftFromProfile(settings);
        setDraft({
          mode: next.mode,
          presetId: next.presetId,
          intervalsDays: normalizeIntervals(next.intervalsDays, next.presetId),
        });
      })
      .catch((err) => {
        if (!cancelled) {
          toast.error(err instanceof Error ? err.message : errorLabel);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [errorLabel]);

  async function handleSave() {
    setBusy(true);
    try {
      await api.setProfileAlgorithm(
        toProfilePayload({
          mode: draft.mode,
          presetId: draft.presetId,
          intervalsDays: draft.intervalsDays,
        }),
      );
      toast.success(t.algorithm.saved);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.algorithm.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleApplyAll(inherit: boolean) {
    setBusy(true);
    try {
      await api.setProfileAlgorithm(
        toProfilePayload({
          mode: draft.mode,
          presetId: draft.presetId,
          intervalsDays: draft.intervalsDays,
        }),
      );
      const count = await api.applyProfileAlgorithmToAllDecks(inherit);
      toast.success(
        (inherit
          ? t.algorithm.appliedInherit
          : t.algorithm.appliedCopy
        ).replace("{count}", String(count)),
      );
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.algorithm.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="space-y-4">
      <div className="space-y-1">
        <h3 className="text-lg font-semibold tracking-tight">
          {t.algorithm.profileTitle}
        </h3>
        <p className="text-sm text-muted-foreground text-pretty">
          {t.algorithm.profileDescription}
        </p>
      </div>

      {loading ? (
        <FormSkeleton fields={4} />
      ) : (
        <>
          <AlgorithmEditor
            idPrefix="profile-algorithm"
            value={draft}
            onChange={setDraft}
            disabled={busy}
          />
          <div className="flex flex-wrap gap-2">
            <Button type="button" onClick={() => void handleSave()} disabled={busy}>
              {busy ? <Spinner /> : null}
              {t.algorithm.saveProfile}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => void handleApplyAll(true)}
              disabled={busy}
            >
              {t.algorithm.applyInheritAll}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => void handleApplyAll(false)}
              disabled={busy}
            >
              {t.algorithm.applyCopyAll}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground text-pretty">
            {t.algorithm.applyAllHint}
          </p>
        </>
      )}
    </section>
  );
}
