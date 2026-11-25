// Chart → ChartData. Burç, derece, retro, yanma ve vargottama Rust'tan hazır gelir
// (`Chart.vargas`, `Chart.grahas`); arayüz hiçbir boylamdan burç türetmez (spec §4).
import type { Chart, Sign, Varga } from "../bindings";
import type { ChartData, ChartPlanet } from "./types";

/** 0 = Mesha (Koç) … 11 = Meena (Balık); bindings `Sign` sırası. */
export const SIGNS = [
  "Aries",
  "Taurus",
  "Gemini",
  "Cancer",
  "Leo",
  "Virgo",
  "Libra",
  "Scorpio",
  "Sagittarius",
  "Capricorn",
  "Aquarius",
  "Pisces",
] as const satisfies readonly Sign[];

/** `Varga::ALL` sırası (Shodasavarga). */
export const VARGAS = ["D1", "D2", "D3", "D4", "D7", "D9", "D10", "D12", "D16", "D20", "D24", "D27", "D30", "D40", "D45", "D60"] as const satisfies readonly Varga[];

/** Varga adları (Sanskritçe terim; dile göre değişmez). */
export const VARGA_NAMES: Record<Varga, string> = {
  D1: "Rasi",
  D2: "Hora",
  D3: "Drekkana",
  D4: "Chaturthamsa",
  D7: "Saptamsa",
  D9: "Navamsa",
  D10: "Dasamsa",
  D12: "Dwadasamsa",
  D16: "Shodasamsa",
  D20: "Vimsamsa",
  D24: "Chaturvimsamsa",
  D27: "Bhamsa",
  D30: "Trimsamsa",
  D40: "Khavedamsa",
  D45: "Akshavedamsa",
  D60: "Shashtiamsa",
};

export function isVarga(v: unknown): v is Varga {
  return typeof v === "string" && (VARGAS as readonly string[]).includes(v);
}

export function signIndex(sign: Sign): number {
  return (SIGNS as readonly Sign[]).indexOf(sign);
}

/**
 * Bir vargayı çizim girdisine çevirir; varga haritada yoksa `null`. Retro her vargada,
 * yanma yalnızca D1'de, vargottama (D1 ve D9 burcu aynı) D1 ve D9'da gösterilir.
 * `showLagna: false` (saat bilinmiyor) lagnayı gizler: 1. ev Koç çizilir (spec §7).
 */
export function chartDataFromChart(chart: Chart, varga: Varga, { showLagna }: { showLagna: boolean }): ChartData | null {
  const v = chart.vargas.find((x) => x.varga === varga);
  if (!v) return null;
  const grahas = new Map(chart.grahas.map((g) => [g.graha, g]));
  const planets: ChartPlanet[] = v.placements.map((p) => {
    const g = grahas.get(p.graha);
    return {
      id: p.graha,
      sign: signIndex(p.sign),
      // `null`: Rust'ta sonlu olmayan değer (NaN); çizim bu gezegeni açıkça atlar.
      degInSign: p.deg ?? Number.NaN,
      retrograde: g?.retrograde ?? false,
      combust: varga === "D1" && (g?.combust ?? false),
      vargottama: (varga === "D1" || varga === "D9") && (g?.vargottama ?? false),
    };
  });
  return { ascSign: showLagna ? signIndex(v.lagna_sign) : null, planets };
}
