import { hasOverlap } from "./grid";
import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, MAX_LAYOUTS, MAX_PANELS, type LayoutState } from "./model";
import { activeLayout, layoutReducer, newId, NOTES_PANEL, normalizeState } from "./reducer";

const { interview, research, transit, prashna } = BUILTIN_LAYOUT_IDS;

function withUser(): LayoutState {
  return layoutReducer(defaultLayoutState(), { type: "saveAs", id: "user:1", name: "Benim" });
}

test("select switches the active layout and ignores unknown ids", () => {
  const s = layoutReducer(defaultLayoutState(), { type: "select", id: research });
  expect(s.active_layout_id).toBe(research);
  expect(layoutReducer(s, { type: "select", id: "nope" })).toBe(s);
});

test("selectIndex follows layout order and ignores out-of-range", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "selectIndex", index: 2 }).active_layout_id).toBe(transit);
  expect(layoutReducer(s, { type: "selectIndex", index: 3 }).active_layout_id).toBe(prashna);
  expect(layoutReducer(s, { type: "selectIndex", index: 4 }).active_layout_id).toBe("user:1");
  expect(layoutReducer(s, { type: "selectIndex", index: 8 })).toBe(s);
});

test("saveAs copies the active layout, appends it and makes it active", () => {
  const s = withUser();
  const copy = s.layouts.at(-1)!;
  expect(copy).toMatchObject({ id: "user:1", name: "Benim", builtin: false });
  expect(copy.panels).toEqual(builtinLayout(interview)!.panels);
  expect(s.active_layout_id).toBe("user:1");
  // Derin kopya: kopyanın ayarı değişince hazır düzen etkilenmez.
  copy.panels[0].settings.style = "south";
  expect(s.layouts[0].panels[0].settings.style).toBe("north");
});

test("saveAs rejects blank names and duplicate ids", () => {
  const s = defaultLayoutState();
  expect(layoutReducer(s, { type: "saveAs", id: "user:1", name: "   " })).toBe(s);
  const u = withUser();
  expect(layoutReducer(u, { type: "saveAs", id: "user:1", name: "Again" })).toBe(u);
});

test("rename changes user layouts only", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "rename", id: "user:1", name: " Yeni " }).layouts.at(-1)!.name).toBe("Yeni");
  expect(layoutReducer(s, { type: "rename", id: interview, name: "X" })).toBe(s);
  expect(layoutReducer(s, { type: "rename", id: "user:1", name: "" })).toBe(s);
});

test("saveAs and duplicate stop at MAX_LAYOUTS so nothing is lost on the next read", () => {
  let s = defaultLayoutState();
  for (let i = s.layouts.length; i < MAX_LAYOUTS; i++) s = layoutReducer(s, { type: "saveAs", id: `user:${i}`, name: `L${i}` });
  expect(s.layouts).toHaveLength(MAX_LAYOUTS);
  expect(layoutReducer(s, { type: "saveAs", id: "user:x", name: "X" })).toBe(s);
  expect(layoutReducer(s, { type: "duplicate", sourceId: research, id: "user:y", name: "Y" })).toBe(s);
});

test("duplicate inserts a copy right after its source without changing the active layout", () => {
  const s = layoutReducer(defaultLayoutState(), { type: "duplicate", sourceId: research, id: "user:2", name: "Araştırma kopyası" });
  expect(s.layouts.map((l) => l.id)).toEqual([interview, research, "user:2", transit, prashna]);
  expect(s.layouts[2].builtin).toBe(false);
  expect(s.layouts[2].panels).toEqual(builtinLayout(research)!.panels);
  expect(s.active_layout_id).toBe(interview);
});

test("builtins cannot be deleted; deleting the active user layout falls back to the first", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "delete", id: interview })).toBe(s);
  const d = layoutReducer(s, { type: "delete", id: "user:1" });
  expect(d.layouts.map((l) => l.id)).toEqual([interview, research, transit, prashna]);
  expect(d.active_layout_id).toBe(interview);
});

test("resetBuiltin restores the preset panels", () => {
  let s = defaultLayoutState();
  s = layoutReducer(s, { type: "updatePanels", panels: [] });
  expect(activeLayout(s).panels).toEqual([]);
  s = layoutReducer(s, { type: "resetBuiltin", id: interview });
  expect(activeLayout(s).panels).toEqual(builtinLayout(interview)!.panels);
  expect(layoutReducer(s, { type: "resetBuiltin", id: "user:1" })).toBe(s);
});

