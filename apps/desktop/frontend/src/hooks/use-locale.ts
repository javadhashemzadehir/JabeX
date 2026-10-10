"use client";

import { useLocaleStore } from "@/lib/i18n/locale-store";

export function useLocale() {
  const locale = useLocaleStore((s) => s.locale);
  const dictionary = useLocaleStore((s) => s.dictionary);
  const direction = useLocaleStore((s) => s.direction);
  const setLocale = useLocaleStore((s) => s.setLocale);
  const hydrated = useLocaleStore((s) => s.hydrated);

  return { locale, dictionary, direction, setLocale, hydrated };
}

export function useTranslations() {
  return useLocaleStore((s) => s.dictionary);
}
