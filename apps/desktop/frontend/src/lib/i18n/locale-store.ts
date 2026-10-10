"use client";

import { create } from "zustand";
import {
  DEFAULT_LOCALE,
  LOCALE_STORAGE_KEY,
  getDirection,
  isLocale,
  type Locale,
} from "./config";
import { getDictionary, type Dictionary } from "./dictionaries";

interface LocaleState {
  locale: Locale;
  dictionary: Dictionary;
  direction: "rtl" | "ltr";
  hydrated: boolean;
  setLocale: (locale: Locale) => void;
  hydrate: () => void;
}

function applyDocumentLocale(locale: Locale) {
  if (typeof document === "undefined") return;
  const direction = getDirection(locale);
  document.documentElement.lang = locale;
  document.documentElement.dir = direction;
}

export const useLocaleStore = create<LocaleState>((set) => ({
  locale: DEFAULT_LOCALE,
  dictionary: getDictionary(DEFAULT_LOCALE),
  direction: getDirection(DEFAULT_LOCALE),
  hydrated: false,
  setLocale: (locale) => {
    applyDocumentLocale(locale);
    try {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, locale);
    } catch {
      // Ignore storage failures (private mode, etc.)
    }
    set({
      locale,
      dictionary: getDictionary(locale),
      direction: getDirection(locale),
    });
  },
  hydrate: () => {
    let locale = DEFAULT_LOCALE;
    try {
      const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
      if (isLocale(stored)) locale = stored;
    } catch {
      // Ignore storage failures
    }
    applyDocumentLocale(locale);
    set({
      locale,
      dictionary: getDictionary(locale),
      direction: getDirection(locale),
      hydrated: true,
    });
  },
}));
