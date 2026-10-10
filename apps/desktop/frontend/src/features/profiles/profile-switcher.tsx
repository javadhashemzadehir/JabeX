"use client";

import { useEffect, useState } from "react";
import { UserRound } from "lucide-react";
import { toast } from "sonner";
import type { Profile } from "@jabex/types";
import { Button } from "@/components/ui/button";
import { ConfirmDeleteDialog } from "@/components/ui/confirm-delete-dialog";
import { Spinner } from "@/components/ui/spinner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useTranslations } from "@/hooks/use-locale";
import { useProfileStore } from "./profile-store";
import { useProfileSwitcherUi } from "./profile-switcher-ui";

export function ProfileSwitcher() {
  const t = useTranslations();
  const { profiles, active, desktop, hydrate, create, switchTo, remove } =
    useProfileStore();
  const menuOpen = useProfileSwitcherUi((s) => s.menuOpen);
  const setMenuOpen = useProfileSwitcherUi((s) => s.setMenuOpen);
  const [createOpen, setCreateOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  if (!desktop) {
    return null;
  }

  async function handleCreate() {
    const trimmed = name.trim();
    if (!trimmed) return;
    setBusy(true);
    try {
      const profile = await create(trimmed);
      if (profile) {
        await switchTo(profile.id);
        toast.success(t.profiles.created);
        setCreateOpen(false);
        setName("");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.profiles.error);
    } finally {
      setBusy(false);
    }
  }

  async function handleSwitch(id: string) {
    if (id === active?.id) return;
    try {
      await switchTo(id);
      toast.success(t.profiles.switched);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.profiles.error);
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    setBusy(true);
    try {
      await remove(deleteTarget.id);
      toast.success(t.profiles.deleted);
      setDeleteTarget(null);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t.profiles.error);
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <DropdownMenu open={menuOpen} onOpenChange={setMenuOpen}>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="gap-2"
            aria-label={t.profiles.label}
          >
            <UserRound className="size-4" aria-hidden />
            <span className="max-w-28 truncate">
              {active?.displayName ?? t.profiles.label}
            </span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel>{t.profiles.label}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {profiles.map((profile) => (
            <DropdownMenuItem
              key={profile.id}
              className="flex items-center justify-between gap-2"
              onSelect={(event) => {
                event.preventDefault();
                void handleSwitch(profile.id);
              }}
            >
              <span className="truncate">
                {profile.displayName}
                {profile.id === active?.id ? " ✓" : ""}
              </span>
              {profile.id !== active?.id ? (
                <button
                  type="button"
                  className="text-xs text-muted-foreground hover:text-destructive"
                  onClick={(event) => {
                    event.stopPropagation();
                    setMenuOpen(false);
                    setDeleteTarget(profile);
                  }}
                >
                  {t.profiles.delete}
                </button>
              ) : null}
            </DropdownMenuItem>
          ))}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onSelect={(event) => {
              event.preventDefault();
              setCreateOpen(true);
            }}
          >
            {t.profiles.create}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t.profiles.createTitle}</DialogTitle>
            <DialogDescription>{t.profiles.createHint}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="profile-name">{t.profiles.displayName}</Label>
            <Input
              id="profile-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t.profiles.displayNamePlaceholder}
              autoFocus
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setCreateOpen(false)}
              disabled={busy}
            >
              {t.profiles.cancel}
            </Button>
            <Button onClick={() => void handleCreate()} disabled={busy || !name.trim()}>
              {busy ? <Spinner /> : null}
              {t.profiles.create}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDeleteDialog
        open={deleteTarget != null}
        onOpenChange={(open) => {
          if (!open && !busy) setDeleteTarget(null);
        }}
        title={t.profiles.deleteTitle}
        description={t.profiles.deleteHint.replace(
          "{name}",
          deleteTarget?.displayName ?? "",
        )}
        cancelLabel={t.profiles.cancel}
        deleteLabel={t.profiles.delete}
        holdHint={t.common.holdToDeleteHint}
        confirmingLabel={t.common.holdToDeleteConfirming}
        busy={busy}
        onConfirm={() => void handleDelete()}
      />
    </>
  );
}
