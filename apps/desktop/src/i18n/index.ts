import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enCommon from "@locales/en/common.json";
import frCommon from "@locales/fr/common.json";

export const SUPPORTED_LANGUAGES = ["en", "fr"] as const;
export type Language = (typeof SUPPORTED_LANGUAGES)[number];

/** Resolves a preference (`null` = follow the OS) to a supported language. */
export function resolveLanguage(preference: string | null | undefined): Language {
  const candidate = (preference ?? navigator.language ?? "en").slice(0, 2).toLowerCase();
  return (SUPPORTED_LANGUAGES as readonly string[]).includes(candidate)
    ? (candidate as Language)
    : "en";
}

void i18n.use(initReactI18next).init({
  resources: {
    en: { common: enCommon },
    fr: { common: frCommon },
  },
  lng: resolveLanguage(null),
  fallbackLng: "en",
  defaultNS: "common",
  interpolation: { escapeValue: false },
});

export default i18n;
