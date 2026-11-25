import { i18n } from "../i18n";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { buildPrintDocument, type ChartsPage, type CoverPage, type DashaPage, type GrahasPage } from "./model";
import { defaultPrintOptions } from "./options";

const t = i18n.getFixedT("tr");
const build = (opened = fixture().opened, options = defaultPrintOptions()) => buildPrintDocument(t, opened, options);
const page = <P extends { id: string }>(doc: { pages: { id: string }[] }, id: P["id"]) => doc.pages.find((p) => p.id === id) as P;

test("the document names the client and the chart and builds only the chosen pages, in order", () => {
  const doc = build();
  expect([doc.client, doc.chart, doc.title]).toEqual(["Örnek Danışan", "Doğum haritası", "Örnek Danışan – Doğum haritası"]);
  expect(doc.pages.map((p) => p.id)).toEqual(["cover", "charts", "dasha", "grahas"]);
  expect(build(fixture().opened, { pages: ["dasha"], style: "north" }).pages.map((p) => p.id)).toEqual(["dasha"]);
  const labelled = fixture().opened;
  labelled.chart_record.label = "Evlilik";
  expect(build(labelled).title).toBe("Örnek Danışan – Evlilik");
});

test("cover: stored birth data, the settings the chart was computed with and the engine versions", () => {
  const cover = page<CoverPage>(build(), "cover");
  expect(cover.heading).toBe("Örnek Danışan");
  expect(cover.birth).toEqual(expect.arrayContaining([["Tarih", "2000-01-01"], ["Saat", "17:30"], ["Yer", "New Delhi, Delhi, India"], ["UTC anı", "2000-01-01 12:00:00 UTC"]]));
  expect(cover.settings).toEqual([
    ["Zodyak", "Sidereal"],
    ["Ayanamsa", "Lahiri 23°51′"],
    ["Düğüm", "Ortalama"],
    ["Ev sistemi", "Tam burç"],
    ["Dasa yılı", "365.25 gün"],
  ]);
  expect(cover.warnings).toEqual([]);
  expect(cover.credit).toBe("Jyotish Desk 0.9.0 · Swiss Ephemeris 2.10.03");
});

test("cover warnings carry the name and the explanation as text", () => {
  const opened = unknownTimeOpened();
  opened.chart.meta.warnings = ["LowPrecision"];
  expect(page<CoverPage>(build(opened), "cover").warnings).toEqual([
    { name: "Saat bilinmiyor", hint: "Doğum saati bilinmiyor; öğlen 12:00 alındı." },
    { name: "Düşük hassasiyet", hint: "Güneş veya Ay yedek ephemeris ile hesaplandı." },
  ]);
});

test("charts: D1 and D9 in the chosen style with the lagna Rust sent; no lagna when the time is unknown", () => {
  const charts = page<ChartsPage>(build(fixture().opened, { pages: ["charts"], style: "south" }), "charts");
  expect(charts.style).toBe("south");
  expect(charts.charts.map((c) => c.caption)).toEqual(["D1 · Rasi", "D9 · Navamsa"]);
  expect(charts.charts[0].data?.ascSign).toBe(2);
  expect(charts.charts[0].data?.planets).toHaveLength(9);
  expect(charts.charts[0].labels.title).toBe("D1 haritası");
  expect(charts.note).toBeNull();
  const unknown = page<ChartsPage>(build(unknownTimeOpened()), "charts");
  expect(unknown.charts.map((c) => c.data?.ascSign)).toEqual([null, null]);
  expect(unknown.note).toContain("Doğum saati bilinmiyor");
});

