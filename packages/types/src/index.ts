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
 * Versioned card-face document (TipTap-compatible JSON).
 * Shape: `{ version: 1|2, type: "doc", content: TipTapNodes[] }`
 *
 * v2 adds spoilers, math, tables, and highlighted code blocks (ADR 014).
 */
export type RichTextDocumentVersion = 1 | 2;

export interface RichTextDocument {
  version: RichTextDocumentVersion;
  type: "doc";
  content: RichTextNode[];
}

/** Current write version for new/updated card faces. */
export const RICH_TEXT_DOCUMENT_VERSION = 2 as const;

/** Export / backup envelope for a single card face. */
export interface JabexJsonFace {
  format: "jabex-json";
  version: typeof RICH_TEXT_DOCUMENT_VERSION;
  content: RichTextDocument;
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

/** List-row projection without full rich-text documents. */
export interface CardSummary {
  id: CardId;
  deckId: DeckId;
  frontPreview: string;
  backPreview: string;
  boxPosition: number;
  dueAt: string;
  state: CardState;
  detectedLanguage?: string | null;
  languageAssistStatus?: LanguageAssistStatus;
  revision: number;
  createdAt: string;
  updatedAt: string;
}

/** Paginated card list from `list_cards`. */
export interface CardListPage {
  items: CardSummary[];
  total: number;
  offset: number;
  limit: number;
  hasMore: boolean;
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

/** Device-level keyboard shortcut chords (`mod+k`, `space`, `1`, …). */
export interface KeyboardShortcuts {
  openSearch: string;
  reviewFlipOrGood: string;
  reviewFlipOrGoodAlt: string;
  reviewGradeAgain: string;
  reviewGradeHard: string;
  reviewGradeGood: string;
  reviewGradeEasy: string;
}

/** Device-level preferences (tray / window) — not per-profile study settings. */
export interface AppPreferences {
  enableMinimizeToTray: boolean;
  closeButtonHidesToTray: boolean;
  startMinimized: boolean;
  launchOnStartup: boolean;
  keyboardShortcuts: KeyboardShortcuts;
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

/** Extensible global search entity kinds (ADR 015). */
export type SearchEntityType =
  | "card"
  | "deck"
  | "tag"
  | "review"
  | "note"
  | "attachment"
  | "setting"
  | "command";

export type SearchFilter =
  | "all"
  | "cards"
  | "decks"
  | "tags"
  | "reviews"
  | "commands";

export interface SearchHighlightRange {
  start: number;
  end: number;
}

export interface SearchResult {
  entityType: SearchEntityType;
  entityId: string;
  title: string;
  snippet: string;
  breadcrumb?: string | null;
  highlights: SearchHighlightRange[];
  score: number;
  href?: string | null;
  metadata: Record<string, unknown>;
  textDirection: "rtl" | "ltr" | string;
}

export interface SearchResponse {
  results: SearchResult[];
  query: string;
  filter: SearchFilter;
}

export interface RecentSearch {
  id: string;
  profileId: string;
  query: string;
  filters: SearchFilter[];
  createdAt: string;
}

export interface SearchHistoryEntry {
  id: string;
  profileId: string;
  query: string;
  entityType?: SearchEntityType | null;
  entityId?: string | null;
  createdAt: string;
}

/** Aggregated local query frequency (from search history — never uploaded). */
export interface PopularSearch {
  query: string;
  hitCount: number;
  lastSearchedAt: string;
}
