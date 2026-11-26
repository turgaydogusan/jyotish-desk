import { fixture } from "../test/fixture";
import { builtinLayouts } from "../layout/model";
import { foldText, localItems, matches, type LocalSources } from "./items";

test("text matching ignores case and marks and needs every word", () => {
  expect(foldText("Navāmsa İŞ", "tr")).toBe("navamsa is");
  expect(matches("D9 Navamsa", "navām", "tr")).toBe(true);
  expect(matches("Yeni harita", "har yen", "tr")).toBe(true);
  expect(matches("Yeni harita", "harita düzen", "tr")).toBe(false);
  expect(matches("Araştırma", "ARAŞ", "tr")).toBe(true);
  expect(matches("Anything", "  ", "tr")).toBe(true);
});

test("local items keep command, chart, layout, varga order and are filtered by the query", () => {
  const chart = fixture().opened.chart_record;
  const src: LocalSources = {
    charts: [chart],
    chartLabel: () => "Doğum haritası",
    layouts: builtinLayouts(),
    layoutLabel: (l) => l.id,
    vargas: [{ varga: "D9", label: "D9 Navamsa" }],
    commands: [{ command: "newChart", label: "Yeni harita" }],
    panels: [{ kind: "drishti", label: "Bakışlar panelini aç/kapat" }],
  };
  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "panel", "chart", "layout", "layout", "layout", "varga"]);
  expect(localItems(src, "bakış", "tr")).toEqual([{ kind: "panel", key: "panel:drishti", label: "Bakışlar panelini aç/kapat", panelKind: "drishti" }]);
  expect(localItems(src, "harita", "tr").map((i) => i.key)).toEqual(["command:newChart", `chart:${chart.id}`]);
  expect(localItems(src, "navamsa", "tr")).toEqual([{ kind: "varga", key: "varga:D9", label: "D9 Navamsa", varga: "D9" }]);
});
