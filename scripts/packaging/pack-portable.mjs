#!/usr/bin/env node
/**
 * Assemble a no-installer Windows portable folder:
 *
 *   dist/JabeX-<version>-portable/
 *     JabeX.exe
 *     resources/
 *     WebView2 requirements.txt
 *     README.txt
 *
 * Prerequisites: frontend production build + `cargo build --release`
 * (or pass --build to run them).
 *
 * Does not create NSIS/MSI installers.
 */
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(__dirname, "..", "..");
const tauriDir = join(repoRoot, "apps", "desktop", "src-tauri");
const frontendDir = join(repoRoot, "apps", "desktop", "frontend");
const templateDir = join(__dirname, "portable");

const rootPkg = JSON.parse(
  readFileSync(join(repoRoot, "package.json"), "utf8"),
);
const version = rootPkg.version || "0.0.0";
const folderName = `JabeX-${version}-portable`;
const outDir = join(repoRoot, "dist", folderName);

const shouldBuild = process.argv.includes("--build");

function resolveCommand(command) {
  if (process.platform !== "win32") return command;
  if (command === "pnpm") return "pnpm.cmd";
  if (command === "cargo") return "cargo.exe";
  return command;
}

function run(command, args, cwd) {
  const resolved = resolveCommand(command);
  console.log(`> ${resolved} ${args.join(" ")}`);
  const result = spawnSync(resolved, args, {
    cwd,
    stdio: "inherit",
    shell: false,
    env: process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function main() {
  if (shouldBuild) {
    run("pnpm", ["build"], repoRoot);
    run("cargo", ["build", "--release"], tauriDir);
  }

  const releaseExe = join(tauriDir, "target", "release", "JabeX.exe");
  const legacyExe = join(tauriDir, "target", "release", "jabex.exe");
  const sourceExe = existsSync(releaseExe)
    ? releaseExe
    : existsSync(legacyExe)
      ? legacyExe
      : null;

  if (!sourceExe) {
    console.error(
      "Missing release binary. Build first:\n" +
        "  pnpm build\n" +
        "  cd apps/desktop/src-tauri && cargo build --release\n" +
        "Or: pnpm portable:pack -- --build",
    );
    process.exit(1);
  }

  if (!existsSync(join(frontendDir, "out", "index.html"))) {
    console.warn(
      "Warning: apps/desktop/frontend/out is missing. " +
        "Release binary may be stale; run `pnpm build` before cargo build --release.",
    );
  }

  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  mkdirSync(join(outDir, "resources"), { recursive: true });

  copyFileSync(sourceExe, join(outDir, "JabeX.exe"));
  // Keep a consistently named artifact under target/release as documented.
  if (sourceExe !== releaseExe) {
    copyFileSync(sourceExe, releaseExe);
  }

  cpSync(join(templateDir, "resources"), join(outDir, "resources"), {
    recursive: true,
  });
  copyFileSync(
    join(templateDir, "WebView2 requirements.txt"),
    join(outDir, "WebView2 requirements.txt"),
  );
  copyFileSync(join(templateDir, "README.txt"), join(outDir, "README.txt"));

  console.log(`\nPortable folder ready:\n  ${outDir}\n`);
  console.log("Contents:");
  console.log(`  JabeX.exe`);
  console.log(`  resources/`);
  console.log(`  WebView2 requirements.txt`);
  console.log(`  README.txt`);
}

main();
