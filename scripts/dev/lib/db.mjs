import { DatabaseSync } from "node:sqlite";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { homedir, platform } from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
export const SEED_DATA_PATH = join(__dirname, "..", "seed-data.json");
export const SEED_MARKER_KEY = "dev.seed.marker";

/** Resolve `{app_data}/JabeX/JabeX/data` (directories crate ProjectDirs). */
export function resolveAppDataRoot() {
  if (process.env.JABEX_DATA_DIR) {
    return process.env.JABEX_DATA_DIR;
  }

  // Matches `directories::ProjectDirs::from("app", "JabeX", "JabeX").data_dir()`.
  switch (platform()) {
    case "win32":
      return join(
        process.env.APPDATA || join(homedir(), "AppData", "Roaming"),
        "JabeX",
        "JabeX",
        "data",
      );
    case "darwin":
      return join(
        homedir(),
        "Library",
        "Application Support",
        "app.JabeX.JabeX",
      );
    default:
      return join(
        process.env.XDG_DATA_HOME || join(homedir(), ".local", "share"),
        "jabex",
      );
  }
}

export function loadSeedData() {
  const raw = readFileSync(SEED_DATA_PATH, "utf8");
  const data = JSON.parse(raw);
  if (!data?.decks?.length) {
    throw new Error(`Invalid seed file: ${SEED_DATA_PATH}`);
  }
  return data;
}

export function openAppDatabase(root = resolveAppDataRoot()) {
  const path = join(root, "app.sqlite");
  if (!existsSync(path)) {
    throw new Error(
      `App database not found at ${path}\nRun the app once (pnpm tauri:dev) so a local profile exists.`,
    );
  }
  return new DatabaseSync(path);
}

export function resolveActiveProfile(appDb, { profile: profileArg } = {}) {
  if (profileArg) {
    const byId = appDb
      .prepare(
        "SELECT id, display_name FROM profiles WHERE id = ? AND deleted_at IS NULL",
      )
      .get(profileArg);
    if (byId) return byId;

    const byName = appDb
      .prepare(
        "SELECT id, display_name FROM profiles WHERE display_name = ? AND deleted_at IS NULL",
      )
      .get(profileArg);
    if (byName) return byName;

    throw new Error(`Profile not found: ${profileArg}`);
  }

  const active = appDb
    .prepare("SELECT value FROM app_meta WHERE key = ?")
    .get("active_profile_id");
  if (!active?.value) {
    throw new Error("No active profile. Open the app and create/select a profile first.");
  }

  const profile = appDb
    .prepare(
      "SELECT id, display_name FROM profiles WHERE id = ? AND deleted_at IS NULL",
    )
    .get(active.value);
  if (!profile) {
    throw new Error(`Active profile id ${active.value} is missing from the registry.`);
  }
  return profile;
}

export function openProfileDatabase(root, profileId) {
  const path = join(root, "profiles", profileId, "database.sqlite");
  if (!existsSync(path)) {
    throw new Error(`Profile database not found: ${path}`);
  }
  const db = new DatabaseSync(path);
  db.exec("PRAGMA foreign_keys = ON;");
  return db;
}

export function isoNow(offsetDays = 0) {
  const ms = Date.now() + offsetDays * 24 * 60 * 60 * 1000;
  return new Date(ms).toISOString();
}

export function seedIds(data) {
  return {
    deckIds: data.decks.map((d) => d.id),
    cardIds: data.cards.map((c) => c.id),
    reviewIds: (data.reviews ?? []).map((r) => r.id),
  };
}

export function placeholders(count) {
  return Array.from({ length: count }, () => "?").join(", ");
}

export function parseArgs(argv = process.argv.slice(2)) {
  const flags = new Set();
  const options = {};
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--profile" || arg === "-p") {
      options.profile = argv[++i];
      continue;
    }
    if (arg.startsWith("--")) {
      flags.add(arg.slice(2));
      continue;
    }
    flags.add(arg);
  }
  return { flags, options };
}
