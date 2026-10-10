#!/usr/bin/env node
/**
 * Insert mock seed decks/cards/reviews into the active local profile.
 *
 * Usage:
 *   pnpm db:seed
 *   pnpm db:seed -- --profile "جواد"
 *   JABEX_DATA_DIR=/custom/path pnpm db:seed
 *
 * Prefer closing the app first if you hit SQLITE_BUSY.
 */
import {
  SEED_MARKER_KEY,
  isoNow,
  loadSeedData,
  openAppDatabase,
  openProfileDatabase,
  parseArgs,
  resolveActiveProfile,
  resolveAppDataRoot,
} from "./lib/db.mjs";

/** Persist seed faces as versioned TipTap documents (matches Rust RichTextDocument). */
function toRichTextStorage(value) {
  if (value && typeof value === "object" && value.type === "doc") {
    return JSON.stringify({
      version: value.version ?? 1,
      type: "doc",
      content: value.content ?? [{ type: "paragraph" }],
    });
  }
  const text = String(value ?? "").trim();
  if (!text) {
    return JSON.stringify({
      version: 1,
      type: "doc",
      content: [{ type: "paragraph" }],
    });
  }
  return JSON.stringify({
    version: 1,
    type: "doc",
    content: [
      {
        type: "paragraph",
        content: [{ type: "text", text }],
      },
    ],
  });
}

const { options } = parseArgs();
const root = resolveAppDataRoot();
const seed = loadSeedData();

const appDb = openAppDatabase(root);
const profile = resolveActiveProfile(appDb, options);
appDb.close();

const db = openProfileDatabase(root, profile.id);

const insertDeck = db.prepare(`
  INSERT INTO decks (id, name, description, settings, created_at, updated_at, deleted_at, revision)
  VALUES (?, ?, ?, ?, ?, ?, NULL, 1)
  ON CONFLICT(id) DO UPDATE SET
    name = excluded.name,
    description = excluded.description,
    settings = excluded.settings,
    updated_at = excluded.updated_at,
    deleted_at = NULL,
    revision = decks.revision + 1
`);

const insertCard = db.prepare(`
  INSERT INTO cards (
    id, deck_id, card_type, front_content, back_content,
    box_position, due_at, state, review_count, lapse_count,
    difficulty, created_at, updated_at, deleted_at, revision
  ) VALUES (?, ?, 'basic', ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, NULL, 1)
  ON CONFLICT(id) DO UPDATE SET
    deck_id = excluded.deck_id,
    front_content = excluded.front_content,
    back_content = excluded.back_content,
    box_position = excluded.box_position,
    due_at = excluded.due_at,
    state = excluded.state,
    review_count = excluded.review_count,
    lapse_count = excluded.lapse_count,
    updated_at = excluded.updated_at,
    deleted_at = NULL,
    revision = cards.revision + 1
`);

const insertReview = db.prepare(`
  INSERT INTO reviews (
    id, card_id, grade, reviewed_at, duration_ms,
    box_before, box_after, created_at, updated_at, revision
  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
  ON CONFLICT(id) DO UPDATE SET
    grade = excluded.grade,
    reviewed_at = excluded.reviewed_at,
    duration_ms = excluded.duration_ms,
    box_before = excluded.box_before,
    box_after = excluded.box_after,
    updated_at = excluded.updated_at
`);

const upsertMarker = db.prepare(`
  INSERT INTO settings (key, value, created_at, updated_at, deleted_at, revision)
  VALUES (?, ?, ?, ?, NULL, 1)
  ON CONFLICT(key) DO UPDATE SET
    value = excluded.value,
    updated_at = excluded.updated_at,
    deleted_at = NULL,
    revision = settings.revision + 1
`);

const now = isoNow(0);

try {
  db.exec("BEGIN");

  for (const deck of seed.decks) {
    insertDeck.run(
      deck.id,
      deck.name,
      deck.description ?? null,
      JSON.stringify(deck.settings ?? {}),
      now,
      now,
    );
  }

  for (const card of seed.cards) {
    const dueAt = isoNow(card.dueOffsetDays ?? 0);
    insertCard.run(
      card.id,
      card.deckId,
      toRichTextStorage(card.frontContent),
      toRichTextStorage(card.backContent),
      card.boxPosition ?? 1,
      dueAt,
      card.state ?? "new",
      card.reviewCount ?? 0,
      card.lapseCount ?? 0,
      now,
      now,
    );
  }

  for (const review of seed.reviews ?? []) {
    const reviewedAt = isoNow(review.reviewedOffsetDays ?? 0);
    insertReview.run(
      review.id,
      review.cardId,
      review.grade,
      reviewedAt,
      review.durationMs ?? null,
      review.boxBefore,
      review.boxAfter,
      reviewedAt,
      reviewedAt,
    );
  }

  upsertMarker.run(
    SEED_MARKER_KEY,
    JSON.stringify({
      marker: seed.marker,
      version: seed.version,
      appliedAt: now,
      deckIds: seed.decks.map((d) => d.id),
      cardIds: seed.cards.map((c) => c.id),
      reviewIds: (seed.reviews ?? []).map((r) => r.id),
    }),
    now,
    now,
  );

  db.exec("COMMIT");
} catch (error) {
  db.exec("ROLLBACK");
  db.close();
  const hint =
    String(error?.message ?? "").includes("busy") ||
    String(error?.code ?? "").includes("BUSY")
      ? "\nTip: quit the running JabeX app, then retry."
      : "";
  console.error(`Seed failed: ${error.message}${hint}`);
  process.exit(1);
}

db.close();

console.log(
  `Seeded ${seed.decks.length} decks, ${seed.cards.length} cards, ${(seed.reviews ?? []).length} reviews`,
);
console.log(`Profile: ${profile.display_name} (${profile.id})`);
console.log(`Data dir: ${root}`);
console.log("Remove with: pnpm db:unseed");
