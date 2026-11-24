import {
  clampPanel,
  collides,
  columnsForWidth,
  compact,
  GRID_COLUMNS,
  hasOverlap,
  minSize,
  movePanel,
  normalizePanels,
  reflow,
  resizePanel,
  ROW_HEIGHT_PX,
  settle,
} from "./grid";
import { builtinLayouts, type Panel } from "./model";

function p(id: string, x: number, y: number, w: number, h: number, kind = "notes"): Panel {
  return { id, kind, x, y, w, h, settings: {} };
}
const at = (panels: Panel[], id: string) => {
  const q = panels.find((r) => r.id === id)!;
  return [q.x, q.y, q.w, q.h];
};

/** Değişmezler: çakışma yok, ızgara içinde, en küçük boyut, tamsayı. */
function expectValid(panels: Panel[], columns = GRID_COLUMNS) {
  expect(hasOverlap(panels)).toBe(false);
  for (const q of panels) {
    const m = minSize(q.kind);
    expect(q.x).toBeGreaterThanOrEqual(0);
    expect(q.y).toBeGreaterThanOrEqual(0);
    expect(q.x + q.w).toBeLessThanOrEqual(columns);
    expect(q.w).toBeGreaterThanOrEqual(Math.min(m.w, columns));
    expect(q.h).toBeGreaterThanOrEqual(m.h);
    for (const v of [q.x, q.y, q.w, q.h]) expect(Number.isInteger(v)).toBe(true);
  }
}

test("constants follow the spec", () => {
  expect(GRID_COLUMNS).toBe(12);
  expect(ROW_HEIGHT_PX).toBe(32);
  expect(minSize("chart")).toEqual({ w: 4, h: 6 });
  expect(minSize("notes")).toEqual({ w: 3, h: 4 });
  expect(minSize("something_new")).toEqual({ w: 3, h: 4 });
});

test("columnsForWidth breakpoints", () => {
  expect(columnsForWidth(1600)).toBe(12);
  expect(columnsForWidth(1000)).toBe(12);
  expect(columnsForWidth(999)).toBe(8);
  expect(columnsForWidth(640)).toBe(8);
  expect(columnsForWidth(639)).toBe(4);
  expect(columnsForWidth(0)).toBe(4);
});

test("collides treats touching edges as free", () => {
  expect(collides(p("a", 0, 0, 3, 4), p("b", 3, 0, 3, 4))).toBe(false);
  expect(collides(p("a", 0, 0, 3, 4), p("b", 0, 4, 3, 4))).toBe(false);
  expect(collides(p("a", 0, 0, 3, 4), p("b", 2, 3, 3, 4))).toBe(true);
});

test("clampPanel enforces minimum size, grid bounds and integers", () => {
  expect(at([clampPanel(p("a", -2, -1, 1, 1))], "a")).toEqual([0, 0, 3, 4]);
  expect(at([clampPanel(p("c", 0, 0, 2, 2, "chart"))], "c")).toEqual([0, 0, 4, 6]);
  expect(at([clampPanel(p("a", 11, 0, 5, 4))], "a")).toEqual([7, 0, 5, 4]);
  expect(at([clampPanel(p("a", 0, 0, 30, 4))], "a")).toEqual([0, 0, 12, 4]);
  expect(at([clampPanel(p("a", 1.4, 2.6, 3.5, 4.4))], "a")).toEqual([1, 3, 4, 4]);
});

test("moving onto a panel pushes it down; the moved panel stays put", () => {
  const panels = [p("a", 0, 0, 6, 4), p("b", 6, 0, 6, 4)];
  const out = movePanel(panels, "b", 2, 2);
  expect(at(out, "b")).toEqual([2, 2, 6, 4]);
  expect(at(out, "a")).toEqual([0, 6, 6, 4]);
  expectValid(out);
});

test("pushing cascades through a stack", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 0, 4, 4, 4), p("c", 0, 8, 4, 4), p("x", 6, 0, 4, 4)];
  const out = movePanel(panels, "x", 0, 2);
  expect(at(out, "x")).toEqual([0, 2, 4, 4]);
  expect(at(out, "a")).toEqual([0, 6, 4, 4]);
  expect(at(out, "b")).toEqual([0, 10, 4, 4]);
  expect(at(out, "c")).toEqual([0, 14, 4, 4]);
  expectValid(out);
});

test("panels that do not collide keep their positions (gaps are preserved)", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 8, 10, 4, 4)];
  const out = movePanel(panels, "a", 0, 3);
  expect(at(out, "a")).toEqual([0, 3, 4, 4]);
  expect(at(out, "b")).toEqual([8, 10, 4, 4]);
});

