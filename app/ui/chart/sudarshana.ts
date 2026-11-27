// Sudarshana chakra için saf geometri (yol haritası §4.6): üç iç içe halka, her halkada on iki
// ev. Koordinatlar 100 × 100 kutudadır (svg viewBox). Aynı numaralı ev üç halkada da aynı
// açıdadır: 1. ev solda (daire haritasıyla aynı yön), evler saat yönünün tersine ilerler.
// Halka sırası içten dışa `SudarshanaRef::ALL` sırasıdır (lagna, Ay, Güneş).
import type { ChartCell, Point } from "./geometry";

/** Halka sayısı (Rust `SudarshanaRef::ALL`). */
export const SUDARSHANA_RING_COUNT = 3;
/**
 * Halka sınırları: göbek, sonra üç halkanın dış yarıçapı. Değerler, en dar açıdaki hücrede bile
 * iç halkaya altı, orta ve dış halkaya dört etiket sığacak biçimde seçilmiştir
 * (`sudarshana.test.ts`); daha çok graha en küçük yazı boyutunda, taşarak yazılır.
 */
export const SUDARSHANA_RADII = [5, 22, 36, 49] as const;
/** Burç numarasının yazıldığı iç şeridin kalınlığı; etiketler şeridin dışına yerleşir. */
export const SUDARSHANA_SIGN_BAND = 2.6;
/** Burç numarasının yazı boyutu. */
export const SUDARSHANA_SIGN_FONT = 2.2;
const ARC_STEPS = 8;

function polar(r: number, deg: number): Point {
  const rad = (deg * Math.PI) / 180;
  // Matematiksel açı (saat yönünün tersi, y yukarı) → ekran (y aşağı).
  return { x: 50 + r * Math.cos(rad), y: 50 - r * Math.sin(rad) };
}

/** 30°'lik halka diliminin çokgeni: dış yay ileri, iç yay geri. */
function sectorPoints(rInner: number, rOuter: number, startDeg: number): Point[] {
  const outer: Point[] = [];
  const inner: Point[] = [];
  for (let i = 0; i <= ARC_STEPS; i++) {
    outer.push(polar(rOuter, startDeg + (30 * i) / ARC_STEPS));
    inner.push(polar(rInner, startDeg + 30 - (30 * i) / ARC_STEPS));
  }
  return [...outer, ...inner];
}

export interface SudarshanaCell {
  /** 0 = iç halka (lagna), 1 = orta (Ay), 2 = dış (Güneş). */
  ring: number;
  /** 1…12. */
  house: number;
  /** Hücrenin tam çokgeni (çizim). */
  points: Point[];
  /** Burç numarasının yeri: iç şeridin ortası. */
  signAnchor: Point;
  /** Etiket alanı: iç şerit dışındaki dilim; `layoutLabels` bununla çağrılır. */
  labelCell: ChartCell;
}

/** Bir halkanın bir evi. `ring` 0…2, `house` 1…12. */
export function sudarshanaCell(ring: number, house: number): SudarshanaCell {
  const rInner = SUDARSHANA_RADII[ring];
  const rOuter = SUDARSHANA_RADII[ring + 1];
  const centre = 180 + (house - 1) * 30;
  const start = centre - 15;
  const rLabel = rInner + SUDARSHANA_SIGN_BAND;
  return {
    ring,
    house,
    points: sectorPoints(rInner, rOuter, start),
    signAnchor: polar(rInner + SUDARSHANA_SIGN_BAND / 2 + 0.2, centre),
    labelCell: {
      house,
      sign: 0,
      points: sectorPoints(rLabel, rOuter, start),
      planetAnchor: polar((rLabel + rOuter) / 2, centre),
      signAnchor: null,
      lagnaAnchor: null,
      sector: { rInner: rLabel, rOuter, startDeg: start, endDeg: start + 30 },
    },
  };
}

/** 36 hücre: halka sırasıyla (içten dışa), her halkada 1. evden 12. eve. */
export function sudarshanaCells(): SudarshanaCell[] {
  const cells: SudarshanaCell[] = [];
  for (let ring = 0; ring < SUDARSHANA_RING_COUNT; ring++) for (let house = 1; house <= 12; house++) cells.push(sudarshanaCell(ring, house));
  return cells;
}
