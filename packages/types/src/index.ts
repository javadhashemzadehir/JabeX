/** Shared domain contracts for JabeX (aligned with Rust backend). */

export type Locale = "fa" | "en";

export type ThemePreference = "light" | "dark" | "system";

/** UUIDv7 string */
export type EntityId = string;

export type ProfileId = EntityId;
export type DeckId = EntityId;
export type CardId = EntityId;
export type ReviewId = EntityId;

export type CardType = "basic";
export type CardState = "new" | "learning" | "review" | "suspended";
export type Grade = "again" | "hard" | "good" | "easy";

export interface DeckSettings {
  newCardsPerDay?: number;
  reviewsPerDay?: number;
}

/** Local device profile — not an online account. */
export interface Profile {
  id: ProfileId;
  displayName: string;
  avatar?: string | null;
  createdAt: string;
  updatedAt: string;
  lastActiveAt: string;
  deletedAt?: string | null;
}

export interface Deck {
  id: DeckId;
  name: string;
  description?: string | null;
  settings: DeckSettings;
  revision: number;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
}

export interface Card {
  id: CardId;
  deckId: DeckId;
  cardType: CardType;
  frontContent: string;
  backContent: string;
  boxPosition: number;
  dueAt: string;
  state: CardState;
  reviewCount: number;
  lapseCount: number;
  difficulty?: number | null;
  revision: number;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
}

export interface Review {
  id: ReviewId;
  cardId: CardId;
  grade: Grade;
  reviewedAt: string;
  durationMs?: number | null;
  boxBefore: number;
  boxAfter: number;
  revision: number;
  createdAt: string;
  updatedAt: string;
}

export interface AppSettings {
  locale: Locale;
  theme: ThemePreference;
}

/** Device-level preferences (tray / window) — not per-profile study settings. */
export interface AppPreferences {
  enableMinimizeToTray: boolean;
  closeButtonHidesToTray: boolean;
  startMinimized: boolean;
  launchOnStartup: boolean;
}

/** Payload for `tray://action` (Rust emits; frontend listens). */
export type TrayActionPayload =
  | { action: "switch-profile" }
  | { action: "restoreBackup" }
  | { action: "settings" }
  | { action: "backupCreated"; path: string }
  | { action: "backupFailed"; message: string };

export interface ReviewOutcome {
  card: Card;
  review: Review;
}

export interface JbxManifest {
  format: string;
  version: number;
  createdAt: string;
  profileId: ProfileId;
  profileDisplayName: string;
  entityCounts: {
    decks: number;
    cards: number;
    reviews: number;
    settings: number;
  };
}

export interface ImportBackupResult {
  profileId: ProfileId;
  mode: string;
  decks: number;
  cards: number;
  reviews: number;
}

export type ImportBackupMode = "create_profile" | "replace_active";
