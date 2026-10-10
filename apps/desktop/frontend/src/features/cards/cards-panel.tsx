"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import {
  ArrowLeft,
  BookOpen,
  Maximize2,
  Minimize2,
  Plus,
  SquareStack,
  Trash2,
} from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { motion } from "framer-motion";
import { toast } from "sonner";
import type { Card, CardSummary, RichTextDocument } from "@jabex/types";
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
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { DeckAlgorithmPanel } from "@/features/algorithm/deck-algorithm-panel";
import { CardAttachmentsPanel } from "@/features/attachments/card-attachments-panel";
import {
  RichTextEditor,
  type EditorLayoutWidth,
} from "@/features/editor/rich-text-editor";
import { InkEditor } from "@/features/ink/ink-editor";
import { CardLanguageAssistPanel } from "@/features/language-assist/card-language-assist-panel";
import { DeckLanguageAssistPanel } from "@/features/language-assist/deck-language-assist-panel";
import { LanguageBadge } from "@/features/language-assist/language-badge";
import { useIsTauri } from "@/hooks/use-is-tauri";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "@/features/profiles/profile-store";
import {
  emptyRichTextDocument,
  isRichTextBlank,
  plainTextFromDocument,
} from "@/lib/rich-text";
import { cn } from "@/lib/utils";
import { useCardStore } from "./card-store";

type DialogMode = "create" | "edit" | "delete" | null;
const ROW_ESTIMATE = 96;

type CardsVirtualListProps = {
  cards: CardSummary[];
  total: number;
  hasMore: boolean;
  loadingMore: boolean;
  boxLabel: string;
  openLabel: string;
  deleteLabel: string;
  loadingLabel: string;
  onOpen: (card: CardSummary) => void;
  onDelete: (card: CardSummary) => void;
  onLoadMore: () => void;
};

