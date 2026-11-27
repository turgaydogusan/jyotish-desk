import { fixture } from "../test/fixture";
import { chalitData, chartDataFromChart, isVarga, signIndex, SIGNS, VARGAS } from "./adapter";

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

test("chalit data moves a graha to the cell of its bhava and marks it; sign and degree stay", () => {
  const f = fixture();
  // Tam burç: bhava burç evidir; hiçbir graha yer değiştirmez ve nesne aynen döner.
  const rasi = chartDataFromChart(f.opened.chart, "D1", { showLagna: true })!;
  const same = chalitData(f.opened.chart, rasi);
  expect(same.planets).toEqual(rasi.planets);
  expect(same.planets.some((p) => p.shifted || p.cell !== undefined)).toBe(false);
  // Placidus (fikstür): Merkür Yay'da (7. burç evi) ama 6. bhavada; Jüpiter Koç'ta (11.) ama
  // 10. bhavada; Venüs Akrep'te (6.) ama 5. bhavada. Lagna İkizler (2): n. hücrenin burcu
  // (2 + n − 1) mod 12'dir.
  const chart = f.bhava.chart_placidus;
  const data = chartDataFromChart(chart, "D1", { showLagna: true })!;
  const chalit = chalitData(chart, data);
  expect(chalit.ascSign).toBe(2);
  const moved = chalit.planets.filter((p) => p.shifted).map((p) => [p.id, p.sign, p.cell]);
  expect(moved).toEqual([
    ["Mercury", 8, 7],
    ["Jupiter", 0, 11],
    ["Venus", 7, 6],
  ]);
  for (const p of chalit.planets) {
    const before = data.planets.find((x) => x.id === p.id)!;
    expect([p.sign, p.degInSign, p.retrograde, p.combust, p.vargottama]).toEqual([before.sign, before.degInSign, before.retrograde, before.combust, before.vargottama]);
    if (!p.shifted) expect(p).toBe(before);
  }
  // Fikstür tablosuyla aynı grahalar (Rust `GrahaBhava.shifted`): arayüz ayrı bir kural uygulamaz.
  expect(chalit.planets.filter((p) => p.shifted).map((p) => p.id)).toEqual(f.bhava.table_placidus.grahas.filter((g) => g.shifted).map((g) => g.graha));
  // Girdi değişmez; transit ve işaret alanları taşınır.
  expect(data.planets.some((p) => p.shifted)).toBe(false);
  expect(chalitData(chart, { ...data, marks: [{ sign: 1, text: "AL" }] }).marks).toEqual([{ sign: 1, text: "AL" }]);
});

test("chalit data without a lagna is the data itself", () => {
  const chart = fixture().bhava.chart_placidus;
  const data = chartDataFromChart(chart, "D1", { showLagna: false })!;
  expect(data.ascSign).toBeNull();
  expect(chalitData(chart, data)).toBe(data);
});
