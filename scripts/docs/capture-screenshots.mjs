#!/usr/bin/env node
/**
 * Capture README / docs screenshots via `pnpm tauri:dev` + WebView2 CDP.
 *
 * Safety: always uses an isolated directory under `<repo>/.tmp/docs-capture-data`
 * (never `%APPDATA%` / the real user database). Seed data comes from
 * `scripts/dev/seed-data.json` into demo profiles Learner + Study only.
 *
 * Usage (repo root):
 *   pnpm docs:screenshots
 */
import { spawn } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(join(__dirname, "..", ".."));
const OUT_DIR = join(ROOT, "docs", "screenshots");
const DATA_DIR = resolve(join(ROOT, ".tmp", "docs-capture-data"));
const DEBUG_PORT = 9229;
const VIEWPORT = { width: 1440, height: 900 };
const DECK_ENGLISH = "11111111-1111-4111-8111-111111111101";
const DECK_PROGRAMMING = "11111111-1111-4111-8111-111111111103";

const REQUIRED = [
  "01-app-shell.png",
  "02-dashboard.png",
  "03-decks.png",
  "04-card-editor.png",
  "05-review-session.png",
  "06-review-result.png",
  "07-search.png",
  "08-profiles.png",
  "09-backup-export.png",
  "10-backup-import.png",
  "11-settings.png",
  "12-theme-switch.png",
  "13-language-switch.png",
  "14-tray.png",
  "15-empty-states.png",
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function assertIsolatedDataDir() {
  const tmpRoot = resolve(join(ROOT, ".tmp"));
  if (!DATA_DIR.startsWith(tmpRoot + "\\") && !DATA_DIR.startsWith(tmpRoot + "/")) {
    throw new Error(
      `Refusing to capture: DATA_DIR must live under .tmp/\n  got: ${DATA_DIR}`,
    );
  }
  const appData = process.env.APPDATA
    ? resolve(process.env.APPDATA)
    : null;
  if (appData && DATA_DIR.toLowerCase().startsWith(appData.toLowerCase())) {
    throw new Error("Refusing to capture: DATA_DIR resolves under APPDATA");
  }
  console.log(`Isolated data dir: ${DATA_DIR}`);
}

function killJabexTree() {
  for (const image of ["jabex.exe", "JabeX.exe"]) {
    try {
      spawn("taskkill", ["/IM", image, "/F"], {
        stdio: "ignore",
        windowsHide: true,
      });
    } catch {
      // ignore
    }
  }
}

async function waitForHttp(url, timeoutMs = 180000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok || res.status === 404) return;
    } catch {
      // retry
    }
    await sleep(500);
  }
  throw new Error(`Timeout waiting for ${url}`);
}

async function waitForCdp(timeoutMs = 180000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      if ((await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/version`)).ok) {
        return;
      }
    } catch {
      // retry
    }
    await sleep(400);
  }
  throw new Error(`CDP not ready on ${DEBUG_PORT}`);
}

function startTauriDev() {
  mkdirSync(DATA_DIR, { recursive: true });
  const child = spawn("pnpm", ["tauri:dev"], {
    cwd: ROOT,
    env: {
      ...process.env,
      JABEX_DATA_DIR: DATA_DIR,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${DEBUG_PORT} --remote-allow-origins=*`,
      BROWSER: "none",
    },
    stdio: ["ignore", "pipe", "pipe"],
    shell: false,
  });
  child.stdout.on("data", (d) => process.stdout.write(d));
  child.stderr.on("data", (d) => process.stderr.write(d));
  child.on("error", (err) => {
    console.error("Failed to start pnpm tauri:dev:", err);
  });
  return child;
}

async function stopTauri(child) {
  if (child?.pid) {
    try {
      spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], {
        stdio: "ignore",
        windowsHide: true,
      });
    } catch {
      // ignore
    }
  }
  killJabexTree();
  await sleep(1500);
}

/** WebView2 rejects Playwright page.evaluate Origin — use CDP Runtime.evaluate. */
async function cdpEval(page, expression, { awaitPromise = true } = {}) {
  const session = await page.context().newCDPSession(page);
  const { result, exceptionDetails } = await session.send("Runtime.evaluate", {
    expression,
    awaitPromise,
    returnByValue: true,
    userGesture: true,
  });
  if (exceptionDetails) {
    throw new Error(
      exceptionDetails.exception?.description ||
        exceptionDetails.text ||
        "CDP evaluate failed",
    );
  }
  return result?.value;
}

