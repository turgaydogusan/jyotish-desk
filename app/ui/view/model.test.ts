import { BUILTIN_LAYOUT_IDS, defaultLayoutState } from "../layout/model";
import { initialViewModel, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
import { viewReducer } from "./reducer";

test("parseDashaPath is tolerant like the Rust store", () => {
  expect(parseDashaPath(["Rahu", "Jupiter"])).toEqual(["Rahu", "Jupiter"]);
  expect(parseDashaPath(["Rahu", "Pluto"])).toEqual([]);
  expect(parseDashaPath(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn"])).toEqual([]);
  expect(parseDashaPath("Rahu")).toEqual([]);
  expect(parseDashaPath(null)).toEqual([]);
});

test("toViewState serialises the whole model in the IPC shape", () => {
  const m = initialViewModel({ active_layout_id: BUILTIN_LAYOUT_IDS.research }, { id: 3, dashaPath: ["Venus"] });
  const vs = toViewState(m);
  expect(vs).toEqual({
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: defaultLayoutState().layouts },
    dasha_expanded_path: ["Venus"],
  });
  expect(m.chartId).toBe(3);
  // Serileştirilen durum yeniden okunduğunda aynı modeli verir.
  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path })).toEqual(m);
});

test("the reducer keeps identity for no-op changes", () => {
  const m = initialViewModel(null);
  expect(viewReducer(m, { type: "setDashaPath", path: [] })).toBe(m);
  expect(viewReducer(m, { type: "layout", action: { type: "select", id: "nope" } })).toBe(m);
  const opened = viewReducer(m, { type: "openChart", chartId: 1, dashaPath: ["Sun"] });
  expect([opened.chartId, opened.dashaPath]).toEqual([1, ["Sun"]]);
  expect(opened.layout).toBe(m.layout);
});
