import {
  chartCells,
  circleCells,
  houseSign,
  northCells,
  pointInPolygon,
  signHouse,
  southCells,
  stackPositions,
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

test("stackPositions centres on the anchor and never puts two labels on the same spot", () => {
  const rule = { axis: "y" as const, step: 5, perLine: 3, lineGap: 9 };
  expect(stackPositions({ x: 10, y: 10 }, 1, rule)).toEqual([{ x: 10, y: 10 }]);
  expect(stackPositions({ x: 10, y: 10 }, 2, rule)).toEqual([
    { x: 10, y: 7.5 },
    { x: 10, y: 12.5 },
  ]);
  const nine = stackPositions({ x: 50, y: 50 }, 9, rule);
  const keys = new Set(nine.map((p) => `${p.x},${p.y}`));
  expect(keys.size).toBe(9);
  for (let i = 0; i < nine.length; i++)
    for (let j = i + 1; j < nine.length; j++) {
      const d = Math.hypot(nine[i].x - nine[j].x, nine[i].y - nine[j].y);
      expect(d).toBeGreaterThanOrEqual(5);
    }
});

test.each(STYLES)("%s: up to three planets in a house stay inside it", (style) => {
  for (const c of chartCells(style, 0)) {
    for (const p of stackPositions(c.planetAnchor, 3, c.stack)) {
      expect(pointInPolygon(p, c.points), `house ${c.house}`).toBe(true);
    }
  }
});