test("dasha: nine mahadashas with nine antardashas each, dated in the birth offset and contiguous", () => {
  const dasha = page<DashaPage>(build(), "dasha");
  expect(dasha.blocks.map((b) => b.lord)).toEqual(["Rahu", "Jüpiter", "Satürn", "Merkür", "Ketu", "Venüs", "Güneş", "Ay", "Mars"]);
  expect(dasha.blocks[0]).toMatchObject({ start: "1982-09-19", end: "2000-09-18" });
  const jupiter = dasha.blocks[1];
  expect(jupiter.rows).toHaveLength(9);
  expect(jupiter.rows[0]).toEqual({ lord: "Jüpiter", start: "2000-09-18", end: "2002-11-07" });
  expect(jupiter.rows[8]).toEqual({ lord: "Rahu", start: "2014-04-26", end: "2016-09-18" });
  for (const block of dasha.blocks) {
    expect(block.rows).toHaveLength(9);
    expect(block.rows[0].start).toBe(block.start);
    expect(block.rows[8].end).toBe(block.end);
    block.rows.slice(1).forEach((row, i) => expect(row.start).toBe(block.rows[i].end));
  }
  expect(dasha.note).toBeNull();
  expect(page<DashaPage>(build(unknownTimeOpened()), "dasha").note).toBe("Doğum saati bilinmiyor: dasa tarihleri yaklaşıktır (≈).");
});

test("graha table: every column, lagna first, plain R for retrograde grahas but never for the nodes", () => {
  const grahas = page<GrahasPage>(build(), "grahas");
  expect(grahas.columns.map((c) => c.id)).toEqual(["sign", "degree", "lon", "lat", "speed", "retro", "nakshatra", "pada", "combust", "dignity", "house"]);
  expect(grahas.note).toBeNull();
  expect(grahas.rows.map((r) => r.key)).toEqual(["Lagna", "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]);
  const cell = (key: string, column: string) => grahas.rows.find((r) => r.key === key)!.cells[grahas.columns.findIndex((c) => c.id === column)];
  expect(cell("Saturn", "retro")).toBe("R");
  // Fikstürde Rahu ve Ketu'nun `retrograde` alanı doğrudur; işaret yine de basılmaz.
  expect(fixture().opened.chart.grahas.filter((g) => g.retrograde).map((g) => g.graha)).toEqual(["Saturn", "Rahu", "Ketu"]);
  expect([cell("Rahu", "retro"), cell("Ketu", "retro"), cell("Sun", "retro")]).toEqual(["", "", ""]);
  expect([cell("Sun", "sign"), cell("Sun", "degree"), cell("Sun", "nakshatra"), cell("Sun", "house")]).toEqual(["Yay", "16°30′", "Purva Ashadha", "7"]);
  expect([cell("Mercury", "combust"), cell("Saturn", "dignity"), cell("Rahu", "dignity")]).toEqual(["yanık", "Düşük", "—"]);
  expect([cell("Lagna", "sign"), cell("Lagna", "degree"), cell("Lagna", "house"), cell("Lagna", "retro")]).toEqual(["İki", "16°20′", "1", ""]);
});

test("graha table with an unknown birth time: no lagna row, no house column, the Moon marked approximate", () => {
  const grahas = page<GrahasPage>(build(unknownTimeOpened()), "grahas");
  expect(grahas.columns.map((c) => c.id)).not.toContain("house");
  expect(grahas.rows.map((r) => r.key)).not.toContain("Lagna");
  // "≈" kâğıtta açıklamasız basılmaz (C9).
  expect(grahas.note).toBe("Doğum saati bilinmiyor: lagna satırı ve ev sütunu basılmaz; Ay'ın “≈” ile başlayan değerleri yaklaşıktır.");
  const moon = grahas.rows.find((r) => r.key === "Moon")!.cells;
  const at = (column: string) => grahas.columns.findIndex((c) => c.id === column);
  expect([moon[at("sign")], moon[at("degree")], moon[at("nakshatra")], moon[at("pada")]]).toEqual(["≈ Ter", "≈ 19°28′", "≈ Swati", "≈ 4"]);
  expect(moon[at("speed")]).toBe("+12.021");
  expect(grahas.rows.find((r) => r.key === "Sun")!.cells[at("sign")]).toBe("Yay");
});
