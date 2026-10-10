import { invoke } from "@tauri-apps/api/core";
import type {
  AppPreferences,
  Attachment,
  AttachmentId,
  AttachmentOwnerType,
  Card,
  CardId,
  Deck,
  DeckId,
  DeckSettings,
  Grade,
  ImportBackupMode,
  ImportBackupResult,
  InkDocument,
  InkDocumentBundle,
  InkDocumentId,
  InkDocumentRef,
  InkOwnerType,
  LanguageAssistOverrides,
  LanguageAssistSettings,
  MediaIntegrityReport,
  MediaLibraryFilter,
  MediaLibraryItem,
  MediaLibraryStats,
  MediaPurgeResult,
  Profile,
  ProfileId,
  ProfileLeitnerSettings,
  ReviewOutcome,
  RichTextDocument,
  StudyConsistencyStats,
  StudyStats,
  TrashEntityKind,
  TrashItem,
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
  deleteProfile: (id: ProfileId) => call<void>("delete_profile", { id }),
  switchProfile: (id: ProfileId) => call<Profile>("switch_profile", { id }),
  isInitialProfileSetupPending: () =>
    call<boolean>("is_initial_profile_setup_pending"),
  completeInitialProfileSetup: (displayName: string) =>
    call<Profile>("complete_initial_profile_setup", {
      input: { displayName },
    }),

  listDecks: () => call<Deck[]>("list_decks"),
  getDeck: (id: DeckId) => call<Deck>("get_deck", { id }),
  createDeck: (name: string, description?: string | null) =>
    call<Deck>("create_deck", {
      input: { name, description: description ?? null },
    }),
  renameDeck: (id: DeckId, name: string, description?: string | null) =>
    call<Deck>("rename_deck", {
      input: { id, name, description: description ?? null },
    }),
  updateDeckSettings: (id: DeckId, settings: DeckSettings) =>
    call<Deck>("update_deck_settings", { input: { id, settings } }),
  deleteDeck: (id: DeckId) => call<void>("delete_deck", { id }),

  getProfileAlgorithm: () =>
    call<ProfileLeitnerSettings>("get_profile_algorithm"),
  setProfileAlgorithm: (settings: ProfileLeitnerSettings) =>
    call<ProfileLeitnerSettings>("set_profile_algorithm", { settings }),
  applyProfileAlgorithmToAllDecks: (inherit: boolean) =>
    call<number>("apply_profile_algorithm_to_all_decks", {
      input: { inherit },
    }),

  listCards: (deckId: DeckId) => call<Card[]>("list_cards", { deckId }),
  getCard: (id: CardId) => call<Card>("get_card", { id }),
  createCard: (
    deckId: DeckId,
    frontContent: RichTextDocument,
    backContent: RichTextDocument,
  ) =>
    call<Card>("create_card", {
      input: { deckId, frontContent, backContent },
    }),
  updateCard: (
    id: CardId,
    frontContent: RichTextDocument,
    backContent: RichTextDocument,
  ) =>
    call<Card>("update_card", { input: { id, frontContent, backContent } }),
  deleteCard: (id: CardId) => call<void>("delete_card", { id }),

  getLanguageAssistSettings: () =>
    call<LanguageAssistSettings>("get_language_assist_settings"),
  setLanguageAssistSettings: (settings: LanguageAssistSettings) =>
    call<LanguageAssistSettings>("set_language_assist_settings", { settings }),
  regenerateLanguageAssist: (cardId: CardId) =>
    call<Card>("regenerate_language_assist", { cardId }),
  readPronunciationAudio: (cardId: CardId) =>
    call<number[]>("read_pronunciation_audio", { cardId }),
  setCardLanguageAssist: (cardId: CardId, overrides: LanguageAssistOverrides) =>
    call<Card>("set_card_language_assist", {
      input: { cardId, overrides },
    }),

  listDueCards: (deckId?: DeckId | null) =>
    call<Card[]>("list_due_cards", { deckId: deckId ?? null }),
  submitReview: (cardId: string, grade: Grade, durationMs?: number) =>
    call<ReviewOutcome>("submit_review", {
      input: { cardId, grade, durationMs },
    }),
  getStudyStats: () => call<StudyStats>("get_study_stats"),
  getStudyConsistencyStats: () =>
    call<StudyConsistencyStats>("get_study_consistency_stats"),

  getAppPreferences: () => call<AppPreferences>("get_app_preferences"),
  setAppPreferences: (preferences: AppPreferences) =>
    call<AppPreferences>("set_app_preferences", { preferences }),

  exportBackup: (outputPath?: string) =>
    call<string>("export_backup", {
      input: { outputPath: outputPath ?? null },
    }),
  importBackup: (path: string, mode: ImportBackupMode = "create_profile") =>
    call<ImportBackupResult>("import_backup", {
      input: { path, mode },
    }),

  createAttachment: (input: {
    ownerType: AttachmentOwnerType;
    ownerId: string;
    fileName: string;
    mimeType: string;
    bytes?: number[];
    sourcePath?: string | null;
  }) => call<Attachment>("create_attachment", { input }),
  listAttachments: (ownerType: AttachmentOwnerType, ownerId: string) =>
    call<Attachment[]>("list_attachments", {
      input: { ownerType, ownerId },
    }),
  deleteAttachment: (id: AttachmentId) =>
    call<void>("delete_attachment", { id }),
  restoreAttachment: (id: AttachmentId) =>
    call<Attachment>("restore_attachment", { id }),
  readAttachmentBytes: (id: AttachmentId) =>
    call<number[]>("read_attachment_bytes", { id }),

  listMediaLibrary: (filter?: MediaLibraryFilter | null) =>
    call<MediaLibraryItem[]>("list_media_library", {
      filter: filter ?? null,
    }),
  getMediaLibraryStats: () =>
    call<MediaLibraryStats>("get_media_library_stats"),
  purgeAttachment: (id: AttachmentId) =>
    call<MediaPurgeResult>("purge_attachment", { id }),
  purgeDeletedAttachments: () =>
    call<MediaPurgeResult>("purge_deleted_attachments"),
  removeOrphanMediaDirs: () => call<number>("remove_orphan_media_dirs"),
  verifyMediaLibrary: () =>
    call<MediaIntegrityReport>("verify_media_library"),

  createInkDocument: (input: {
    ownerType: InkOwnerType;
    ownerId: string;
    width: number;
    height: number;
  }) => call<InkDocumentBundle>("create_ink_document", { input }),
  getInkDocument: (id: InkDocumentId) =>
    call<InkDocumentBundle>("get_ink_document", { id }),
  listInkDocuments: (ownerType: InkOwnerType, ownerId: string) =>
    call<InkDocumentRef[]>("list_ink_documents", {
      input: { ownerType, ownerId },
    }),
  saveInkDocument: (document: InkDocument) =>
    call<InkDocumentBundle>("save_ink_document", { document }),

  listDeletedItems: () => call<TrashItem[]>("list_deleted_items"),
  restoreTrashItem: (entityKind: TrashEntityKind, id: string) =>
    call<void>("restore_item", { input: { entityKind, id } }),
  permanentlyDeleteTrashItem: (entityKind: TrashEntityKind, id: string) =>
    call<void>("permanently_delete_item", { input: { entityKind, id } }),
};
