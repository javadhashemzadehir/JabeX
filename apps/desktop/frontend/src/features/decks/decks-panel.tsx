"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ArrowRight, Layers, Pencil, Plus, Trash2 } from "lucide-react";
import { motion } from "framer-motion";
import { toast } from "sonner";
import type { Deck } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { ListSkeleton } from "@/components/feedback/list-skeleton";
import { ConfirmDeleteDialog } from "@/components/ui/confirm-delete-dialog";
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
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import { useDeckStore } from "./deck-store";

type DialogMode = "create" | "rename" | "delete" | null;

export function DecksPanel() {
  const t = useTranslations();
  const router = useRouter();
  const tauri = useIsTauri();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const { decks, loading, error, desktop, hydrate, create, rename, remove } =
    useDeckStore();

  const [dialog, setDialog] = useState<DialogMode>(null);
  const [target, setTarget] = useState<Deck | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void hydrate();
  }, [hydrate, activeProfileId]);

  function openCreate() {
    setTarget(null);
    setName("");
    setDescription("");
    setDialog("create");
  }

  function openRename(deck: Deck) {
    setTarget(deck);
    setName(deck.name);
    setDescription(deck.description ?? "");
    setDialog("rename");
  }

  function openDelete(deck: Deck) {
    setTarget(deck);
    setDialog("delete");
  }

  function closeDialog() {
    if (busy) return;
    setDialog(null);
    setTarget(null);
  }

  async function handleCreate() {
    const trimmed = name.trim();
    if (!trimmed) return;
    setBusy(true);
    try {
      const deck = await create(trimmed, description.trim() || null);
      if (deck) {
        toast.success(t.decks.created);
        setDialog(null);
        setName("");
        setDescription("");
        router.push(`/decks/view?id=${encodeURIComponent(deck.id)}`);
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.decks.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleRename() {
    if (!target) return;
    const trimmed = name.trim();
    if (!trimmed) return;
    setBusy(true);
    try {
      await rename(target.id, trimmed, description.trim() || null);
      toast.success(t.decks.renamed);
      setDialog(null);
      setTarget(null);
      setDescription("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.decks.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleDelete() {
    if (!target) return;
    setBusy(true);
    try {
      await remove(target.id);
      toast.success(t.decks.deleted);
      setDialog(null);
      setTarget(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.decks.error);
    } finally {
      setBusy(false);
    }
  }

  if (!desktop && !tauri) {
    return (
      <p className="text-sm text-muted-foreground">{t.decks.desktopOnly}</p>
    );
  }

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35 }}
      className="flex w-full flex-col gap-6"
    >
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-1">
          <h2 className="text-2xl font-semibold tracking-tight">{t.decks.title}</h2>
          <p className="text-sm text-muted-foreground text-pretty">
            {t.decks.description}
          </p>
        </div>
        <Button data-tour="create-deck" onClick={openCreate}>
          <Plus className="size-4" />
          {t.decks.create}
        </Button>
      </div>

      {error && decks.length === 0 ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}

      {loading && decks.length === 0 ? <ListSkeleton rows={4} /> : null}

      {!loading && !error && decks.length === 0 ? (
        <div className="flex flex-col items-start gap-4 rounded-lg border border-dashed border-border/80 bg-muted/20 px-6 py-10">
          <div className="flex size-10 items-center justify-center rounded-md bg-muted">
            <Layers className="size-5 text-muted-foreground" />
          </div>
          <div className="space-y-1">
            <p className="font-medium">{t.decks.emptyTitle}</p>
            <p className="max-w-md text-sm text-muted-foreground text-pretty">
              {t.decks.emptyHint}
            </p>
          </div>
          <Button data-tour="create-deck-empty" onClick={openCreate}>
            <Plus className="size-4" />
            {t.decks.create}
          </Button>
        </div>
      ) : null}

      {decks.length > 0 ? (
        <ul className="divide-y divide-border rounded-lg border border-border/60">
          {decks.map((deck, index) => (
            <motion.li
              key={deck.id}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.03 * index, duration: 0.25 }}
              className="flex flex-wrap items-center justify-between gap-3 px-4 py-3"
            >
              <Link
                href={`/decks/view?id=${encodeURIComponent(deck.id)}`}
                className="min-w-0 flex-1 space-y-0.5 rounded-md outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                <p className="truncate font-medium">{deck.name}</p>
                {deck.description ? (
                  <p className="truncate text-sm text-muted-foreground">
                    {deck.description}
                  </p>
                ) : null}
              </Link>
              <div className="flex shrink-0 flex-wrap items-center gap-1">
                <Button variant="outline" size="sm" asChild>
                  <Link
                    href={`/decks/view?id=${encodeURIComponent(deck.id)}`}
                  >
                    <Layers className="size-4" />
                    {t.decks.open}
                    <ArrowRight className="size-4 rtl:rotate-180" />
                  </Link>
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t.decks.rename}
                  onClick={() => openRename(deck)}
                >
                  <Pencil className="size-4" />
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t.decks.delete}
                  onClick={() => openDelete(deck)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            </motion.li>
          ))}
        </ul>
      ) : null}

      <Dialog
        open={dialog === "create" || dialog === "rename"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {dialog === "rename" ? t.decks.renameTitle : t.decks.createTitle}
            </DialogTitle>
            <DialogDescription>
              {dialog === "rename" ? t.decks.renameHint : t.decks.createHint}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="deck-name">{t.decks.name}</Label>
              <Input
                id="deck-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder={t.decks.namePlaceholder}
                autoFocus
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    if (dialog === "rename") void handleRename();
                    else void handleCreate();
                  }
                }}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="deck-description">{t.decks.descriptionLabel}</Label>
              <Input
                id="deck-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder={t.decks.descriptionPlaceholder}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={closeDialog} disabled={busy}>
              {t.decks.cancel}
            </Button>
            <Button
              onClick={() =>
                void (dialog === "rename" ? handleRename() : handleCreate())
              }
              disabled={busy || !name.trim()}
            >
              {busy ? <Spinner /> : null}
              {dialog === "rename" ? t.decks.rename : t.decks.create}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDeleteDialog
        open={dialog === "delete"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
        title={t.decks.deleteTitle}
        description={t.decks.deleteHint.replace("{name}", target?.name ?? "")}
        cancelLabel={t.decks.cancel}
        deleteLabel={t.decks.delete}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handleDelete()}
      />
    </motion.section>
  );
}
