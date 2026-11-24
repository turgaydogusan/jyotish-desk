// Üç harita stili için saf geometri. Koordinatlar 100 × 100 kutudadır (svg viewBox).
// Evler bütün burç (whole sign) kabulüyle burçlara eşlenir: ev h → (lagna + h − 1) mod 12.
import type { ChartStyle } from "./types";

export interface Point {
  x: number;
  y: number;
}

export interface StackRule {
  /** Birden çok gezegen bu eksen boyunca dizilir. */
  axis: "x" | "y";
  step: number;
  perLine: number;
  lineGap: number;
}

export interface ChartCell {
  house: number; // 1…12
  sign: number; // 0…11
  points: Point[];
  planetAnchor: Point;
  /** Burç numarasının yeri (yalnızca Kuzey Hint). */
  signAnchor: Point | null;
  /** Lagna işaretinin yeri (Güney Hint ve daire, yalnızca 1. ev). */
  lagnaAnchor: Point | null;
  stack: StackRule;
}

export const VIEWBOX = 100;

export function houseSign(asc: number, house: number): number {
  return (asc + house - 1) % 12;
}

export function signHouse(asc: number, sign: number): number {
  return ((sign - asc + 12) % 12) + 1;
}

const pt = (x: number, y: number): Point => ({ x, y });

// Kuzey Hint: dış kare, iki köşegen ve kenar ortalarını birleştiren eşkenar dörtgen.
// 1. ev üstteki baklava; evler saat yönünün tersine ilerler.
const TL = pt(0, 0), TR = pt(100, 0), BR = pt(100, 100), BL = pt(0, 100);
const T = pt(50, 0), R = pt(100, 50), B = pt(50, 100), L = pt(0, 50), C = pt(50, 50);
const PTL = pt(25, 25), PTR = pt(75, 25), PBR = pt(75, 75), PBL = pt(25, 75);

// Baklavalar ve yan üçgenler dikey, üst/alt üçgenler yatay dizer.
const VERTICAL: StackRule = { axis: "y", step: 5.5, perLine: 4, lineGap: 9 };
const HORIZONTAL: StackRule = { axis: "x", step: 7.5, perLine: 4, lineGap: 5.5 };

const NORTH: ReadonlyArray<{ points: Point[]; planet: Point; sign: Point; stack: StackRule }> = [
  { points: [T, PTR, C, PTL], planet: pt(50, 22), sign: pt(50, 42), stack: VERTICAL }, // 1
  { points: [TL, T, PTL], planet: pt(25, 7), sign: pt(25, 19), stack: HORIZONTAL }, // 2
  { points: [TL, PTL, L], planet: pt(7, 25), sign: pt(19, 25), stack: VERTICAL }, // 3
  { points: [L, PTL, C, PBL], planet: pt(22, 50), sign: pt(42, 50), stack: VERTICAL }, // 4
  { points: [L, PBL, BL], planet: pt(7, 75), sign: pt(19, 75), stack: VERTICAL }, // 5
  { points: [BL, PBL, B], planet: pt(25, 93), sign: pt(25, 81), stack: HORIZONTAL }, // 6
  { points: [B, PBL, C, PBR], planet: pt(50, 78), sign: pt(50, 58), stack: VERTICAL }, // 7
  { points: [B, PBR, BR], planet: pt(75, 93), sign: pt(75, 81), stack: HORIZONTAL }, // 8
  { points: [BR, PBR, R], planet: pt(93, 75), sign: pt(81, 75), stack: VERTICAL }, // 9
  { points: [R, PBR, C, PTR], planet: pt(78, 50), sign: pt(58, 50), stack: VERTICAL }, // 10
  { points: [R, PTR, TR], planet: pt(93, 25), sign: pt(81, 25), stack: VERTICAL }, // 11
  { points: [TR, PTR, T], planet: pt(75, 7), sign: pt(75, 19), stack: HORIZONTAL }, // 12
];

export function northCells(asc: number): ChartCell[] {
  return NORTH.map((h, i) => ({
    house: i + 1,
    sign: houseSign(asc, i + 1),
    points: h.points,
    planetAnchor: h.planet,
    signAnchor: h.sign,
    lagnaAnchor: null,
    stack: h.stack,
  }));
}

