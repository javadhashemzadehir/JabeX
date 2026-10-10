"use client";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { HoldToConfirmButton } from "@/components/ui/hold-to-confirm-button";

export interface ConfirmDeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  cancelLabel: string;
  deleteLabel: string;
  holdHint: string;
  confirmingLabel: string;
  busy?: boolean;
  onConfirm: () => void;
}

export function ConfirmDeleteDialog({
  open,
  onOpenChange,
  title,
  description,
  cancelLabel,
  deleteLabel,
  holdHint,
  confirmingLabel,
  busy = false,
  onConfirm,
}: ConfirmDeleteDialogProps) {
  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (busy) return;
        onOpenChange(next);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <p className="text-xs text-muted-foreground text-pretty">{holdHint}</p>
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={busy}
          >
            {cancelLabel}
          </Button>
          <HoldToConfirmButton
            key={open ? "open" : "closed"}
            holdLabel={deleteLabel}
            confirmingLabel={confirmingLabel}
            disabled={busy}
            onConfirm={onConfirm}
          >
            {deleteLabel}
          </HoldToConfirmButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