test("move clamps to the grid and keeps array order", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 4, 0, 4, 4)];
  const out = movePanel(panels, "b", 20, -3);
  expect(out.map((q) => q.id)).toEqual(["a", "b"]);
  expect(at(out, "b")).toEqual([8, 0, 4, 4]);
});

test("moving an unknown id returns an equal copy", () => {
  const panels = [p("a", 0, 0, 4, 4)];
  expect(movePanel(panels, "zz", 1, 1)).toEqual(panels);
});

test("resize grows into neighbours and pushes them down", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 4, 0, 4, 4), p("c", 0, 4, 8, 4)];
  const out = resizePanel(panels, "a", 6, 6);
  expect(at(out, "a")).toEqual([0, 0, 6, 6]);
  expect(at(out, "b")).toEqual([4, 6, 4, 4]);
  expect(at(out, "c")).toEqual([0, 10, 8, 4]);
  expectValid(out);
});

test("resize respects minimum size and the right edge", () => {
  const panels = [p("c", 8, 0, 4, 6, "chart"), p("n", 0, 0, 3, 4)];
  expect(at(resizePanel(panels, "c", 1, 1), "c")).toEqual([8, 0, 4, 6]);
  expect(at(resizePanel(panels, "n", 40, 4), "n")).toEqual([0, 0, 12, 4]);
  expect(at(resizePanel(panels, "c", 9, 6), "c")).toEqual([8, 0, 4, 6]);
});

test("settle without a pin resolves overlapping input in reading order", () => {
  const out = settle([p("a", 0, 0, 4, 4), p("b", 0, 1, 4, 4), p("c", 2, 2, 4, 4)]);
  expect(at(out, "a")).toEqual([0, 0, 4, 4]);
  expect(at(out, "b")).toEqual([0, 4, 4, 4]);
  expect(at(out, "c")).toEqual([2, 8, 4, 4]);
});

test("normalizePanels clamps and de-overlaps stored panels", () => {
  const out = normalizePanels([p("c", 10, 0, 2, 2, "chart"), p("n", 9, 0, 3, 4)]);
  expect(at(out, "c")).toEqual([8, 0, 4, 6]);
  expect(at(out, "n")).toEqual([9, 6, 3, 4]);
  expectValid(out);
});

test("compact floats panels up without passing each other", () => {
  const out = compact([p("a", 0, 5, 4, 4), p("b", 0, 20, 4, 4), p("c", 6, 3, 4, 4)]);
  expect(at(out, "a")).toEqual([0, 0, 4, 4]);
  expect(at(out, "b")).toEqual([0, 4, 4, 4]);
  expect(at(out, "c")).toEqual([6, 0, 4, 4]);
});

test("reflow to 8 columns scales widths and keeps reading order", () => {
  const interview = builtinLayouts()[0].panels;
  const out = reflow(interview, 8);
  expectValid(out, 8);
  expect(out.map((q) => [q.id, q.x, q.y, q.w])).toEqual([
    ["d1", 0, 0, 4],
    ["d9", 4, 0, 4],
    ["dasha", 0, 12, 5],
    ["notes", 5, 12, 3],
  ]);
});

test("reflow to 4 columns stacks every panel full width in reading order", () => {
  const research = builtinLayouts()[1].panels;
  const out = reflow(research, 4);
  expectValid(out, 4);
  const sorted = [...out].sort((a, b) => a.y - b.y);
  expect(sorted.map((q) => q.id)).toEqual(["d1", "d9", "d10", "grahas", "dasha"]);
  expect(out.every((q) => q.x === 0 && q.w === 4)).toBe(true);
  expect(out.map((q) => q.h)).toEqual(research.map((q) => q.h));
});

test("reflow at 12 columns is a normalisation, not a re-layout", () => {
  const research = builtinLayouts()[1].panels;
  expect(reflow(research, 12)).toEqual(research);
});

/** Deterministik sözde rastgele (mulberry32). */
function rng(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

test.each([1, 2, 3, 4, 5])("no overlap across 300 random moves and resizes (seed %i)", (seed) => {
  const rand = rng(seed);
  const int = (lo: number, hi: number) => lo + Math.floor(rand() * (hi - lo + 1));
  for (const layout of builtinLayouts()) {
    let panels = layout.panels;
    for (let i = 0; i < 300; i++) {
      const target = panels[int(0, panels.length - 1)];
      panels =
        rand() < 0.5
          ? movePanel(panels, target.id, int(-2, 13), int(-2, 30))
          : resizePanel(panels, target.id, int(0, 14), int(0, 14));
      expectValid(panels);
    }
    for (const cols of [8, 4] as const) expectValid(reflow(panels, cols), cols);
  }
});