/** Güney Hint: burçlar sabit, 4 × 4 ızgaranın dış halkası, Meena sol üstte, saat yönünde. */
export const SOUTH_SIGN_CELLS: ReadonlyArray<{ sign: number; row: number; col: number }> = [
  { sign: 11, row: 0, col: 0 },
  { sign: 0, row: 0, col: 1 },
  { sign: 1, row: 0, col: 2 },
  { sign: 2, row: 0, col: 3 },
  { sign: 3, row: 1, col: 3 },
  { sign: 4, row: 2, col: 3 },
  { sign: 5, row: 3, col: 3 },
  { sign: 6, row: 3, col: 2 },
  { sign: 7, row: 3, col: 1 },
  { sign: 8, row: 3, col: 0 },
  { sign: 9, row: 2, col: 0 },
  { sign: 10, row: 1, col: 0 },
];

const SOUTH_STACK: StackRule = { axis: "y", step: 5, perLine: 3, lineGap: 9 };

export function southCells(asc: number): ChartCell[] {
  const cells = SOUTH_SIGN_CELLS.map(({ sign, row, col }) => {
    const x = col * 25;
    const y = row * 25;
    const house = signHouse(asc, sign);
    return {
      house,
      sign,
      points: [pt(x, y), pt(x + 25, y), pt(x + 25, y + 25), pt(x, y + 25)],
      planetAnchor: pt(x + 12.5, y + 14),
      signAnchor: null,
      lagnaAnchor: house === 1 ? pt(x + 4, y + 6) : null,
      stack: SOUTH_STACK,
    };
  });
  return cells.sort((a, b) => a.house - b.house);
}

const CIRCLE_R_OUTER = 48;
const CIRCLE_R_INNER = 18;
const CIRCLE_STACK: StackRule = { axis: "y", step: 5, perLine: 3, lineGap: 7 };

function polar(r: number, deg: number): Point {
  const rad = (deg * Math.PI) / 180;
  // Matematiksel açı (saat yönünün tersi, y yukarı) → ekran (y aşağı).
  return pt(50 + r * Math.cos(rad), 50 - r * Math.sin(rad));
}

/** Daire: 1. ev solda (doğu ufku), evler saat yönünün tersine; her ev 30°'lik halka dilimi. */
export function circleCells(asc: number): ChartCell[] {
  const cells: ChartCell[] = [];
  for (let house = 1; house <= 12; house++) {
    const center = 180 + (house - 1) * 30;
    const start = center - 15;
    const outer: Point[] = [];
    const inner: Point[] = [];
    for (let i = 0; i <= 8; i++) {
      outer.push(polar(CIRCLE_R_OUTER, start + (30 * i) / 8));
      inner.push(polar(CIRCLE_R_INNER, start + 30 - (30 * i) / 8));
    }
    cells.push({
      house,
      sign: houseSign(asc, house),
      points: [...outer, ...inner],
      planetAnchor: polar((CIRCLE_R_OUTER + CIRCLE_R_INNER) / 2 + 2, center),
      signAnchor: null,
      lagnaAnchor: house === 1 ? polar(CIRCLE_R_INNER + 4, center) : null,
      stack: CIRCLE_STACK,
    });
  }
  return cells;
}

export function chartCells(style: ChartStyle, asc: number): ChartCell[] {
  if (style === "south") return southCells(asc);
  if (style === "circle") return circleCells(asc);
  return northCells(asc);
}

/** Aynı evdeki n gezegenin etiket konumları: kural eksenine dizilir, sığmazsa yeni satır. */
export function stackPositions(anchor: Point, n: number, rule: StackRule): Point[] {
  const lines = Math.ceil(n / rule.perLine);
  const out: Point[] = [];
  for (let i = 0; i < n; i++) {
    const line = Math.floor(i / rule.perLine);
    const idx = i % rule.perLine;
    const inLine = Math.min(rule.perLine, n - line * rule.perLine);
    const along = (idx - (inLine - 1) / 2) * rule.step;
    const across = (line - (lines - 1) / 2) * rule.lineGap;
    out.push(rule.axis === "y" ? pt(anchor.x + across, anchor.y + along) : pt(anchor.x + along, anchor.y + across));
  }
  return out;
}

export function pathOf(points: readonly Point[]): string {
  return points.map((p, i) => `${i === 0 ? "M" : "L"}${round(p.x)} ${round(p.y)}`).join(" ") + " Z";
}

function round(v: number): number {
  return Math.round(v * 100) / 100;
}

/** Işın atma ile nokta-çokgen testi (testlerde ve isabet denetiminde kullanılır). */
export function pointInPolygon(p: Point, poly: readonly Point[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const a = poly[i];
    const b = poly[j];
    if (a.y > p.y !== b.y > p.y && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}