test("updatePanels normalises the active layout's panels", () => {
  const s = layoutReducer(defaultLayoutState(), {
    type: "updatePanels",
    panels: [
      { id: "a", kind: "chart", x: 0, y: 0, w: 1, h: 1, settings: {} },
      { id: "b", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
    ],
  });
  const panels = activeLayout(s).panels;
  expect(panels[0]).toMatchObject({ x: 0, y: 0, w: 4, h: 6 });
  expect(hasOverlap(panels)).toBe(false);
});

test("toggleNotes removes notes when present and adds a 4-column panel on the right when absent", () => {
  let s = defaultLayoutState();
  s = layoutReducer(s, { type: "toggleNotes", panelId: "n1" });
  expect(activeLayout(s).panels.some((p) => p.kind === "notes")).toBe(false);
  s = layoutReducer(s, { type: "toggleNotes", panelId: "n2" });
  const panels = activeLayout(s).panels;
  expect(panels.find((p) => p.id === "n2")).toMatchObject({ kind: "notes", ...NOTES_PANEL });
  expect(hasOverlap(panels)).toBe(false);
  // Sağ üstteki D9 haritası notların altına itildi.
  expect(panels.find((p) => p.id === "d9")!.y).toBe(NOTES_PANEL.h);
});

test("togglePanel adds a module panel below the layout without moving the others and removes it again", () => {
  const before = activeLayout(defaultLayoutState()).panels;
  const bottom = Math.max(...before.map((p) => p.y + p.h));
  let s = layoutReducer(defaultLayoutState(), { type: "togglePanel", kind: "drishti", panelId: "dr1" });
  let panels = activeLayout(s).panels;
  expect(panels.slice(0, before.length)).toEqual(before);
  expect(panels.at(-1)).toEqual({ id: "dr1", kind: "drishti", x: 0, y: bottom, w: 6, h: 10, settings: {} });
  expect(hasOverlap(panels)).toBe(false);
  // İkinci panel öncekinin altına gelir.
  s = layoutReducer(s, { type: "togglePanel", kind: "panchang", panelId: "pa1" });
  panels = activeLayout(s).panels;
  expect(panels.at(-1)).toMatchObject({ id: "pa1", kind: "panchang", x: 0, y: bottom + 10, w: 4, h: 9 });
  expect(hasOverlap(panels)).toBe(false);
  // Aynı tür yeniden istenince kaldırılır; diğer paneller yerinde durur.
  s = layoutReducer(s, { type: "togglePanel", kind: "drishti", panelId: "dr2" });
  panels = activeLayout(s).panels;
  expect(panels.some((p) => p.kind === "drishti")).toBe(false);
  expect(panels.find((p) => p.id === "pa1")).toMatchObject({ x: 0, y: bottom + 10 });
  // Yalnızca etkin düzen değişir.
  expect(s.layouts[1]).toEqual(defaultLayoutState().layouts[1]);
});

test("togglePanel adds to an empty layout at the top and does nothing at the panel limit or on a duplicate id", () => {
  const empty = layoutReducer(defaultLayoutState(), { type: "updatePanels", panels: [] });
  const added = layoutReducer(empty, { type: "togglePanel", kind: "ashtakavarga", panelId: "av" });
  expect(activeLayout(added).panels).toEqual([{ id: "av", kind: "ashtakavarga", x: 0, y: 0, w: 6, h: 11, settings: {} }]);
  const full = layoutReducer(defaultLayoutState(), {
    type: "updatePanels",
    panels: Array.from({ length: MAX_PANELS }, (_, i) => ({ id: `n${i}`, kind: "notes", x: 0, y: i * 4, w: 3, h: 4, settings: {} })),
  });
  expect(layoutReducer(full, { type: "togglePanel", kind: "drishti", panelId: "dr" })).toEqual(full);
  const state = defaultLayoutState();
  expect(layoutReducer(state, { type: "togglePanel", kind: "drishti", panelId: "d1" })).toEqual(state);
});

test("normalizeState fixes stored geometry", () => {
  const s = defaultLayoutState();
  s.layouts[0].panels = [
    { id: "a", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
    { id: "b", kind: "notes", x: 1, y: 1, w: 3, h: 4, settings: {} },
  ];
  expect(hasOverlap(normalizeState(s).layouts[0].panels)).toBe(false);
});

test("newId yields distinct prefixed ids", () => {
  const a = newId("user");
  const b = newId("user");
  expect(a).toMatch(/^user:/);
  expect(a).not.toBe(b);
});

test("setPanelSettings merges into one panel of the active layout and keeps unknown keys", () => {
  const state = defaultLayoutState();
  const panel = activeLayout(state).panels.find((p) => p.kind === "chart")!;
  const withExtra = layoutReducer(state, { type: "setPanelSettings", panelId: panel.id, patch: { future: 1 } });
  const next = layoutReducer(withExtra, { type: "setPanelSettings", panelId: panel.id, patch: { varga: "D10" } });
  const updated = activeLayout(next).panels.find((p) => p.id === panel.id)!;
  expect(updated.settings).toEqual({ ...panel.settings, future: 1, varga: "D10" });
  expect(layoutReducer(next, { type: "setPanelSettings", panelId: "missing", patch: { varga: "D2" } })).toEqual(next);
});