// Opt out of React Compiler: useVirtualizer returns a mutable instance.
function CardsVirtualList({
  cards,
  total,
  hasMore,
  loadingMore,
  boxLabel,
  openLabel,
  deleteLabel,
  loadingLabel,
  onOpen,
  onDelete,
  onLoadMore,
}: CardsVirtualListProps) {
  "use no memo";

  const listParentRef = useRef<HTMLDivElement>(null);

  // eslint-disable-next-line react-hooks/incompatible-library -- mutable virtualizer; opted out via "use no memo"
  const virtualizer = useVirtualizer({
    count: cards.length,
    getScrollElement: () => listParentRef.current,
    estimateSize: () => ROW_ESTIMATE,
    overscan: 8,
  });

  const virtualItems = virtualizer.getVirtualItems();
  const lastVirtualIndex = virtualItems[virtualItems.length - 1]?.index ?? -1;

  useEffect(() => {
    if (lastVirtualIndex < 0 || !hasMore || loadingMore) return;
    if (lastVirtualIndex >= cards.length - 8) {
      onLoadMore();
    }
  }, [lastVirtualIndex, hasMore, loadingMore, cards.length, onLoadMore]);

  return (
    <div className="space-y-2">
      {total > 0 ? (
        <p className="text-xs text-muted-foreground tabular-nums">
          {cards.length}
          {total > cards.length ? ` / ${total}` : ""}
        </p>
      ) : null}
      <div
        ref={listParentRef}
        className="max-h-[min(70vh,40rem)] overflow-auto rounded-lg border border-border/60"
      >
        <ul
          className="relative w-full"
          style={{ height: `${virtualizer.getTotalSize()}px` }}
        >
          {virtualItems.map((virtualRow) => {
            const card = cards[virtualRow.index];
            if (!card) return null;
            return (
              <li
                key={card.id}
                data-index={virtualRow.index}
                ref={virtualizer.measureElement}
                className="absolute inset-s-0 top-0 flex w-full flex-wrap items-start justify-between gap-3 border-b border-border px-4 py-3"
                style={{
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <button
                  type="button"
                  onClick={() => onOpen(card)}
                  className="min-w-0 flex-1 space-y-1 rounded-md text-start outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="line-clamp-2 font-medium" dir="auto">
                      {card.frontPreview}
                    </p>
                    <LanguageBadge card={card} />
                  </div>
                  <p
                    className="line-clamp-2 text-sm text-muted-foreground"
                    dir="auto"
                  >
                    {card.backPreview}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {boxLabel.replace("{box}", String(card.boxPosition))}
                  </p>
                </button>
                <div className="flex shrink-0 flex-wrap items-center gap-1">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => onOpen(card)}
                  >
                    <SquareStack className="size-4" />
                    {openLabel}
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={deleteLabel}
                    onClick={() => onDelete(card)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </li>
            );
          })}
        </ul>
      </div>
      {loadingMore ? (
        <p className="text-xs text-muted-foreground">{loadingLabel}</p>
      ) : null}
    </div>
  );
}

export function CardsPanel({
  deckId,
  openCreate: openCreateOnMount = false,
}: {
  deckId: string;
  openCreate?: boolean;
}) {
  const t = useTranslations();
  const tauri = useIsTauri();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const {
    deck,
    cards,
    total,
    hasMore,
    loading,
    loadingMore,
    error,
    desktop,
    hydrate,
    loadMore,
    create,
    update,
    remove,
    getCard,
    setDeck,
    reset,
  } = useCardStore();

  const [dialog, setDialog] = useState<DialogMode>(null);
  const [target, setTarget] = useState<Card | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<CardSummary | null>(null);
  const [front, setFront] = useState<RichTextDocument>(emptyRichTextDocument);
  const [back, setBack] = useState<RichTextDocument>(emptyRichTextDocument);
  const [busy, setBusy] = useState(false);
  const [editorLayout, setEditorLayout] =
    useState<EditorLayoutWidth>("medium");
  const openedCreateFromQuery = useRef(false);

  const canSave = !isRichTextBlank(front) && !isRichTextBlank(back);

  useEffect(() => {
    openedCreateFromQuery.current = false;
    void hydrate(deckId);
    return () => reset();
  }, [hydrate, reset, deckId, activeProfileId]);

  useEffect(() => {
    if (!openCreateOnMount || loading || openedCreateFromQuery.current) return;
    openedCreateFromQuery.current = true;
    setTarget(null);
    setFront(emptyRichTextDocument());
    setBack(emptyRichTextDocument());
    setDialog("create");
  }, [openCreateOnMount, loading, deckId]);

  function openCreate() {
    setTarget(null);
    setDeleteTarget(null);
    setFront(emptyRichTextDocument());
    setBack(emptyRichTextDocument());
    setDialog("create");
  }

  async function openEdit(summary: CardSummary) {
    setBusy(true);
    try {
      const card = await getCard(summary.id);
      if (!card) {
        toast.error(t.cards.error);
        return;
      }
      setDeleteTarget(null);
      setTarget(card);
      setFront(card.frontContent);
      setBack(card.backContent);
      setDialog("edit");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.cards.error);
    } finally {
      setBusy(false);
    }
  }

  function openDelete(summary: CardSummary) {
    setTarget(null);
    setDeleteTarget(summary);
    setDialog("delete");
  }

  function closeDialog() {
    if (busy) return;
    setDialog(null);
    setTarget(null);
    setDeleteTarget(null);
  }

  async function handleCreate() {
    if (!canSave) return;
    setBusy(true);
    try {
      const card = await create(front, back);
      if (card) {
        toast.success(t.cards.created);
        setDialog(null);
        setFront(emptyRichTextDocument());
        setBack(emptyRichTextDocument());
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.cards.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleEdit() {
    if (!target || !canSave) return;
    setBusy(true);
    try {
      await update(target.id, front, back);
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
    if (!deleteTarget) return;
    setBusy(true);
    try {
      await remove(deleteTarget.id);
      toast.success(t.cards.deleted);
      setDialog(null);
      setDeleteTarget(null);
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
          <div className="flex flex-wrap gap-2">
            {cards.length > 0 ? (
              <Button variant="outline" asChild>
                <Link
                  href={`/review?deckId=${encodeURIComponent(deckId)}`}
                >
                  <BookOpen className="size-4" />
                  {t.review.start}
                </Link>
              </Button>
            ) : null}
            <Button onClick={openCreate} disabled={!deck}>
              <Plus className="size-4" />
              {t.cards.create}
            </Button>
          </div>
        </div>
      </div>

      {loading && cards.length === 0 ? <ListSkeleton rows={5} /> : null}

      {!loading && !error && cards.length === 0 ? (
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
      ) : cards.length > 0 ? (
        <CardsVirtualList
          cards={cards}
          total={total}
          hasMore={hasMore}
          loadingMore={loadingMore}
          boxLabel={t.cards.boxLabel}
          openLabel={t.cards.open}
          deleteLabel={t.cards.delete}
          loadingLabel={t.cards.loading}
          onOpen={(card) => void openEdit(card)}
          onDelete={openDelete}
          onLoadMore={() => void loadMore()}
        />
      ) : null}

      {deck ? (
        <div className="grid gap-4 lg:grid-cols-2">
          <DeckAlgorithmPanel deck={deck} onUpdated={setDeck} />
          <DeckLanguageAssistPanel deck={deck} onUpdated={setDeck} />
        </div>
      ) : null}

      <Dialog
        open={dialog === "create" || dialog === "edit"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
      >
        <DialogContent
          className={cn(
            "max-h-[90vh] overflow-y-auto transition-[max-width] duration-200",
            editorLayout === "large"
              ? "max-w-[min(96rem,96vw)]"
              : "max-w-4xl",
          )}
        >
          <DialogHeader>
            <div className="flex items-start justify-between gap-3 pe-8">
              <div className="space-y-1.5">
                <DialogTitle>
                  {dialog === "edit" ? t.cards.editTitle : t.cards.createTitle}
                </DialogTitle>
                <DialogDescription>
                  {dialog === "edit" ? t.cards.editHint : t.cards.createHint}
                </DialogDescription>
              </div>
              <Button
                type="button"
                variant="outline"
                size="icon"
                className="shrink-0"
                aria-label={
                  editorLayout === "large"
                    ? t.editor.layoutMedium
                    : t.editor.layoutLarge
                }
                title={
                  editorLayout === "large"
                    ? t.editor.layoutMedium
                    : t.editor.layoutLarge
                }
                onClick={() =>
                  setEditorLayout((current) =>
                    current === "large" ? "medium" : "large",
                  )
                }
              >
                {editorLayout === "large" ? (
                  <Minimize2 className="size-4" />
                ) : (
                  <Maximize2 className="size-4" />
                )}
              </Button>
            </div>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="card-front">{t.cards.front}</Label>
              <RichTextEditor
                id="card-front"
                value={front}
                onChange={setFront}
                placeholder={t.cards.frontPlaceholder}
                autoFocus
                aria-label={t.cards.front}
                disabled={busy}
                layoutWidth={editorLayout}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="card-back">{t.cards.back}</Label>
              <RichTextEditor
                id="card-back"
                value={back}
                onChange={setBack}
                placeholder={t.cards.backPlaceholder}
                aria-label={t.cards.back}
                disabled={busy}
                layoutWidth={editorLayout}
              />
            </div>
            {dialog === "edit" && target ? (
              <>
                <CardLanguageAssistPanel
                  card={target}
                  onCardUpdated={(next) => {
                    setTarget(next);
                    useCardStore.setState((state) => ({
                      cards: state.cards.map((c) =>
                        c.id === next.id
                          ? {
                              ...c,
                              detectedLanguage: next.detectedLanguage,
                              languageAssistStatus: next.languageAssistStatus,
                              frontPreview:
                                plainTextFromDocument(next.frontContent) ||
                                c.frontPreview,
                              backPreview:
                                plainTextFromDocument(next.backContent) ||
                                c.backPreview,
                              revision: next.revision,
                              updatedAt: next.updatedAt,
                            }
                          : c,
                      ),
                    }));
                  }}
                />
                <CardAttachmentsPanel cardId={target.id} />
                <InkEditor cardId={target.id} />
              </>
            ) : null}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={closeDialog} disabled={busy}>
              {t.cards.cancel}
            </Button>
            <Button
              onClick={() =>
                void (dialog === "edit" ? handleEdit() : handleCreate())
              }
              disabled={busy || !canSave}
            >
              {busy ? <Spinner /> : null}
              {dialog === "edit" ? t.cards.save : t.cards.create}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDeleteDialog
        open={dialog === "delete"}
        onOpenChange={(open) => {
          if (!open) closeDialog();
        }}
        title={t.cards.deleteTitle}
        description={t.cards.deleteHint.replace(
          "{front}",
          deleteTarget?.frontPreview || "…",
        )}
        cancelLabel={t.cards.cancel}
        deleteLabel={t.cards.delete}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handleDelete()}
      />
    </motion.section>
  );
}
