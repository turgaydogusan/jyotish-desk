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

/** 1 tabanlı nakshatra numarasının adı; aralık dışıysa numaranın kendisi. */
export function nakshatraName(n: number): string {
  return NAKSHATRA_NAMES[n - 1] ?? String(n);
}
