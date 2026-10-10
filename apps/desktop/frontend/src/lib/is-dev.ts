/** True only for Next.js / Tauri `dev` builds — never in release installers. */
export function isDevMode(): boolean {
  return process.env.NODE_ENV === "development";
}
