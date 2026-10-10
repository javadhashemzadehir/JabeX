import { invoke } from "@tauri-apps/api/core";
import type {
  Card,
  CardId,
  Deck,
  DeckId,
  Grade,
  ImportBackupMode,
  ImportBackupResult,
  JbxManifest,
  Profile,
  ProfileId,
  ReviewOutcome,
} from "@jabex/types";

/** True when running inside the Tauri webview. */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      `Tauri command "${cmd}" is only available in the desktop app`,
    );
  }
  return invoke<T>(cmd, args);
}

export const api = {
  listProfiles: () => call<Profile[]>("list_profiles"),
  getActiveProfile: () => call<Profile>("get_active_profile"),
  createProfile: (displayName: string, avatar?: string | null) =>
    call<Profile>("create_profile", {
      input: { displayName, avatar: avatar ?? null },
    }),
  updateProfile: (
    id: ProfileId,
    displayName?: string,
    avatar?: string | null,
  ) =>
    call<Profile>("update_profile", {
      input: { id, displayName, avatar },
    }),
  deleteProfile: (id: ProfileId) => call<void>("delete_profile", { id }),
  switchProfile: (id: ProfileId) => call<Profile>("switch_profile", { id }),

  listDecks: () => call<Deck[]>("list_decks"),
  getDeck: (id: DeckId) => call<Deck>("get_deck", { id }),
  createDeck: (name: string, description?: string | null) =>
    call<Deck>("create_deck", {
      input: { name, description: description ?? null },
    }),
  renameDeck: (id: DeckId, name: string) =>
    call<Deck>("rename_deck", { input: { id, name } }),
  deleteDeck: (id: DeckId) => call<void>("delete_deck", { id }),

  listCards: (deckId: DeckId) => call<Card[]>("list_cards", { deckId }),
  createCard: (deckId: DeckId, frontContent: string, backContent: string) =>
    call<Card>("create_card", {
      input: { deckId, frontContent, backContent },
    }),
  updateCard: (id: CardId, frontContent: string, backContent: string) =>
    call<Card>("update_card", { input: { id, frontContent, backContent } }),
  deleteCard: (id: CardId) => call<void>("delete_card", { id }),

  listDueCards: (deckId?: DeckId | null) =>
    call<Card[]>("list_due_cards", { deckId: deckId ?? null }),
  submitReview: (cardId: string, grade: Grade, durationMs?: number) =>
    call<ReviewOutcome>("submit_review", {
      input: { cardId, grade, durationMs },
    }),

  exportBackup: (outputPath?: string) =>
    call<string>("export_backup", {
      input: { outputPath: outputPath ?? null },
    }),
  validateBackup: (path: string) =>
    call<JbxManifest>("validate_backup", { path }),
  importBackup: (path: string, mode: ImportBackupMode = "create_profile") =>
    call<ImportBackupResult>("import_backup", {
      input: { path, mode },
    }),
};
