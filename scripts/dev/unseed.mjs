#!/usr/bin/env node
/**
 * Remove mock seed data (or purge all study rows) from a local profile.
 *
 * Usage:
 *   pnpm db:unseed              # delete only seed fixture rows
 *   pnpm db:unseed -- --purge   # delete ALL decks/cards/reviews in the profile
 *   pnpm db:unseed -- --profile "جواد"
 */
import {
  SEED_MARKER_KEY,
  loadSeedData,
  openAppDatabase,
  openProfileDatabase,
  parseArgs,
  placeholders,
  resolveActiveProfile,
  resolveAppDataRoot,
  seedIds,
} from "./lib/db.mjs";

const { flags, options } = parseArgs();
const purgeAll = flags.has("purge") || flags.has("all");
const root = resolveAppDataRoot();

const appDb = openAppDatabase(root);
const profile = resolveActiveProfile(appDb, options);
appDb.close();

const db = openProfileDatabase(root, profile.id);

function busyHint(error) {
  const text = `${error?.message ?? ""} ${error?.code ?? ""}`;
  return /busy/i.test(text)
    ? "\nTip: quit the running JabeX app, then retry."
    : "";
}

try {
  db.exec("BEGIN");

  if (purgeAll) {
    const reviewCount = db.prepare("SELECT COUNT(*) AS n FROM reviews").get().n;
    const cardCount = db
      .prepare("SELECT COUNT(*) AS n FROM cards WHERE deleted_at IS NULL")
      .get().n;
    const deckCount = db
      .prepare("SELECT COUNT(*) AS n FROM decks WHERE deleted_at IS NULL")
      .get().n;

    db.exec("DELETE FROM reviews");
    db.exec("DELETE FROM cards");
    db.exec("DELETE FROM decks");
    db.prepare("DELETE FROM settings WHERE key = ?").run(SEED_MARKER_KEY);

    db.exec("COMMIT");
    db.close();

    console.log(
      `Purged all study data: ${deckCount} decks, ${cardCount} cards, ${reviewCount} reviews`,
    );
    console.log(`Profile: ${profile.display_name} (${profile.id})`);
    process.exit(0);
  }

  const seed = loadSeedData();
  const { deckIds, cardIds, reviewIds } = seedIds(seed);

  let deletedReviews = 0;
  let deletedCards = 0;
  let deletedDecks = 0;

  if (reviewIds.length) {
    const result = db
      .prepare(`DELETE FROM reviews WHERE id IN (${placeholders(reviewIds.length)})`)
      .run(...reviewIds);
    deletedReviews += result.changes;
  }
  // Also drop any reviews that still point at seed cards (user may have reviewed them).
  if (cardIds.length) {
    const result = db
      .prepare(`DELETE FROM reviews WHERE card_id IN (${placeholders(cardIds.length)})`)
      .run(...cardIds);
    deletedReviews += result.changes;
  }

  if (cardIds.length) {
    const result = db
      .prepare(`DELETE FROM cards WHERE id IN (${placeholders(cardIds.length)})`)
      .run(...cardIds);
    deletedCards = result.changes;
  }

  if (deckIds.length) {
    const result = db
      .prepare(`DELETE FROM decks WHERE id IN (${placeholders(deckIds.length)})`)
      .run(...deckIds);
    deletedDecks = result.changes;
  }

  db.prepare("DELETE FROM settings WHERE key = ?").run(SEED_MARKER_KEY);

  db.exec("COMMIT");
  db.close();

  console.log(
    `Removed seed data: ${deletedDecks} decks, ${deletedCards} cards, ${deletedReviews} reviews`,
  );
  console.log(`Profile: ${profile.display_name} (${profile.id})`);
  if (deletedDecks === 0 && deletedCards === 0) {
    console.log("Nothing matched — seed may already be cleared.");
  }
} catch (error) {
  try {
    db.exec("ROLLBACK");
  } catch {
    // ignore
  }
  try {
    db.close();
  } catch {
    // ignore
  }
  console.error(`Unseed failed: ${error.message}${busyHint(error)}`);
  process.exit(1);
}