async function invoke(page, cmd, args) {
  const payload = JSON.stringify({ cmd, args });
  return cdpEval(
    page,
    `(() => {
      const { cmd, args } = ${payload};
      const internals = window.__TAURI_INTERNALS__;
      if (!internals?.invoke) throw new Error("Tauri internals missing");
      return args === undefined
        ? internals.invoke(cmd)
        : internals.invoke(cmd, args);
    })()`,
  );
}

async function setWindowSize(page) {
  const session = await page.context().newCDPSession(page);
  try {
    const { targetInfo } = await session.send("Target.getTargetInfo");
    const { windowId } = await session.send("Browser.getWindowForTarget", {
      targetId: targetInfo.targetId,
    });
    await session.send("Browser.setWindowBounds", {
      windowId,
      bounds: {
        width: VIEWPORT.width,
        height: VIEWPORT.height,
        windowState: "normal",
      },
    });
  } catch (err) {
    console.warn("Browser.setWindowBounds unavailable:", err.message || err);
  }
  await session.send("Emulation.setDeviceMetricsOverride", {
    width: VIEWPORT.width,
    height: VIEWPORT.height,
    deviceScaleFactor: 1,
    mobile: false,
  });
}

async function waitForAppReady(page) {
  const started = Date.now();
  while (Date.now() - started < 120000) {
    const body = await cdpEval(
      page,
      "document.body?.innerText?.slice(0,200) || ''",
      { awaitPromise: false },
    );
    const hasTauri = await cdpEval(
      page,
      `"__TAURI_INTERNALS__" in window && typeof window.__TAURI_INTERNALS__.invoke === "function"`,
      { awaitPromise: false },
    );
    if (
      hasTauri &&
      body &&
      !/can't reach|ERR_CONNECTION|refused to connect/i.test(body)
    ) {
      const main = await cdpEval(
        page,
        `Boolean(document.querySelector("#main-content") || document.querySelector("#initial-profile-name"))`,
        { awaitPromise: false },
      );
      if (main) {
        await sleep(600);
        return;
      }
    }
    await sleep(400);
  }
  throw new Error("App UI did not become ready");
}

async function waitForLocale(page, locale) {
  const dir = locale === "fa" ? "rtl" : "ltr";
  for (let i = 0; i < 40; i += 1) {
    const ok = await cdpEval(
      page,
      `document.documentElement.lang === ${JSON.stringify(locale)} && document.documentElement.dir === ${JSON.stringify(dir)}`,
      { awaitPromise: false },
    );
    if (ok) {
      await sleep(250);
      return;
    }
    await sleep(150);
  }
  throw new Error(`Locale did not settle to ${locale}`);
}

async function waitForTheme(page, theme) {
  for (let i = 0; i < 40; i += 1) {
    const dark = await cdpEval(
      page,
      `document.documentElement.classList.contains("dark")`,
      { awaitPromise: false },
    );
    if (theme === "dark" ? dark : !dark) {
      await sleep(250);
      return;
    }
    await sleep(150);
  }
  throw new Error(`Theme did not settle to ${theme}`);
}

async function waitForPath(page, pathPart) {
  for (let i = 0; i < 40; i += 1) {
    const href = await cdpEval(page, "location.pathname + location.search", {
      awaitPromise: false,
    });
    if (typeof href !== "string") {
      await sleep(150);
      continue;
    }
    const matched =
      !pathPart || pathPart === "/"
        ? href === "/" || href.startsWith("/?") || href === ""
        : href.includes(pathPart);
    if (matched) {
      await sleep(350);
      return;
    }
    await sleep(150);
  }
}

async function connect() {
  await waitForHttp("http://localhost:3000");
  await waitForCdp();
  const browser = await chromium.connectOverCDP(
    `http://127.0.0.1:${DEBUG_PORT}`,
  );
  const context = browser.contexts()[0];
  if (!context) throw new Error("No browser context from CDP");
  const page =
    context.pages().find((p) => /localhost:3000|tauri\.localhost/i.test(p.url())) ??
    context.pages()[0];
  if (!page) throw new Error("No page from CDP");
  await waitForAppReady(page);
  await setWindowSize(page);
  return { browser, page };
}

async function completeSetup(page) {
  const pending = await invoke(page, "is_initial_profile_setup_pending");
  if (!pending) return;
  await page.locator("#initial-profile-name").fill("Learner");
  await page.getByRole("button", { name: /ادامه|Continue/i }).click();
  for (let i = 0; i < 50; i += 1) {
    if (!(await invoke(page, "is_initial_profile_setup_pending"))) break;
    await sleep(200);
  }
  await sleep(400);
}

