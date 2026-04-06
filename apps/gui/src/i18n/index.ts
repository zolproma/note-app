import { createContext, useContext } from "react";
import { type Locale, type Messages, messages } from "./locales";

export type { Locale, Messages };
export { messages };

const STORAGE_KEY = "ono-locale";

export function getStoredLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "en" || stored === "zh") return stored;
  } catch {}
  // Auto-detect from browser
  const lang = navigator.language.toLowerCase();
  if (lang.startsWith("zh")) return "zh";
  return "en";
}

export function setStoredLocale(locale: Locale) {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {}
}

export const I18nContext = createContext<Messages>(messages.en);

export function useI18n(): Messages {
  return useContext(I18nContext);
}
