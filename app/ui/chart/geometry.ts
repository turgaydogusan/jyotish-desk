// Üç harita stili için saf geometri. Koordinatlar 100 × 100 kutudadır (svg viewBox).
// Evler bütün burç (whole sign) kabulüyle burçlara eşlenir: ev h → (lagna + h − 1) mod 12.
import type { ChartStyle } from "./types";

export interface Point {
  x: number;
  y: number;
}

export interface ChartCell {
  house: number; // 1…12
  sign: number; // 0…11
  points: Point[];
  planetAnchor: Point;
  /** Burç numarasının yeri (Kuzey Hint ve daire; Güney Hint'te burçlar sabit yerdedir). */
  signAnchor: Point | null;
  /** Lagna işaretinin yeri (Güney Hint ve daire, yalnızca 1. ev). */
  lagnaAnchor: Point | null;
  /** Daire stilinde hücrenin tam halka dilimi (açılar matematiksel, derece). Çokgen yalnızca yaklaşıktır. */
  sector?: Sector;
}

export interface Sector {
  rInner: number;
  rOuter: number;
  startDeg: number;
  endDeg: number;
}

export const VIEWBOX = 100;

/** Herhangi bir tamsayıyı 0…11 aralığına getirir (asc 12 → 0, −1 → 11). */
function mod12(n: number): number {
  return ((n % 12) + 12) % 12;
}

export function houseSign(asc: number, house: number): number {
  return mod12(mod12(asc) + house - 1);
}

export function signHouse(asc: number, sign: number): number {
  return mod12(sign - mod12(asc)) + 1;
}

const pt = (x: number, y: number): Point => ({ x, y });

// Kuzey Hint: dış kare, iki köşegen ve kenar ortalarını birleştiren eşkenar dörtgen.
// 1. ev üstteki baklava; evler saat yönünün tersine ilerler.
const TL = pt(0, 0), TR = pt(100, 0), BR = pt(100, 100), BL = pt(0, 100);
const T = pt(50, 0), R = pt(100, 50), B = pt(50, 100), L = pt(0, 50), C = pt(50, 50);
const PTL = pt(25, 25), PTR = pt(75, 25), PBR = pt(75, 75), PBL = pt(25, 75);

const NORTH: ReadonlyArray<{ points: Point[]; planet: Point; sign: Point }> = [
  { points: [T, PTR, C, PTL], planet: pt(50, 22), sign: pt(50, 42) }, // 1
  { points: [TL, T, PTL], planet: pt(25, 7), sign: pt(25, 19) }, // 2
  { points: [TL, PTL, L], planet: pt(7, 25), sign: pt(19, 25) }, // 3
  { points: [L, PTL, C, PBL], planet: pt(22, 50), sign: pt(42, 50) }, // 4
  { points: [L, PBL, BL], planet: pt(7, 75), sign: pt(19, 75) }, // 5
  { points: [BL, PBL, B], planet: pt(25, 93), sign: pt(25, 81) }, // 6
  { points: [B, PBL, C, PBR], planet: pt(50, 78), sign: pt(50, 58) }, // 7
  { points: [B, PBR, BR], planet: pt(75, 93), sign: pt(75, 81) }, // 8
  { points: [BR, PBR, R], planet: pt(93, 75), sign: pt(81, 75) }, // 9
  { points: [R, PBR, C, PTR], planet: pt(78, 50), sign: pt(58, 50) }, // 10
  { points: [R, PTR, TR], planet: pt(93, 25), sign: pt(81, 25) }, // 11
  { points: [TR, PTR, T], planet: pt(75, 7), sign: pt(75, 19) }, // 12
];

export function northCells(asc: number): ChartCell[] {
  return NORTH.map((h, i) => ({
    house: i + 1,
    sign: houseSign(asc, i + 1),
    points: h.points,
    planetAnchor: h.planet,
    signAnchor: h.sign,
    lagnaAnchor: null,
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
    };
  });
  return cells.sort((a, b) => a.house - b.house);
}

const CIRCLE_R_OUTER = 48;
const CIRCLE_R_INNER = 18;

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
      // Burç numarası dış çemberin hemen içinde, evin ortasında.
      signAnchor: polar(CIRCLE_R_OUTER - 4.5, center),
      lagnaAnchor: house === 1 ? polar(CIRCLE_R_INNER + 4, center) : null,
      sector: { rInner: CIRCLE_R_INNER, rOuter: CIRCLE_R_OUTER, startDeg: start, endDeg: start + 30 },
    });
  }
  return cells;
}

export function chartCells(style: ChartStyle, asc: number): ChartCell[] {
  if (style === "south") return southCells(asc);
  if (style === "circle") return circleCells(asc);
  return northCells(asc);
}

