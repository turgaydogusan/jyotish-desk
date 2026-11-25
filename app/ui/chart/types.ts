// Harita çiziminin girdisi. Değerler Rust'tan hazır gelir; arayüz yalnızca çizer.
import type { Graha } from "../bindings";

/** `Graha::ALL` sırası (bindings `Graha`). */
export const GRAHA_IDS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const satisfies readonly Graha[];
export type GrahaId = (typeof GRAHA_IDS)[number];

export function isGrahaId(v: string): v is GrahaId {
  return (GRAHA_IDS as readonly string[]).includes(v);
}

export interface ChartPlanet {
  id: GrahaId;
  /** 0 = Mesha (Koç) … 11 = Meena (Balık). */
  sign: number;
  /** Burç içi derece, [0, 30). */
  degInSign: number;
  retrograde: boolean;
  combust?: boolean;
  vargottama?: boolean;
}

export interface ChartData {
  /** Lagna burcu; henüz hesaplanmıyorsa `null` (1. ev Koç kabul edilir, çağıran uyarı gösterir). */
  ascSign: number | null;
  planets: ChartPlanet[];
}

export type ChartStyle = "north" | "south" | "circle";
export const CHART_STYLES: readonly ChartStyle[] = ["north", "south", "circle"];

export function isChartStyle(v: unknown): v is ChartStyle {
  return typeof v === "string" && (CHART_STYLES as readonly string[]).includes(v);
}

/** Çizimde kullanılan metinler; çağıran i18n'den doldurur. */
export interface ChartLabels {
  /** Erişilebilir ad (svg `role="group"` + `aria-label`). */
  title: string;
  /** Graha kısaltmaları. */
  graha: Record<GrahaId, string>;
  /** 12 burç kısaltması, 0 = Mesha. */
  signs: readonly string[];
  /** Güney Hint ve daire stilinde lagna hücresindeki işaret. */
  lagna: string;
  /** Ev hücresinin erişilebilir adı. */
  house: (house: number, sign: string) => string;
  /** Erişilebilir adda `℞` yerine okunan sözcük (örn. "retro"). */
  retro: string;
  /** Erişilebilir adda `c` yerine okunan sözcük (örn. "yanık"). */
  combust: string;
  /** Erişilebilir adda alt çizgi yerine okunan sözcük (örn. "vargottama"). */
  vargottama: string;
}
