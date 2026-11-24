import { arrowDelta, GRID_GAP_PX, gridMetrics, pxToGridDelta } from "./metrics";

test("column width accounts for gaps", () => {
  const m = gridMetrics(1200 + 11 * GRID_GAP_PX, 12);
  expect(m.colWidth).toBe(100);
  expect(m.rowHeight).toBe(32);
  expect(gridMetrics(0, 4).colWidth).toBe(0);
});

test("pixel deltas snap to the nearest cell", () => {
  const m = gridMetrics(1200 + 11 * GRID_GAP_PX, 12); // sütun adımı 108, satır adımı 40
  expect(pxToGridDelta(0, 0, m)).toEqual({ dx: 0, dy: 0 });
  expect(pxToGridDelta(53, 19, m)).toEqual({ dx: 0, dy: 0 });
  expect(pxToGridDelta(55, 21, m)).toEqual({ dx: 1, dy: 1 });
  expect(pxToGridDelta(-216, 120, m)).toEqual({ dx: -2, dy: 3 });
  expect(pxToGridDelta(500, 0, gridMetrics(0, 12))).toEqual({ dx: 0, dy: 0 });
});

test("arrow keys map to unit steps", () => {
  expect(arrowDelta("ArrowLeft")).toEqual({ dx: -1, dy: 0 });
  expect(arrowDelta("ArrowDown")).toEqual({ dx: 0, dy: 1 });
  expect(arrowDelta("a")).toBeNull();
});