// ---------------------------------------------------------------------------
// Etiket yerleşimi
//
// Genişlik tahmini: etiketler sistem fontuyla (SF Pro) çizilir. Kısaltma harfleri ve retro "R"
// için ortalama glif genişliği 0.7 em alınır (geniş "M" ≈ 0.85 em, "a/e" ≈ 0.55 em; iki
// harfli kısaltmada ortalama ≤ 0.7 em). Kutunun her yanına 0.15 em pay eklenir:
//   genişlik = fontSize × (0.7 × karakterSayısı + 0.3),   yükseklik = fontSize × 1.2
// Karakter sayısı kod noktasıyla sayılır ("MeRc" = 4). Birimler viewBox birimidir.
// ---------------------------------------------------------------------------

/** Gezegen etiketinin olağan yazı boyutu (viewBox birimi). */
export const LABEL_FONT = 4.2;
/** Kalabalık hücrede inilebilecek en küçük boyut. */
export const LABEL_FONT_MIN = 2.4;
const LABEL_FONT_STEP = 0.2;
/** Burç numarası ve lagna işaretinin yazı boyutu. */
export const SIGN_FONT = 3.4;
const CHAR_EM = 0.7;
const PAD_EM = 0.15;
const LINE_EM = 1.2;
/** Kutular arası boşluk ve hücre kenarına (çizgi kalınlığı dahil) uzaklık. */
const LABEL_GAP = 0.4;
const CELL_MARGIN = 0.4;
const CANDIDATE_STEP = 1;

export function estimateLabelWidth(text: string, fontSize: number): number {
  return fontSize * ([...text].length * CHAR_EM + 2 * PAD_EM);
}

export function labelHeight(fontSize: number): number {
  return fontSize * LINE_EM;
}

/** Merkezi (x, y) olan eksen hizalı kutu. */
export interface LabelBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LabelLayout {
  fontSize: number;
  boxes: LabelBox[];
}

function boxesOverlap(a: LabelBox, b: LabelBox, gap: number): boolean {
  return Math.abs(a.x - b.x) < (a.width + b.width) / 2 + gap && Math.abs(a.y - b.y) < (a.height + b.height) / 2 + gap;
}

function boxCorners(b: LabelBox): Point[] {
  const hw = b.width / 2;
  const hh = b.height / 2;
  return [pt(b.x - hw, b.y - hh), pt(b.x + hw, b.y - hh), pt(b.x + hw, b.y + hh), pt(b.x - hw, b.y + hh)];
}

function distToSegment(p: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const t = Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy)));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

/** Kutu hücrenin içinde ve kenarlardan en az `margin` uzakta mı? */
function cellContainsBox(cell: ChartCell, b: LabelBox, margin: number): boolean {
  const corners = boxCorners(b);
  if (cell.sector) {
    // Halka dilimi: dış yarıçapa en uzak nokta bir köşedir; iç çembere en yakın nokta
    // kutunun merkeze en yakın noktasıdır; 30°'lik kama dışbükey olduğundan köşeler yeter.
    const { rInner, rOuter, startDeg, endDeg } = cell.sector;
    const s = (startDeg * Math.PI) / 180;
    const e = (endDeg * Math.PI) / 180;
    const c = (s + e) / 2;
    for (const p of corners) {
      const dx = p.x - 50;
      const dy = 50 - p.y;
      if (Math.hypot(dx, dy) > rOuter - margin) return false;
      if (dx * Math.cos(c) + dy * Math.sin(c) <= 0) return false;
      if (Math.cos(s) * dy - Math.sin(s) * dx < margin) return false;
      if (Math.sin(e) * dx - Math.cos(e) * dy < margin) return false;
    }
    const nx = Math.min(Math.max(50, b.x - b.width / 2), b.x + b.width / 2);
    const ny = Math.min(Math.max(50, b.y - b.height / 2), b.y + b.height / 2);
    return Math.hypot(nx - 50, ny - 50) >= rInner + margin;
  }
  // Kuzey ve Güney Hint hücreleri dışbükeydir: dört köşe içerideyse kutu da içeridedir.
  const poly = cell.points;
  for (const p of corners) {
    if (!pointInPolygon(p, poly)) return false;
    for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      if (distToSegment(p, poly[j], poly[i]) < margin) return false;
    }
  }
  return true;
}

/** n etiketin satırlara dizilişleri (her satır ortalanır), merkez (0, 0) etrafında. */
function arrangements(widths: readonly number[], height: number): LabelBox[][] {
  const n = widths.length;
  const out: LabelBox[][] = [];
  const seen = new Set<number>();
  for (let rows = 1; rows <= n; rows++) {
    const perRow = Math.ceil(n / rows);
    if (seen.has(perRow)) continue;
    seen.add(perRow);
    const lines = Math.ceil(n / perRow);
    const boxes: LabelBox[] = [];
    for (let line = 0; line < lines; line++) {
      const items = widths.slice(line * perRow, (line + 1) * perRow);
      const total = items.reduce((a, w) => a + w, 0) + LABEL_GAP * (items.length - 1);
      let x = -total / 2;
      const y = (line - (lines - 1) / 2) * (height + LABEL_GAP);
      for (const w of items) {
        boxes.push({ x: x + w / 2, y, width: w, height });
        x += w + LABEL_GAP;
      }
    }
    out.push(boxes);
  }
  return out;
}

