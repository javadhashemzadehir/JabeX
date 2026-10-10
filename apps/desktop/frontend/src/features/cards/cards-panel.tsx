"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowLeft, Pencil, Plus, SquareStack, Trash2 } from "lucide-react";
import { motion } from "framer-motion";
import { toast } from "sonner";
import type { Card } from "@jabex/types";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useCardStore } from "./card-store";

type DialogMode = "create" | "edit" | "delete" | null;

export function CardsPanel({ deckId }: { deckId: string }) {
  const t = useTranslations();
  const tauri = useIsTauri();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const {
    deck,
    cards,
    loading,
    error,
    desktop,
    hydrate,
    create,
    update,
    remove,
    reset,
  } = useCardStore();

  const [dialog, setDialog] = useState<DialogMode>(null);
  const [target, setTarget] = useState<Card | null>(null);
  const [front, setFront] = useState("");
  const [back, setBack] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void hydrate(deckId);
    return () => reset();
  }, [hydrate, reset, deckId, activeProfileId]);

  function openCreate() {
    setTarget(null);
    setFront("");
    setBack("");
    setDialog("create");
  }

  function openEdit(card: Card) {
    setTarget(card);
    setFront(card.frontContent);
    setBack(card.backContent);
    setDialog("edit");
  }

  function openDelete(card: Card) {
    setTarget(card);
    setDialog("delete");
  }

  function closeDialog() {
    if (busy) return;
    setDialog(null);
    setTarget(null);
  }

  async function handleCreate() {
    const frontTrimmed = front.trim();
    const backTrimmed = back.trim();
    if (!frontTrimmed || !backTrimmed) return;
    setBusy(true);
    try {
      const card = await create(frontTrimmed, backTrimmed);
      if (card) {
        toast.success(t.cards.created);
        setDialog(null);
        setFront("");
        setBack("");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.cards.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleEdit() {
    if (!target) return;
    const frontTrimmed = front.trim();
    const backTrimmed = back.trim();
    if (!frontTrimmed || !backTrimmed) return;
    setBusy(true);
    try {
      await update(target.id, frontTrimmed, backTrimmed);
      toast.success(t.cards.updated);
      setDialog(null);
      setTarget(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.cards.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleDelete() {
    if (!target) return;
    setBusy(true);
    try {
      await remove(target.id);
      toast.success(t.cards.deleted);
      setDialog(null);
      setTarget(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.cards.error);
    } finally {
      setBusy(false);
    }
  }

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.cards.desktopOnly}</p>
    );
  }

  if (!loading && (error || !deck)) {
    return (
      <div className="space-y-4">
        <Button variant="ghost" size="sm" asChild>
          <Link href="/decks">
            <ArrowLeft className="size-4 rtl:rotate-180" />
            {t.cards.backToDecks}
          </Link>
        </Button>
        <p className="text-sm text-muted-foreground">
          {error ?? t.cards.notFound}
        </p>
      </div>
    );
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-6"
    >
      <div className="space-y-3">
        <Button variant="ghost" size="sm" className="-ms-2 w-fit" asChild>
          <Link href="/decks">
            <ArrowLeft className="size-4 rtl:rotate-180" />
            {t.cards.backToDecks}
          </Link>
        </Button>
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div className="space-y-1">
            <h2 className="text-2xl font-semibold tracking-tight">
              {deck?.name ?? t.cards.loading}
            </h2>
            <p className="text-sm text-muted-foreground text-pretty">
              {t.cards.description}
            </p>
          </div>
          <Button onClick={openCreate} disabled={!deck}>
            <Plus className="size-4" />
            {t.cards.create}
          </Button>
        </div>
      </div>

      {loading && cards.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t.cards.loading}</p>
      ) : null}

      {!loading && cards.length === 0 ? (
        <div className="flex flex-col items-start gap-4 rounded-lg border border-dashed border-border/80 bg-muted/20 px-6 py-10">
          <div className="flex size-10 items-center justify-center rounded-md bg-muted">
            <SquareStack className="size-5 text-muted-foreground" />
          </div>
          <div className="space-y-1">
            <p className="font-medium">{t.cards.emptyTitle}</p>
            <p className="max-w-md text-sm text-muted-foreground text-pretty">
              {t.cards.emptyHint}
            </p>
          </div>
          <Button onClick={openCreate}>
            <Plus className="size-4" />
            {t.cards.create}
          </Button>
        </div>
      ) : (
        <ul className="divide-y divide-border rounded-lg border border-border/60">
          {cards.map((card, index) => (
            <motion.li
              key={card.id}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.03 * index, duration: 0.25 }}
              className="flex flex-wrap items-start justify-between gap-3 px-4 py-3"
            >
              <div className="min-w-0 flex-1 space-y-1">
                <p className="line-clamp-2 font-medium">{card.frontContent}</p>
                <p className="line-clamp-2 text-sm text-muted-foreground">
                  {card.backContent}
                </p>
                <p className="text-xs text-muted-foreground">
                  {t.cards.boxLabel.replace(
                    "{box}",
                    String(card.boxPosition),
                  )}
                </p>
              </div>
              <div className="flex shrink-0 gap-1">
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t.cards.edit}
                  onClick={() => openEdit(card)}
                >
                  <Pencil className="size-4" />
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t.cards.delete}
                  onClick={() => openDelete(card)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            </motion.li>
          ))}
        </ul>
      )}

      <Dialog
        open={dialog === "create" || dialog === "edit"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {dialog === "edit" ? t.cards.editTitle : t.cards.createTitle}
            </DialogTitle>
            <DialogDescription>
              {dialog === "edit" ? t.cards.editHint : t.cards.createHint}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="card-front">{t.cards.front}</Label>
              <Textarea
                id="card-front"
                value={front}
                onChange={(event) => setFront(event.target.value)}
                placeholder={t.cards.frontPlaceholder}
                autoFocus
                rows={3}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="card-back">{t.cards.back}</Label>
              <Textarea
                id="card-back"
                value={back}
                onChange={(event) => setBack(event.target.value)}
                placeholder={t.cards.backPlaceholder}
                rows={3}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={closeDialog} disabled={busy}>
              {t.cards.cancel}
            </Button>
            <Button
              onClick={() =>
                void (dialog === "edit" ? handleEdit() : handleCreate())
              }
              disabled={busy || !front.trim() || !back.trim()}
            >
              {dialog === "edit" ? t.cards.save : t.cards.create}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={dialog === "delete"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t.cards.deleteTitle}</DialogTitle>
            <DialogDescription>
              {t.cards.deleteHint.replace(
                "{front}",
                target?.frontContent ?? "",
              )}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={closeDialog} disabled={busy}>
              {t.cards.cancel}
            </Button>
            <Button
              variant="destructive"
              onClick={() => void handleDelete()}
              disabled={busy}
            >
              {t.cards.delete}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </motion.section>
  );
}
