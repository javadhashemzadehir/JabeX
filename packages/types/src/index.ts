/** Shared domain contracts for JabeX (aligned with Rust backend). */

export type Locale = "fa" | "en";

export type ThemePreference = "light" | "dark" | "system";

/** UUIDv7 string */
export type EntityId = string;

export type ProfileId = EntityId;
export type DeckId = EntityId;
export type CardId = EntityId;
export type ReviewId = EntityId;
export type AttachmentId = EntityId;
export type InkDocumentId = EntityId;

export type AttachmentOwnerType = "card" | "document";
export type AttachmentType = "image" | "audio" | "video" | "pdf";
export type InkOwnerType = "card" | "document";
export type InkTool = "pen" | "highlighter" | "eraser";

export interface Attachment {
  id: AttachmentId;
  ownerType: AttachmentOwnerType;
  ownerId: EntityId;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  hash: string;
  relativePath: string;
  revision: number;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
  /** Local profile UUID that soft-deleted this row (not an online account). */
  deletedBy?: string | null;
}

export type TrashEntityKind = "deck" | "card" | "attachment" | "ink_document";

/** Unified trash row for the profile Trash screen. */
export interface TrashItem {
  entityKind: TrashEntityKind;
  id: EntityId;
  title: string;
  deletedAt: string;
  deletedBy?: string | null;
  parentId?: string | null;
  parentKind?: TrashEntityKind | null;
}

/** Filter for profile media library browsing. */
export interface MediaLibraryFilter {
  includeDeleted?: boolean;
  attachmentType?: AttachmentType | null;
  query?: string | null;
}

/** Attachment row enriched for library browsing. */
export interface MediaLibraryItem extends Attachment {
  attachmentType?: AttachmentType | null;
  fileMissing: boolean;
}

/** Aggregate media usage for the active local profile (no telemetry). */
export interface MediaLibraryStats {
  totalCount: number;
  deletedCount: number;
  totalBytes: number;
  deletedBytes: number;
  imageCount: number;
  audioCount: number;
  videoCount: number;
  pdfCount: number;
  missingFileCount: number;
  orphanDirCount: number;
}

export interface MediaPurgeResult {
  purgedCount: number;
  freedBytes: number;
}

export interface MediaIntegrityReport {
  checked: number;
  missingFiles: AttachmentId[];
  checksumMismatches: AttachmentId[];
  orphanDirs: AttachmentId[];
}

export interface InkPoint {
  x: number;
  y: number;
  pressure: number;
  timestamp: number;
}

export interface InkStroke {
  id: EntityId;
  tool: InkTool;
  color: string;
  width: number;
  points: InkPoint[];
}

export interface InkLayer {
  id: EntityId;
  order: number;
  visible: boolean;
  opacity: number;
  strokes: InkStroke[];
}

export interface InkDocument {
  version: 1;
  id: InkDocumentId;
  width: number;
  height: number;
  layers: InkLayer[];
}

export interface InkDocumentRef {
  id: InkDocumentId;
  ownerType: InkOwnerType;
  ownerId: EntityId;
  relativePath: string;
  width: number;
  height: number;
  contentHash: string;
  revision: number;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
  deletedBy?: string | null;
}

export interface InkDocumentBundle {
  reference: InkDocumentRef;
  document: InkDocument;
}

export type CardType = "basic";
export type CardState = "new" | "learning" | "review" | "suspended";
export type Grade = "again" | "hard" | "good" | "easy";

/** Named Leitner interval packages. */
export type LeitnerPresetId =
  | "recommended"
  | "intensive"
  | "relaxed"
  | "longTerm";

export type AlgorithmMode = "preset" | "custom";

/** Profile-wide default Leitner algorithm. */
export interface ProfileLeitnerSettings {
  mode: AlgorithmMode;
  presetId: LeitnerPresetId;
  intervalsDays?: number[] | null;
}

/** Per-deck algorithm; `inherit: true` uses the profile default. */
export interface DeckAlgorithmSettings {
  inherit: boolean;
  mode: AlgorithmMode;
  presetId: LeitnerPresetId;
  intervalsDays?: number[] | null;
}

export interface LeitnerConfig {
  presetId?: LeitnerPresetId | null;
  intervalsDays: [number, number, number, number, number];
}

export interface LeitnerPresetInfo {
  id: LeitnerPresetId;
  intervalsDays: [number, number, number, number, number];
}

