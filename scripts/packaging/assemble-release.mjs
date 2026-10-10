#!/usr/bin/env node
/**
 * Assemble public Windows release artifacts into ./release/:
 *
 *   release/
 *     JabeX_<version>_x64-setup.exe
 *     JabeX_<version>_x64_en-US.msi
 *     JabeX-<version>-portable.zip
 *     SHA256SUMS.txt
 *
 * Prerequisites:
 *   - `pnpm tauri:build` (NSIS + MSI under target/release/bundle)
 *   - release binary at target/release/JabeX.exe (or jabex.exe)
 *
 * Does not publish to GitHub, sign binaries, or enable an updater.
 */
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(__dirname, "..", "..");
const tauriDir = join(repoRoot, "apps", "desktop", "src-tauri");
const bundleDir = join(tauriDir, "target", "release", "bundle");
const releaseDir = join(repoRoot, "release");

const rootPkg = JSON.parse(
  readFileSync(join(repoRoot, "package.json"), "utf8"),
);
const version = rootPkg.version || "0.0.0";

const nsisName = `JabeX_${version}_x64-setup.exe`;
const msiName = `JabeX_${version}_x64_en-US.msi`;
const portableZipName = `JabeX-${version}-portable.zip`;
const portableFolderName = `JabeX-${version}-portable`;

function sha256File(filePath) {
  const hash = createHash("sha256");
  hash.update(readFileSync(filePath));
  return hash.digest("hex");
}

function requireFile(path, hint) {
  if (!existsSync(path)) {
    console.error(`Missing: ${path}`);
    if (hint) console.error(hint);
    process.exit(1);
  }
}

function run(command, args, cwd) {
  console.log(`> ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    shell: false,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function zipPortableFolder(portableDir, zipPath) {
  if (process.platform === "win32") {
    const escape = (p) => p.replace(/'/g, "''");
    const ps = `
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath '${escape(zipPath)}') {
  Remove-Item -LiteralPath '${escape(zipPath)}' -Force
}
Compress-Archive -Path '${escape(portableDir)}' -DestinationPath '${escape(zipPath)}' -Force
`;
    const result = spawnSync(
      "powershell.exe",
      ["-NoProfile", "-Command", ps],
      { stdio: "inherit" },
    );
    if (result.status !== 0) {
      process.exit(result.status ?? 1);
    }
    return;
  }

  const parent = dirname(portableDir);
  const folder = portableFolderName;
  const result = spawnSync(
    "tar",
    ["-a", "-cf", zipPath, "-C", parent, folder],
    { stdio: "inherit" },
  );
  if (result.status !== 0) {
    console.error(
      "Failed to create zip. On non-Windows, install tar with zip support or zip manually.",
    );
    process.exit(result.status ?? 1);
  }
}

function main() {
  const nsisSrc = join(bundleDir, "nsis", nsisName);
  const msiSrc = join(bundleDir, "msi", msiName);

  requireFile(
    nsisSrc,
    "Run: pnpm tauri:build\nExpected NSIS under apps/desktop/src-tauri/target/release/bundle/nsis/",
  );
  requireFile(
    msiSrc,
    "Run: pnpm tauri:build\nExpected MSI under apps/desktop/src-tauri/target/release/bundle/msi/",
  );

  run("node", [join(__dirname, "pack-portable.mjs")], repoRoot);
  const portableDir = join(repoRoot, "dist", portableFolderName);
  requireFile(
    join(portableDir, "JabeX.exe"),
    "Portable pack did not produce JabeX.exe",
  );

  rmSync(releaseDir, { recursive: true, force: true });
  mkdirSync(releaseDir, { recursive: true });

  const nsisOut = join(releaseDir, nsisName);
  const msiOut = join(releaseDir, msiName);
  const zipOut = join(releaseDir, portableZipName);

  copyFileSync(nsisSrc, nsisOut);
  copyFileSync(msiSrc, msiOut);
  zipPortableFolder(portableDir, zipOut);

  const artifacts = [nsisName, msiName, portableZipName];
  const lines = [];
  for (const name of artifacts) {
    const full = join(releaseDir, name);
    const digest = sha256File(full);
    const size = statSync(full).size;
    lines.push(`${digest}  ${name}`);
    console.log(`${name}  ${size} bytes  sha256=${digest}`);
  }

  const sumsPath = join(releaseDir, "SHA256SUMS.txt");
  writeFileSync(sumsPath, `${lines.join("\n")}\n`, "utf8");
  console.log(`\nWrote ${sumsPath}`);
  console.log(`\nRelease folder ready:\n  ${releaseDir}\n`);
}

main();
