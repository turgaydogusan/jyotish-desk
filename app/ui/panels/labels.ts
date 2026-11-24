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