/** Hücre içindeki aday merkezler, gezegen çapasına uzaklığa göre sıralı (ilk aday çapanın kendisi). */
function candidateCentres(cell: ChartCell): Point[] {
  const xs = cell.points.map((p) => p.x);
  const ys = cell.points.map((p) => p.y);
  const a = cell.planetAnchor;
  const pts: Point[] = [];
  for (let x = Math.min(...xs); x <= Math.max(...xs); x += CANDIDATE_STEP)
    for (let y = Math.min(...ys); y <= Math.max(...ys); y += CANDIDATE_STEP) {
      const p = pt(x, y);
      if (pointInPolygon(p, cell.points)) pts.push(p);
    }
  pts.sort((p, q) => Math.hypot(p.x - a.x, p.y - a.y) - Math.hypot(q.x - a.x, q.y - a.y));
  return [a, ...pts];
}

/** Burç numarası ve lagna işaretinin kapladığı alan; etiketler bunlara da değmez. */
function reservedBoxes(cell: ChartCell, lagnaText: string): LabelBox[] {
  const out: LabelBox[] = [];
  const h = labelHeight(SIGN_FONT);
  if (cell.signAnchor) out.push({ ...cell.signAnchor, width: estimateLabelWidth(String(cell.sign + 1), SIGN_FONT), height: h });
  if (cell.lagnaAnchor && lagnaText) out.push({ ...cell.lagnaAnchor, width: estimateLabelWidth(lagnaText, SIGN_FONT), height: h });
  return out;
}

/**
 * Aynı hücredeki etiketlerin kutuları. Olağan boyuttan başlayıp gerekirse küçülterek,
 * kutuların birbirine, burç/lagna işaretine değmediği ve hücre içinde kaldığı ilk boyutu
 * seçer; o boyutta çapaya en yakın dizilişi alır. Hiçbiri sığmazsa en küçük boyutta
 * çapaya ortalanmış ızgara döner.
 */
export function layoutLabels(cell: ChartCell, texts: readonly string[], lagnaText = ""): LabelLayout {
  if (texts.length === 0) return { fontSize: LABEL_FONT, boxes: [] };
  const reserved = reservedBoxes(cell, lagnaText);
  let candidates: Point[] | null = null;
  const fits = (rel: LabelBox[], c: Point): LabelBox[] | null => {
    const placed = rel.map((b) => ({ ...b, x: b.x + c.x, y: b.y + c.y }));
    for (const b of placed) {
      if (!cellContainsBox(cell, b, CELL_MARGIN)) return null;
      if (reserved.some((r) => boxesOverlap(b, r, LABEL_GAP))) return null;
    }
    return placed;
  };
  const steps = Math.round((LABEL_FONT - LABEL_FONT_MIN) / LABEL_FONT_STEP);
  for (let k = 0; k <= steps; k++) {
    const fontSize = Math.round((LABEL_FONT - k * LABEL_FONT_STEP) * 100) / 100;
    const widths = texts.map((t) => estimateLabelWidth(t, fontSize));
    const options = arrangements(widths, labelHeight(fontSize));
    // Önce çapanın kendisi: çoğu hücrede arama gerekmez.
    for (const rel of options) {
      const placed = fits(rel, cell.planetAnchor);
      if (placed) return { fontSize, boxes: placed };
    }
    candidates ??= candidateCentres(cell);
    let best: { d: number; boxes: LabelBox[] } | null = null;
    for (const rel of options) {
      for (const c of candidates) {
        const d = Math.hypot(c.x - cell.planetAnchor.x, c.y - cell.planetAnchor.y);
        if (best && d >= best.d) break;
        const placed = fits(rel, c);
        if (placed) {
          best = { d, boxes: placed };
          break;
        }
      }
    }
    if (best) return { fontSize, boxes: best.boxes };
  }
  const widths = texts.map((t) => estimateLabelWidth(t, LABEL_FONT_MIN));
  const grid = arrangements(widths, labelHeight(LABEL_FONT_MIN));
  const rel = grid[Math.min(grid.length - 1, Math.floor(Math.sqrt(texts.length)) - 1)];
  return {
    fontSize: LABEL_FONT_MIN,
    boxes: rel.map((b) => ({ ...b, x: b.x + cell.planetAnchor.x, y: b.y + cell.planetAnchor.y })),
  };
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
