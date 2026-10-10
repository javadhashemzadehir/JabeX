import type { Locale } from "./config";
import en from "@/locales/en/common.json";
import fa from "@/locales/fa/common.json";

export type Dictionary = typeof fa;

const dictionaries: Record<Locale, Dictionary> = {
  fa,
  en,
};

export function getDictionary(locale: Locale): Dictionary {
  return dictionaries[locale];
}