async function ensureDemoProfiles(page) {
  let profiles = await invoke(page, "list_profiles");
  if (!profiles.some((p) => p.displayName === "Study")) {
    await invoke(page, "create_profile", {
      input: { displayName: "Study", avatar: null },
    });
  }
  profiles = await invoke(page, "list_profiles");
  const learner = profiles.find((p) => p.displayName === "Learner");
  if (!learner) throw new Error("Learner profile missing after setup");
  await invoke(page, "switch_profile", { id: learner.id });
}

function runSeed() {
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      [join(ROOT, "scripts", "dev", "seed.mjs"), "--profile", "Learner"],
      {
        cwd: ROOT,
        env: { ...process.env, JABEX_DATA_DIR: DATA_DIR },
        stdio: "inherit",
      },
    );
    child.on("exit", (code) =>
      code === 0 ? resolve() : reject(new Error(`db:seed failed (${code})`)),
    );
  });
}

async function setLocale(page, locale) {
  await page
    .getByRole("button", { name: locale === "fa" ? /فارسی/i : /English/i })
    .first()
    .click();
  await waitForLocale(page, locale);
}

async function setTheme(page, theme) {
  const labels =
    theme === "dark"
      ? /تیره|Dark/i
      : theme === "light"
        ? /روشن|Light/i
        : /سیستم|System/i;
  await page.getByRole("button", { name: labels }).first().click();
  // next-themes may keep "system"; force class for deterministic screenshots
  await cdpEval(
    page,
    `(() => {
      localStorage.setItem("theme", ${JSON.stringify(theme)});
      const root = document.documentElement;
      if (${JSON.stringify(theme)} === "dark") {
        root.classList.add("dark");
        root.style.colorScheme = "dark";
      } else {
        root.classList.remove("dark");
        root.style.colorScheme = "light";
      }
    })()`,
    { awaitPromise: false },
  );
  await waitForTheme(page, theme);
}

async function shot(page, name) {
  await sleep(350);
  await page.screenshot({
    path: join(OUT_DIR, name),
    type: "png",
    animations: "disabled",
  });
  console.log(`  ✓ ${name}`);
}

async function nav(page, href) {
  await page.locator(`a[href="${href}"]`).first().click();
  const pathPart = href.split("?")[0] || href;
  await waitForPath(page, pathPart === "/" ? "" : pathPart);
  await sleep(500);
}

async function go(page, path) {
  const origin = await cdpEval(page, "location.origin", { awaitPromise: false });
  await cdpEval(
    page,
    `window.location.assign(${JSON.stringify(origin + path)})`,
    { awaitPromise: false },
  );
  await waitForAppReady(page);
  await setWindowSize(page);
  await sleep(600);
}

async function openDeckCards(page) {
  await nav(page, "/decks");
  const link = page.locator(`a[href*="${DECK_ENGLISH}"]`).first();
  if (await link.count()) await link.click();
  else await page.getByText("واژگان انگلیسی پایه").first().click();
  await sleep(900);
}

