import { fixture } from "../test/fixture";
import { chartDataFromChart, isVarga, signIndex, SIGNS, VARGAS } from "./adapter";

test("sign and varga lists follow the Rust enums", () => {
  expect(SIGNS).toHaveLength(12);
  expect(signIndex("Aries")).toBe(0);
  expect(signIndex("Pisces")).toBe(11);
  expect(VARGAS).toEqual(fixture().opened.chart.vargas.map((v) => v.varga));
  expect(isVarga("D60")).toBe(true);
  expect(isVarga("D5")).toBe(false);
});

test("D1 takes signs, degrees and markers from the chart without deriving anything", () => {
  const { chart } = fixture().opened;
  const data = chartDataFromChart(chart, "D1", { showLagna: true })!;
  expect(data.ascSign).toBe(signIndex(chart.lagna.sign));
  expect(data.planets).toHaveLength(9);
  for (const g of chart.grahas) {
    const p = data.planets.find((x) => x.id === g.graha)!;
    expect(p).toEqual({
      id: g.graha,
      sign: signIndex(g.sign),
      degInSign: chart.vargas[0].placements.find((x) => x.graha === g.graha)!.deg,
      retrograde: g.retrograde,
      combust: g.combust,
      vargottama: g.vargottama,
    });
  }
});

test("D9 uses its own lagna and placements; combust is D1-only, vargottama D1 and D9", () => {
  const { chart } = fixture().opened;
  chart.grahas[0].combust = true;
  chart.grahas[0].vargottama = true;
  const d9 = chart.vargas.find((v) => v.varga === "D9")!;
  const data = chartDataFromChart(chart, "D9", { showLagna: true })!;
  expect(data.ascSign).toBe(signIndex(d9.lagna_sign));
  const sun = data.planets.find((p) => p.id === "Sun")!;
  expect(sun.sign).toBe(signIndex(d9.placements.find((p) => p.graha === "Sun")!.sign));
  expect([sun.combust, sun.vargottama]).toEqual([false, true]);
  const d10 = chartDataFromChart(chart, "D10", { showLagna: true })!;
  expect(d10.planets.find((p) => p.id === "Sun")!.vargottama).toBe(false);
});

test("hidden lagna, missing vargas and null degrees", () => {
  const { chart } = fixture().opened;
  expect(chartDataFromChart(chart, "D1", { showLagna: false })!.ascSign).toBeNull();
  chart.vargas[0].placements[0].deg = null;
  expect(Number.isNaN(chartDataFromChart(chart, "D1", { showLagna: true })!.planets[0].degInSign)).toBe(true);
  chart.vargas = [];
  expect(chartDataFromChart(chart, "D1", { showLagna: true })).toBeNull();
});
