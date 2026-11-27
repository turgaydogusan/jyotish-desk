import { LABEL_FONT_MIN, layoutLabels, pointInPolygon } from "./geometry";
import { SUDARSHANA_RADII, SUDARSHANA_RING_COUNT, SUDARSHANA_SIGN_BAND, sudarshanaCell, sudarshanaCells } from "./sudarshana";

const radius = (p: { x: number; y: number }) => Math.hypot(p.x - 50, p.y - 50);
/** Matematiksel açı, derece, [0, 360). */
const angle = (p: { x: number; y: number }) => ((Math.atan2(50 - p.y, p.x - 50) * 180) / Math.PI + 360) % 360;

test("thirty-six cells: three rings of twelve houses inside the view box", () => {
  const cells = sudarshanaCells();
  expect(cells).toHaveLength(SUDARSHANA_RING_COUNT * 12);
  expect(cells.map((c) => [c.ring, c.house])).toEqual(Array.from({ length: 36 }, (_, i) => [Math.floor(i / 12), (i % 12) + 1]));
  for (const c of cells) {
    for (const p of c.points) {
      expect(radius(p)).toBeGreaterThanOrEqual(SUDARSHANA_RADII[c.ring] - 1e-9);
      expect(radius(p)).toBeLessThanOrEqual(SUDARSHANA_RADII[c.ring + 1] + 1e-9);
    }
  }
  // Üç halka iç içedir ve kutunun içinde durur.
  expect([...SUDARSHANA_RADII]).toEqual([...SUDARSHANA_RADII].sort((a, b) => a - b));
  expect(SUDARSHANA_RADII[SUDARSHANA_RING_COUNT]).toBeLessThan(50);
});

test("house one is on the left and houses run counter-clockwise at the same angle in every ring", () => {
  for (let ring = 0; ring < SUDARSHANA_RING_COUNT; ring++) {
    expect(angle(sudarshanaCell(ring, 1).signAnchor)).toBeCloseTo(180, 6);
    expect(angle(sudarshanaCell(ring, 4).signAnchor)).toBeCloseTo(270, 6);
    expect(angle(sudarshanaCell(ring, 7).signAnchor)).toBeCloseTo(0, 6);
    expect(angle(sudarshanaCell(ring, 10).signAnchor)).toBeCloseTo(90, 6);
    for (let house = 1; house <= 12; house++) {
      const c = sudarshanaCell(ring, house);
      expect(angle(c.labelCell.planetAnchor)).toBeCloseTo(angle(sudarshanaCell(0, house).labelCell.planetAnchor), 6);
      expect(c.labelCell.house).toBe(house);
    }
  }
});

test("the sign number sits in the inner band and the labels outside it, both inside the cell", () => {
  for (const c of sudarshanaCells()) {
    const rInner = SUDARSHANA_RADII[c.ring];
    expect(pointInPolygon(c.signAnchor, c.points)).toBe(true);
    expect(radius(c.signAnchor)).toBeLessThan(rInner + SUDARSHANA_SIGN_BAND);
    expect(pointInPolygon(c.labelCell.planetAnchor, c.labelCell.points)).toBe(true);
    expect(c.labelCell.sector!.rInner).toBeCloseTo(rInner + SUDARSHANA_SIGN_BAND, 9);
    for (const p of c.labelCell.points) expect(radius(p)).toBeGreaterThanOrEqual(rInner + SUDARSHANA_SIGN_BAND - 1e-9);
  }
});

test("labels fit every cell: up to six in the inner ring and four in the outer rings", () => {
  const texts = ["Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"];
  for (const c of sudarshanaCells()) {
    const most = c.ring === 0 ? 6 : 4;
    for (let n = 1; n <= most; n++) {
      const { fontSize, boxes } = layoutLabels(c.labelCell, texts.slice(0, n));
      expect(boxes).toHaveLength(n);
      expect(fontSize).toBeGreaterThanOrEqual(LABEL_FONT_MIN);
      const band = c.labelCell.sector!;
      for (const b of boxes) {
        const corners = [
          { x: b.x - b.width / 2, y: b.y - b.height / 2 },
          { x: b.x + b.width / 2, y: b.y - b.height / 2 },
          { x: b.x + b.width / 2, y: b.y + b.height / 2 },
          { x: b.x - b.width / 2, y: b.y + b.height / 2 },
        ];
        for (const p of corners) {
          expect(radius(p), `ring ${c.ring} house ${c.house} n ${n}`).toBeLessThanOrEqual(band.rOuter);
          expect(radius(p), `ring ${c.ring} house ${c.house} n ${n}`).toBeGreaterThanOrEqual(band.rInner);
        }
      }
      // Kutular birbirine değmez.
      for (let i = 0; i < n; i++)
        for (let j = i + 1; j < n; j++) {
          const [a, b] = [boxes[i], boxes[j]];
          const apart = Math.abs(a.x - b.x) >= (a.width + b.width) / 2 || Math.abs(a.y - b.y) >= (a.height + b.height) / 2;
          expect(apart, `ring ${c.ring} house ${c.house} n ${n}`).toBe(true);
        }
    }
  }
});
