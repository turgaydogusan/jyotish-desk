import { fixture } from "../test/fixture";
import { childrenOf, formatJdDate, isOpen, samePath } from "./tree";

test("children follow the lord prefix and keep the backend order", () => {
  const { periods } = fixture().opened.chart.dasha;
  const mahas = childrenOf(periods, []);
  expect(mahas.map((p) => p.lords[0])).toEqual(["Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus", "Sun", "Moon", "Mars"]);
  const antars = childrenOf(periods, ["Jupiter"]);
  expect(antars).toHaveLength(9);
  expect(antars[0].lords).toEqual(["Jupiter", "Jupiter"]);
  expect(childrenOf(periods, ["Jupiter", "Saturn"]).every((p) => p.level === 3)).toBe(true);
  expect(childrenOf(periods, ["Jupiter", "Saturn", "Mercury"])).toEqual([]);
});

test("path helpers", () => {
  expect(samePath(["Sun", "Moon"], ["Sun", "Moon"])).toBe(true);
  expect(samePath(["Sun"], ["Sun", "Moon"])).toBe(false);
  expect(isOpen(["Sun", "Moon"], ["Sun"])).toBe(true);
  expect(isOpen(["Sun", "Moon"], ["Sun", "Mars"])).toBe(false);
  expect(isOpen([], ["Sun"])).toBe(false);
});

test("dates are shown in the birth's fixed offset", () => {
  // JD 2451545.0 = 2000-01-01T12:00Z; +05:30 aynı gün, −13:00 önceki gün.
  expect(formatJdDate(2451545.0, 19_800)).toBe("2000-01-01");
  expect(formatJdDate(2451545.0, -13 * 3600)).toBe("1999-12-31");
  expect(formatJdDate(2451544.6, 0)).toBe("2000-01-01");
  expect(formatJdDate(null, 0)).toBe("—");
});
