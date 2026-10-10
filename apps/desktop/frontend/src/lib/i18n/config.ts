export const locales = ["fa", "en"] as const;

export type Locale = (typeof locales)[number];

export const DEFAULT_LOCALE: Locale = "fa";

export const localeDirections: Record<Locale, "rtl" | "ltr"> = {
  fa: "rtl",
  en: "ltr",
};

export const localeLabels: Record<Locale, string> = {
  fa: "فارسی",
  en: "English",
};

export const LOCALE_STORAGE_KEY = "jabex.locale";

export function isLocale(value: unknown): value is Locale {
  return value === "fa" || value === "en";
}

export function getDirection(locale: Locale): "rtl" | "ltr" {
  return localeDirections[locale];
}
