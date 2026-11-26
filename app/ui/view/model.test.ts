import { BUILTIN_LAYOUT_IDS, defaultLayoutState } from "../layout/model";
import { fixture } from "../test/fixture";
import { chartViewOf, emptyCharaPath, initialViewModel, parseCharaPath, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
import { viewReducer } from "./reducer";

test("parseDashaPath is tolerant like the Rust store", () => {
  expect(parseDashaPath(["Rahu", "Jupiter"])).toEqual(["Rahu", "Jupiter"]);
  expect(parseDashaPath(["Rahu", "Pluto"])).toEqual([]);
  expect(parseDashaPath(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn"])).toEqual([]);
  expect(parseDashaPath("Rahu")).toEqual([]);
  expect(parseDashaPath(null)).toEqual([]);
});

test("parseCharaPath is tolerant like the Rust store", () => {
  expect(parseCharaPath({ cycle: 2, signs: ["Aries", "Taurus", "Gemini"] })).toEqual({ cycle: 2, signs: ["Aries", "Taurus", "Gemini"] });
  expect(parseCharaPath({ cycle: 1, signs: [] })).toEqual(emptyCharaPath());
  // Alanı taşımayan eski kayıt, yanlış çevrim, geçersiz burç ve dört burçluk yol: boş yol.
  for (const bad of [undefined, null, ["Aries"], { signs: ["Aries"] }, { cycle: 1 }, { cycle: 0, signs: ["Aries"] }, { cycle: 3, signs: [] }, { cycle: "1", signs: [] }, { cycle: 1, signs: ["Aries", "Ophiuchus"] }, { cycle: 1, signs: ["Aries", "Taurus", "Gemini", "Cancer"] }]) {
    expect(parseCharaPath(bad)).toEqual(emptyCharaPath());
  }
});

test("toViewState serialises the whole model in the IPC shape", () => {
  const m = initialViewModel({ active_layout_id: BUILTIN_LAYOUT_IDS.research }, { id: 3, dashaPath: ["Venus"], charaPath: { cycle: 2, signs: ["Leo"] } });
  const vs = toViewState(m);
  expect(vs).toEqual({
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: defaultLayoutState().layouts },
    dasha_expanded_path: ["Venus"],
    chara_expanded_path: { cycle: 2, signs: ["Leo"] },
  });
  expect(m.chartId).toBe(3);
  // Serileştirilen durum yeniden okunduğunda aynı modeli verir.
  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path, charaPath: vs.chara_expanded_path })).toEqual(m);
});

test("a view written before the chara path opens with the Vimshottari path and an empty chara path", () => {
  const m = initialViewModel(null, { id: 3, dashaPath: ["Rahu", "Saturn"] });
  expect([m.dashaPath, m.charaPath]).toEqual([["Rahu", "Saturn"], emptyCharaPath()]);
  expect(initialViewModel(null).charaPath).toEqual(emptyCharaPath());
  // Açılan haritanın görünüm kısmı iki yolu da taşır.
  const opened = fixture().opened;
  opened.view_state.chara_expanded_path = { cycle: 1, signs: ["Gemini"] };
  expect(chartViewOf(opened)).toEqual({ id: opened.chart_record.id, dashaPath: [], charaPath: { cycle: 1, signs: ["Gemini"] } });
});

test("the reducer keeps identity for no-op changes", () => {
  const m = initialViewModel(null);
  expect(viewReducer(m, { type: "setDashaPath", path: [] })).toBe(m);
  expect(viewReducer(m, { type: "layout", action: { type: "select", id: "nope" } })).toBe(m);
  expect(viewReducer(m, { type: "setCharaPath", path: emptyCharaPath() })).toBe(m);
  const opened = viewReducer(m, { type: "openChart", chartId: 1, dashaPath: ["Sun"] });
  expect([opened.chartId, opened.dashaPath]).toEqual([1, ["Sun"]]);
  expect(opened.layout).toBe(m.layout);
});

test("the two dasha paths change independently and both reset when the chart closes", () => {
  let m = viewReducer(initialViewModel(null), { type: "openChart", chartId: 1, dashaPath: ["Sun"], charaPath: { cycle: 1, signs: ["Aries"] } });
  expect(m.charaPath).toEqual({ cycle: 1, signs: ["Aries"] });
  m = viewReducer(m, { type: "setCharaPath", path: { cycle: 2, signs: ["Aries", "Taurus"] } });
  expect([m.dashaPath, m.charaPath]).toEqual([["Sun"], { cycle: 2, signs: ["Aries", "Taurus"] }]);
  // Aynı burçlar, başka çevrim: ayrı bir yoldur.
  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 2, signs: ["Aries", "Taurus"] } })).toBe(m);
  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 1, signs: ["Aries", "Taurus"] } })).not.toBe(m);
  m = viewReducer(m, { type: "setDashaPath", path: ["Moon"] });
  expect([m.dashaPath, m.charaPath]).toEqual([["Moon"], { cycle: 2, signs: ["Aries", "Taurus"] }]);
  // Geçersiz yol boş yola döner.
  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 5, signs: [] } }).charaPath).toEqual(emptyCharaPath());
  const closed = viewReducer(m, { type: "closeChart" });
  expect([closed.dashaPath, closed.charaPath]).toEqual([[], emptyCharaPath()]);
});
