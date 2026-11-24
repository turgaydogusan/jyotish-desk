// i18next kurulumu: Türkçe varsayılan, İngilizce ve Fransızca (spec §6). Kaynaklar
// pakete gömülüdür; başlatma eşzamanlıdır, ilk çizimde anahtar görünmez.
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import { en } from "./en";
import { fr } from "./fr";
import { tr } from "./tr";

export const LANGUAGES = ["tr", "en", "fr"] as const;
export type Language = (typeof LANGUAGES)[number];
export const LANGUAGE_STORAGE_KEY = "jyotish-desk.language.v1";
export const DEFAULT_LANGUAGE: Language = "tr";

export function isLanguage(v: unknown): v is Language {
  return typeof v === "string" && (LANGUAGES as readonly string[]).includes(v);
}

export function loadLanguage(storage: Storage = window.localStorage): Language {
  try {
    const v = storage.getItem(LANGUAGE_STORAGE_KEY);
    return isLanguage(v) ? v : DEFAULT_LANGUAGE;
  } catch {
    return DEFAULT_LANGUAGE;
  }
}

export const i18n = i18next.createInstance();

void i18n.use(initReactI18next).init({
  resources: { tr: { translation: tr }, en: { translation: en }, fr: { translation: fr } },
  lng: loadLanguage(),
  fallbackLng: DEFAULT_LANGUAGE,
  supportedLngs: [...LANGUAGES],
  interpolation: { escapeValue: false }, // React zaten kaçışlar
  initAsync: false,
});
document.documentElement.lang = i18n.language;

export async function setLanguage(lng: Language, storage: Storage = window.localStorage): Promise<void> {
  try {
    storage.setItem(LANGUAGE_STORAGE_KEY, lng);
  } catch (e) {
    console.warn("language preference not saved", e);
  }
  document.documentElement.lang = lng;
  await i18n.changeLanguage(lng);
}

export function currentLanguage(): Language {
  return isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
}