async function main() {
  assertIsolatedDataDir();
  mkdirSync(OUT_DIR, { recursive: true });
  killJabexTree();
  await sleep(800);

  if (existsSync(DATA_DIR)) {
    rmSync(DATA_DIR, { recursive: true, force: true });
  }
  mkdirSync(DATA_DIR, { recursive: true });

  console.log("Phase 1 — bootstrap demo profiles…");
  let child = startTauriDev();
  try {
    const { page } = await connect();
    await completeSetup(page);
    await ensureDemoProfiles(page);
    // Confirm we never wrote outside .tmp
    const resolved = realpathSync(DATA_DIR);
    if (!resolved.startsWith(realpathSync(join(ROOT, ".tmp")))) {
      throw new Error(`Data escaped isolation: ${resolved}`);
    }
  } finally {
    await stopTauri(child);
  }

  console.log("Phase 2 — seed demo decks/cards…");
  await runSeed();

  console.log("Phase 3 — capture…");
  child = startTauriDev();
  try {
    const { page } = await connect();
    await setLocale(page, "fa");
    await setTheme(page, "light");

    const profiles = await invoke(page, "list_profiles");
    const learner = profiles.find((p) => p.displayName === "Learner");
    if (!learner) throw new Error("Learner missing before capture");
    await invoke(page, "switch_profile", { id: learner.id });
    await sleep(700);

    await nav(page, "/");
    await shot(page, "01-app-shell.png");

    await cdpEval(
      page,
      `document.querySelector("#main-content")?.scrollTo({ top: 220, behavior: "instant" })`,
      { awaitPromise: false },
    );
    await sleep(300);
    await shot(page, "02-dashboard.png");
    await cdpEval(
      page,
      `document.querySelector("#main-content")?.scrollTo({ top: 0, behavior: "instant" })`,
      { awaitPromise: false },
    );

    await nav(page, "/decks");
    await shot(page, "03-decks.png");

    await openDeckCards(page);
    const apple = page.getByRole("button").filter({ hasText: /apple/i });
    if (await apple.count()) await apple.first().click();
    else await page.getByText("apple", { exact: true }).first().click();
    await page.getByRole("dialog").waitFor({ state: "visible", timeout: 15000 });
    await sleep(500);
    await shot(page, "04-card-editor.png");
    await page.keyboard.press("Escape");
    await sleep(300);

    // Programming deck has clear English faces for review screenshots
    await go(page, `/review?deckId=${DECK_PROGRAMMING}`);
    await sleep(800);
    await shot(page, "05-review-session.png");
    const reveal = page.getByRole("button", {
      name: /نمایش پاسخ|Reveal|Show answer|پاسخ/i,
    });
    if (await reveal.count()) await reveal.first().click();
    else await page.keyboard.press("Space");
    await sleep(600);
    await shot(page, "06-review-result.png");

    await nav(page, "/");
    await page.keyboard.press("Control+k");
    await sleep(450);
    await page.locator("input").first().fill("apple");
    await sleep(900);
    await shot(page, "07-search.png");
    await page.keyboard.press("Escape");
    await sleep(250);

    await page
      .getByRole("button")
      .filter({ hasText: /Learner|Study/i })
      .first()
      .click();
    await sleep(500);
    await shot(page, "08-profiles.png");
    await page.keyboard.press("Escape");
    await sleep(250);

    const backupPath = join(DATA_DIR, "jabex-learner-demo.jbx");
    await invoke(page, "export_backup", {
      input: { outputPath: backupPath },
    });

    await nav(page, "/settings");
    await cdpEval(
      page,
      `document.querySelector("#main-content")?.scrollTo({ top: 0, behavior: "instant" })`,
      { awaitPromise: false },
    );
    await sleep(400);
    await shot(page, "11-settings.png");
    await shot(page, "14-tray.png");

    await page.locator("#backup").scrollIntoViewIfNeeded();
    await sleep(400);
    await page.getByRole("button", { name: /خروجی \.jbx|Export/i }).focus();
    await sleep(250);
    await shot(page, "09-backup-export.png");
    await page.getByRole("button", { name: /ورود \.jbx|Import/i }).focus();
    await sleep(250);
    await shot(page, "10-backup-import.png");

    await setTheme(page, "dark");
    await nav(page, "/");
    await sleep(500);
    await shot(page, "12-theme-switch.png");
    await setTheme(page, "light");

    await setLocale(page, "en");
    await nav(page, "/");
    await sleep(500);
    await shot(page, "13-language-switch.png");
    await setLocale(page, "fa");

    const list = await invoke(page, "list_profiles");
    const study = list.find((p) => p.displayName === "Study");
    if (!study) throw new Error("Study profile missing for empty-state shot");
    await invoke(page, "switch_profile", { id: study.id });
    await sleep(800);
    await nav(page, "/decks");
    await sleep(700);
    await shot(page, "15-empty-states.png");

    const missing = REQUIRED.filter((n) => !existsSync(join(OUT_DIR, n)));
    if (missing.length) {
      throw new Error(`Missing screenshots: ${missing.join(", ")}`);
    }

    writeFileSync(
      join(OUT_DIR, "manifest.json"),
      JSON.stringify(
        {
          capturedAt: new Date().toISOString(),
          viewport: VIEWPORT,
          dataDir: ".tmp/docs-capture-data",
          isolated: true,
          captured: REQUIRED,
          defaults: { locale: "fa", direction: "rtl", theme: "light" },
          variants: {
            "12-theme-switch.png": { theme: "dark", locale: "fa" },
            "13-language-switch.png": { theme: "light", locale: "en", direction: "ltr" },
            "15-empty-states.png": { profile: "Study", locale: "fa" },
          },
          notes: [
            "Never writes to the real user APPDATA database",
            "09/10 = Settings backup panel (native file dialogs not captured)",
            "14 = in-app tray preferences (OS tray chrome varies)",
          ],
        },
        null,
        2,
      ),
    );
    console.log(`\nCaptured ${REQUIRED.length} → docs/screenshots/`);
    console.log(`Data stayed under: ${DATA_DIR}`);
  } finally {
    await stopTauri(child);
  }
}

main().catch(async (err) => {
  console.error(err);
  killJabexTree();
  process.exit(1);
});
