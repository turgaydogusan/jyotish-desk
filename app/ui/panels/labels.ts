import type { TFunction } from "i18next";
import { GRAHA_IDS, type ChartLabels, type GrahaId } from "../chart/types";

export function grahaAbbreviations(t: TFunction): Record<GrahaId, string> {
  return Object.fromEntries(GRAHA_IDS.map((id) => [id, t(`graha.${id}`)])) as Record<GrahaId, string>;
}

export function signAbbreviations(t: TFunction): string[] {
  return Array.from({ length: 12 }, (_, i) => t(`sign.${i}`));
}

export function chartLabels(t: TFunction, varga: string): ChartLabels {
  return {
    title: t("chart.title", { varga }),
    graha: grahaAbbreviations(t),
    signs: signAbbreviations(t),
    lagna: t("chart.lagna"),
    house: (house, sign) => t("chart.house", { house, sign }),
    retro: t("chart.retro"),
    combust: t("chart.combust"),
    vargottama: t("chart.vargottama"),
  };
}

/** Nakshatra adları, 1 = Ashwini … 27 = Revati (Sanskritçe terim; dile göre değişmez). */
export const NAKSHATRA_NAMES = [
  "Ashwini",
  "Bharani",
  "Krittika",
  "Rohini",
  "Mrigashira",
  "Ardra",
  "Punarvasu",
  "Pushya",
  "Ashlesha",
  "Magha",
  "Purva Phalguni",
  "Uttara Phalguni",
  "Hasta",
  "Chitra",
  "Swati",
  "Vishakha",
  "Anuradha",
  "Jyeshtha",
  "Mula",
  "Purva Ashadha",
  "Uttara Ashadha",
  "Shravana",
  "Dhanishta",
  "Shatabhisha",
  "Purva Bhadrapada",
  "Uttara Bhadrapada",
  "Revati",
] as const;

/** Tithi adları, paksha içi sıra 1–14; 15. tithi Shukla'da Purnima, Krishna'da Amavasya'dır. */
export const TITHI_NAMES = [
  "Pratipada",
  "Dwitiya",
  "Tritiya",
  "Chaturthi",
  "Panchami",
  "Shashthi",
  "Saptami",
  "Ashtami",
  "Navami",
  "Dashami",
  "Ekadashi",
  "Dwadashi",
  "Trayodashi",
  "Chaturdashi",
] as const;

/** Tithinin (1–30) paksha içi sırası, 1–15. */
export function tithiInPaksha(no: number): number {
  return ((no - 1) % 15) + 1;
}

/** Tithinin (1–30) adı; aralık dışıysa numaranın kendisi. */
export function tithiName(no: number): string {
  if (!Number.isInteger(no) || no < 1 || no > 30) return String(no);
  if (no === 15) return "Purnima";
  if (no === 30) return "Amavasya";
  return TITHI_NAMES[tithiInPaksha(no) - 1];
}

/** Nitya yoga adları, 1 = Vishkambha … 27 = Vaidhriti. */
export const NITYA_YOGA_NAMES = [
  "Vishkambha",
  "Priti",
  "Ayushman",
  "Saubhagya",
  "Shobhana",
  "Atiganda",
  "Sukarma",
  "Dhriti",
  "Shula",
  "Ganda",
  "Vriddhi",
  "Dhruva",
  "Vyaghata",
  "Harshana",
  "Vajra",
  "Siddhi",
  "Vyatipata",
  "Variyan",
  "Parigha",
  "Shiva",
  "Siddha",
  "Sadhya",
  "Shubha",
  "Shukla",
  "Brahma",
  "Indra",
  "Vaidhriti",
] as const;

export function nityaYogaName(n: number): string {
  return NITYA_YOGA_NAMES[n - 1] ?? String(n);
}

/** Vara adları (Sanskritçe terim; dile göre değişmez). Günün çevirisi i18n'dedir. */
export const VARA_NAMES = {
  Sunday: "Ravivara",
  Monday: "Somavara",
  Tuesday: "Mangalavara",
  Wednesday: "Budhavara",
  Thursday: "Guruvara",
  Friday: "Shukravara",
  Saturday: "Shanivara",
} as const;

/** 1 tabanlı nakshatra numarasının adı; aralık dışıysa numaranın kendisi. */
export function nakshatraName(n: number): string {
  return NAKSHATRA_NAMES[n - 1] ?? String(n);
}