export interface EffectiveAlgorithm {
  deckId?: string | null;
  config: LeitnerConfig;
  profile: ProfileLeitnerSettings;
}

/** Per-deck / per-card language assist override (profile settings remain master). */
export type AssistToggle = "inherit" | "enabled" | "disabled";

export interface LanguageAssistOverrides {
  translation: AssistToggle;
  pronunciation: AssistToggle;
}

export interface DeckSettings {
  newCardsPerDay?: number;
  reviewsPerDay?: number;
  algorithm?: DeckAlgorithmSettings;
  languageAssist?: LanguageAssistOverrides;
}

/** TipTap-compatible node tree (opaque JSON). */
export type RichTextNode = Record<string, unknown>;

/**
 * Versioned card-face document.
 * Shape: `{ version: 1, type: "doc", content: TipTapNodes[] }`
 */
export interface RichTextDocument {
  version: 1;
  type: "doc";
  content: RichTextNode[];
}

export const RICH_TEXT_DOCUMENT_VERSION = 1 as const;

/** Local device profile — not an online account. */
export interface Profile {
  id: ProfileId;
  displayName: string;
  avatar?: string | null;
  createdAt: string;
  updatedAt: string;
  lastActiveAt: string;
  deletedAt?: string | null;
  deletedBy?: string | null;
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
  deletedBy?: string | null;
}

/** Optional Smart Language Assistant metadata (local-first; network never mandatory). */
export type LanguageAssistStatus =
  | "idle"
  | "pending"
  | "ready"
  | "failed"
  | "skipped";

export type LanguageAssistProviderKind = "none" | "google";

export interface TransliterationMetadata {
  isTransliteration: boolean;
  sourceScript: string;
  resolvedWord?: string | null;
  resolvedLanguage?: string | null;
  confidence: number;
}

export interface LanguageAssistSettings {
  autoTranslation: boolean;
  pronunciationGeneration: boolean;
  provider: LanguageAssistProviderKind;
  targetLanguage: string;
}

export interface Card {
  id: CardId;
  deckId: DeckId;
  cardType: CardType;
  frontContent: RichTextDocument;
  backContent: RichTextDocument;
  boxPosition: number;
  dueAt: string;
  state: CardState;
  reviewCount: number;
  lapseCount: number;
  difficulty?: number | null;
  detectedLanguage?: string | null;
  translation?: string | null;
  pronunciationAudioPath?: string | null;
  transliteration?: TransliterationMetadata | null;
  languageAssistStatus?: LanguageAssistStatus;
  languageAssistOverrides?: LanguageAssistOverrides;
  revision: number;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
  deletedBy?: string | null;
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

/** Aggregate counts for the active local profile (no telemetry). */
export interface StudyStats {
  deckCount: number;
  cardCount: number;
  dueCount: number;
  reviewsToday: number;
  /** Leitner boxes 1–5 (index 0 = box 1). */
  cardsByBox: [number, number, number, number, number];
}

/** Heatmap intensity from cards reviewed that day. */
export type ActivityIntensity = "none" | "low" | "medium" | "high";

/** One local calendar day of Leitner review activity. */
export interface StudyActivity {
  id: string;
  profileId: string;
  /** Local calendar day `YYYY-MM-DD`. */
  activityDate: string;
  cardsReviewed: number;
  minutesSpent: number;
  createdAt: string;
}

export interface HeatmapDay {
  date: string;
  cardsReviewed: number;
  minutesSpent: number;
  intensity: ActivityIntensity;
}

/** Learning consistency stats derived from study activity (never telemetry). */
export interface StudyConsistencyStats {
  currentStreak: number;
  longestStreak: number;
  totalStudyDays: number;
  cardsReviewedToday: number;
  cardsReviewedThisWeek: number;
  cardsReviewedThisMonth: number;
  averageCardsPerStudyDay: number;
  minutesSpentToday: number;
  heatmap: HeatmapDay[];
}

export interface MediaChecksum {
  relativePath: string;
  sha256: string;
  sizeBytes: number;
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
    attachments?: number;
    inkDocuments?: number;
    studyActivity?: number;
  };
  mediaVersion?: number;
  mediaChecksums?: MediaChecksum[];
}

export interface ImportBackupResult {
  profileId: ProfileId;
  mode: string;
  decks: number;
  cards: number;
  reviews: number;
  attachments?: number;
  inkDocuments?: number;
}

export type ImportBackupMode = "create_profile" | "replace_active";
