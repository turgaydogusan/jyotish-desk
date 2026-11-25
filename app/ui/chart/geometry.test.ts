import {
  chartCells,
  circleCells,
  estimateLabelWidth,
  houseSign,
  layoutLabels,
  LABEL_FONT,
  northCells,
  pointInPolygon,
  signHouse,
  southCells,
  type Point,
} from "./geometry";
import type { ChartStyle } from "./types";

const STYLES: ChartStyle[] = ["north", "south", "circle"];

function polygonArea(points: Point[]): number {
  let a = 0;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) a += (points[j].x + points[i].x) * (points[j].y - points[i].y);
  return Math.abs(a / 2);
}

test("house ↔ sign mapping is whole-sign from the ascendant", () => {
  expect(houseSign(0, 1)).toBe(0);
  expect(houseSign(4, 1)).toBe(4);
  expect(houseSign(10, 5)).toBe(2);
  for (let asc = 0; asc < 12; asc++) for (let h = 1; h <= 12; h++) expect(signHouse(asc, houseSign(asc, h))).toBe(h);
});

test("house ↔ sign mapping normalises out-of-range ascendants", () => {
  expect(houseSign(12, 1)).toBe(0);
  expect(houseSign(-1, 1)).toBe(11);
  expect(houseSign(-1, 3)).toBe(1);
  expect(signHouse(12, 0)).toBe(1);
  expect(signHouse(-1, 0)).toBe(2);
  expect(signHouse(13, 0)).toBe(12);
  for (const asc of [12, -1, 25, -13]) for (let h = 1; h <= 12; h++) {
    const s = houseSign(asc, h);
    expect(s).toBeGreaterThanOrEqual(0);
    expect(s).toBeLessThan(12);
    expect(signHouse(asc, s)).toBe(h);
  }
});

