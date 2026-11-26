import { BUILTIN_LAYOUT_IDS, defaultLayoutState } from "../layout/model";
import { fixture } from "../test/fixture";
import { chartViewOf, emptyCharaPath, emptyCompat, initialViewModel, parseCharaPath, parseCompat, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
import { viewReducer } from "./reducer";

const PARTNER = "4b0f2a8e-0000-4000-8000-00000000000a";

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
    compat_partner_uuid: null,
    compat_roles: "OpenIsGroom",
  });
  expect(m.chartId).toBe(3);
  // Serileştirilen durum yeniden okunduğunda aynı modeli verir.
  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path, charaPath: vs.chara_expanded_path })).toEqual(m);
});

test("the partner link is parsed tolerantly and serialised with the chart", () => {
  expect(parseCompat({ partnerUuid: PARTNER, roles: "OpenIsBride" })).toEqual({ partnerUuid: PARTNER, roles: "OpenIsBride" });
  // Tanınmayan rol varsayılana düşer; bağ durur.
  expect(parseCompat({ partnerUuid: PARTNER, roles: "Both" })).toEqual({ partnerUuid: PARTNER, roles: "OpenIsGroom" });
  expect(parseCompat({ partnerUuid: PARTNER })).toEqual({ partnerUuid: PARTNER, roles: "OpenIsGroom" });
  // Kanonik UUID olmayan eş (büyük harf dahil), eksik alan ve nesne olmayan değer: bağ yok, rol varsayılan.
  for (const bad of [undefined, null, "x", { roles: "OpenIsBride" }, { partnerUuid: null, roles: "OpenIsBride" }, { partnerUuid: "ayşe", roles: "OpenIsBride" }, { partnerUuid: PARTNER.toUpperCase(), roles: "OpenIsBride" }, { partnerUuid: 7 }]) {
    expect(parseCompat(bad)).toEqual(emptyCompat());
  }
  const opened = fixture().opened;
  // Fikstürdeki harita eşsizdir (Rust varsayılanı).
  expect([opened.view_state.compat_partner_uuid, opened.view_state.compat_roles]).toEqual([null, "OpenIsGroom"]);
  opened.view_state.compat_partner_uuid = PARTNER;
  opened.view_state.compat_roles = "OpenIsBride";
  const m = initialViewModel(null, chartViewOf(opened));
  expect(m.compat).toEqual({ partnerUuid: PARTNER, roles: "OpenIsBride" });
  const vs = toViewState(m);
  expect([vs.compat_partner_uuid, vs.compat_roles]).toEqual([PARTNER, "OpenIsBride"]);
  // Harita açık değilken bağ yoktur.
  expect(initialViewModel(null).compat).toEqual(emptyCompat());
});

test("the reducer sets, swaps and clears the partner link of the open chart only", () => {
  const closed = initialViewModel(null);
  // Harita açık değilken eş atanmaz, rol değişmez.
  expect(viewReducer(closed, { type: "setCompatPartner", partnerUuid: PARTNER, roles: "OpenIsGroom" })).toBe(closed);
  expect(viewReducer(closed, { type: "swapCompatRoles" })).toBe(closed);
  let m = viewReducer(closed, { type: "openChart", chartId: 1, dashaPath: [] });
  expect(m.compat).toEqual(emptyCompat());
  // Eş yokken rol değişimi yok sayılır.
  expect(viewReducer(m, { type: "swapCompatRoles" })).toBe(m);
  m = viewReducer(m, { type: "setCompatPartner", partnerUuid: PARTNER, roles: "OpenIsBride" });
  expect(m.compat).toEqual({ partnerUuid: PARTNER, roles: "OpenIsBride" });
  expect(viewReducer(m, { type: "setCompatPartner", partnerUuid: PARTNER, roles: "OpenIsBride" })).toBe(m);
  m = viewReducer(m, { type: "swapCompatRoles" });
  expect(m.compat.roles).toBe("OpenIsGroom");
  m = viewReducer(m, { type: "swapCompatRoles" });
  expect(m.compat.roles).toBe("OpenIsBride");
  // Geçersiz UUID bağı kaldırır; kaldırılan bağın rolü saklanmaz.
  expect(viewReducer(m, { type: "setCompatPartner", partnerUuid: "nope", roles: "OpenIsBride" }).compat).toEqual(emptyCompat());
  const cleared = viewReducer(m, { type: "setCompatPartner", partnerUuid: null, roles: "OpenIsBride" });
  expect(cleared.compat).toEqual(emptyCompat());
  expect([toViewState(cleared).compat_partner_uuid, toViewState(cleared).compat_roles]).toEqual([null, "OpenIsGroom"]);
  // Bağ haritanındır: başka harita kendi bağıyla açılır, kapanınca bağ sıfırlanır.
  const other = viewReducer(m, { type: "openChart", chartId: 2, dashaPath: [] });
  expect(other.compat).toEqual(emptyCompat());
  const linked = viewReducer(m, { type: "openChart", chartId: 3, dashaPath: [], compat: { partnerUuid: PARTNER, roles: "OpenIsGroom" } });
  expect(linked.compat).toEqual({ partnerUuid: PARTNER, roles: "OpenIsGroom" });
  expect(viewReducer(m, { type: "closeChart" }).compat).toEqual(emptyCompat());
});

test("a view written before the chara path opens with the Vimshottari path and an empty chara path", () => {
  const m = initialViewModel(null, { id: 3, dashaPath: ["Rahu", "Saturn"] });
  expect([m.dashaPath, m.charaPath]).toEqual([["Rahu", "Saturn"], emptyCharaPath()]);
  expect(initialViewModel(null).charaPath).toEqual(emptyCharaPath());
  // Açılan haritanın görünüm kısmı iki yolu da taşır.
  const opened = fixture().opened;
  opened.view_state.chara_expanded_path = { cycle: 1, signs: ["Gemini"] };
  expect(chartViewOf(opened)).toEqual({ id: opened.chart_record.id, dashaPath: [], charaPath: { cycle: 1, signs: ["Gemini"] }, compat: { partnerUuid: null, roles: "OpenIsGroom" } });
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
