"use client";

import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "./profile-store";

export function InitialProfileSetupDialog() {
  const t = useTranslations();
  const { desktop, initialSetupPending, loading, completeInitialSetup } =
    useProfileStore();
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);

  const open = Boolean(desktop && initialSetupPending && !loading);

  async function handleSubmit() {
    const trimmed = name.trim();
    if (!trimmed) return;
    setBusy(true);
    try {
      await completeInitialSetup(trimmed);
      toast.success(t.profiles.setupSaved);
      setName("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.profiles.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={() => {
        // Required on first launch — ignore dismiss attempts.
      }}
    >
      <DialogContent
        showCloseButton={false}
        onEscapeKeyDown={(event) => event.preventDefault()}
        onPointerDownOutside={(event) => event.preventDefault()}
        onInteractOutside={(event) => event.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>{t.profiles.setupTitle}</DialogTitle>
          <DialogDescription>{t.profiles.setupHint}</DialogDescription>
        </DialogHeader>
        <div className="space-y-2">
          <Label htmlFor="initial-profile-name">{t.profiles.displayName}</Label>
          <Input
            id="initial-profile-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder={t.profiles.displayNamePlaceholder}
            autoFocus
            disabled={busy}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void handleSubmit();
              }
            }}
          />
        </div>
        <DialogFooter>
          <Button
            type="button"
            onClick={() => void handleSubmit()}
            disabled={busy || !name.trim()}
          >
            {busy ? <Spinner /> : null}
            {t.profiles.setupContinue}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