test.each(STYLES)("%s: 12 houses, each sign exactly once, house 1 holds the ascendant", (style) => {
  for (const asc of [0, 4, 11]) {
    const cells = chartCells(style, asc);
    expect(cells.map((c) => c.house)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    expect(new Set(cells.map((c) => c.sign)).size).toBe(12);
    expect(cells[0].sign).toBe(asc);
    for (const c of cells) expect(c.sign).toBe(houseSign(asc, c.house));
  }
});

test.each(STYLES)("%s: every anchor lies inside its own cell", (style) => {
  for (const c of chartCells(style, 3)) {
    expect(pointInPolygon(c.planetAnchor, c.points), `house ${c.house} planet`).toBe(true);
    if (c.signAnchor) expect(pointInPolygon(c.signAnchor, c.points), `house ${c.house} sign`).toBe(true);
    if (c.lagnaAnchor) expect(pointInPolygon(c.lagnaAnchor, c.points), `house ${c.house} lagna`).toBe(true);
  }
});

test("north: house 1 is the top diamond and houses run counter-clockwise", () => {
  const cells = northCells(0);
  const centroid = (h: number) => {
    const pts = cells[h - 1].points;
    return { x: pts.reduce((s, p) => s + p.x, 0) / pts.length, y: pts.reduce((s, p) => s + p.y, 0) / pts.length };
  };
  expect(centroid(1)).toEqual({ x: 50, y: 25 });
  expect(centroid(4)).toEqual({ x: 25, y: 50 });
  expect(centroid(7)).toEqual({ x: 50, y: 75 });
  expect(centroid(10)).toEqual({ x: 75, y: 50 });
  expect(centroid(2).x).toBeLessThan(50); // sola doğru
  expect(centroid(12).x).toBeGreaterThan(50);
  // Dört baklava ve sekiz üçgen kareyi tam kaplar.
  const total = cells.reduce((s, c) => s + polygonArea(c.points), 0);
  expect(total).toBeCloseTo(100 * 100, 6);
  expect(cells.filter((c) => c.points.length === 4)).toHaveLength(4);
  expect(cells.every((c) => c.signAnchor !== null && c.lagnaAnchor === null)).toBe(true);
});

test("south: signs are fixed with Pisces top-left; the lagna marker follows the ascendant", () => {
  const cells = southCells(4);
  const pisces = cells.find((c) => c.sign === 11)!;
  expect(pisces.points[0]).toEqual({ x: 0, y: 0 });
  const aries = cells.find((c) => c.sign === 0)!;
  expect(aries.points[0]).toEqual({ x: 25, y: 0 });
  const leo = cells.find((c) => c.sign === 4)!;
  expect(leo.house).toBe(1);
  expect(leo.lagnaAnchor).not.toBeNull();
  expect(cells.filter((c) => c.lagnaAnchor !== null)).toHaveLength(1);
  // Sabit burç yerleri lagnaya göre değişmez.
  expect(southCells(0).find((c) => c.sign === 4)!.points).toEqual(leo.points);
});

test("circle: house 1 sits on the left and houses run counter-clockwise", () => {
  const cells = circleCells(0);
  expect(cells[0].planetAnchor.x).toBeLessThan(50);
  expect(cells[0].planetAnchor.y).toBeCloseTo(50, 6);
  // 2. ev ekranda sol altta (y aşağı büyür).
  expect(cells[1].planetAnchor.y).toBeGreaterThan(50);
  expect(cells[3].planetAnchor.y).toBeGreaterThan(cells[1].planetAnchor.y);
});

test("circle: every house has a sign anchor inside the outer ring", () => {
  for (const c of circleCells(2)) {
    expect(c.signAnchor, `house ${c.house}`).not.toBeNull();
    const r = Math.hypot(c.signAnchor!.x - 50, c.signAnchor!.y - 50);
    expect(r).toBeGreaterThan(36);
    expect(r).toBeLessThan(48);
    expect(pointInPolygon(c.signAnchor!, c.points)).toBe(true);
  }
});

test("label width estimate grows with character count and font size", () => {
  expect(estimateLabelWidth("RaR", LABEL_FONT)).toBeGreaterThan(estimateLabelWidth("Ra", LABEL_FONT));
  expect(estimateLabelWidth("MeRc", LABEL_FONT)).toBeCloseTo((4 / 3) * estimateLabelWidth("RaR", LABEL_FONT) - (1 / 3) * estimateLabelWidth("", LABEL_FONT), 6);
  expect(estimateLabelWidth("MeRc", 2)).toBeLessThan(estimateLabelWidth("MeRc", 4));
});

// Beş gezegen, hepsi retro ve yanık: en uzun gerçekçi etiketler.
const CROWD = ["RaRc", "MeRc", "MaRc", "JuRc", "SaRc"];

type Box = { x: number; y: number; width: number; height: number };
const corners = (b: Box): Point[] => [
  { x: b.x - b.width / 2, y: b.y - b.height / 2 },
  { x: b.x + b.width / 2, y: b.y - b.height / 2 },
  { x: b.x + b.width / 2, y: b.y + b.height / 2 },
  { x: b.x - b.width / 2, y: b.y + b.height / 2 },
];
const overlap = (a: Box, b: Box) =>
  Math.abs(a.x - b.x) < (a.width + b.width) / 2 && Math.abs(a.y - b.y) < (a.height + b.height) / 2;

function expectPairwiseDisjoint(boxes: Box[], what: string) {
  for (let i = 0; i < boxes.length; i++)
    for (let j = i + 1; j < boxes.length; j++) expect(overlap(boxes[i], boxes[j]), `${what}: ${i}×${j}`).toBe(false);
}

test.each(["north", "south"] as const)("%s: five marked planets fit in every house without overlapping", (style) => {
  for (const asc of [0, 7]) {
    for (const c of chartCells(style, asc)) {
      const n = CROWD.slice(0, 5);
      const { boxes, fontSize } = layoutLabels(c, n, "As");
      expect(boxes).toHaveLength(5);
      boxes.forEach((b, i) => expect(b.width).toBeGreaterThanOrEqual(estimateLabelWidth(n[i], fontSize) - 1e-9));
      expectPairwiseDisjoint(boxes, `house ${c.house}`);
      for (const b of boxes)
        for (const p of corners(b)) expect(pointInPolygon(p, c.points), `house ${c.house} corner ${p.x},${p.y}`).toBe(true);
    }
  }
});

test("circle: five marked planets stay within the radial and angular bounds of every house", () => {
  for (const c of circleCells(0)) {
    const { boxes } = layoutLabels(c, CROWD, "As");
    expect(boxes).toHaveLength(5);
    expectPairwiseDisjoint(boxes, `house ${c.house}`);
    const centre = 180 + (c.house - 1) * 30;
    for (const b of boxes) {
      for (const p of corners(b)) {
        const r = Math.hypot(p.x - 50, p.y - 50);
        expect(r, `house ${c.house} r`).toBeLessThanOrEqual(48);
        const a = (Math.atan2(-(p.y - 50), p.x - 50) * 180) / Math.PI;
        const d = ((a - centre + 540) % 360) - 180;
        expect(Math.abs(d), `house ${c.house} angle`).toBeLessThanOrEqual(15);
      }
      // Kutunun merkeze en yakın noktası bile iç çemberin dışında kalır.
      const nx = Math.min(Math.max(50, b.x - b.width / 2), b.x + b.width / 2);
      const ny = Math.min(Math.max(50, b.y - b.height / 2), b.y + b.height / 2);
      expect(Math.hypot(nx - 50, ny - 50), `house ${c.house} inner`).toBeGreaterThanOrEqual(18);
    }
  }
});

test("a single short label keeps the full font size at the planet anchor", () => {
  for (const style of STYLES) {
    for (const c of chartCells(style, 0)) {
      const { boxes, fontSize } = layoutLabels(c, ["Su"]);
      expect(fontSize).toBe(LABEL_FONT);
      expect(Math.hypot(boxes[0].x - c.planetAnchor.x, boxes[0].y - c.planetAnchor.y), `${style} ${c.house}`).toBeLessThan(6);
    }
  }
});
