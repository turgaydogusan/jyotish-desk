import { GRID_COLUMNS, minSize } from "./grid";
import {
  BUILTIN_LAYOUT_IDS,
  builtinLayout,
  builtinLayouts,
  defaultLayoutState,
  isKnownPanelKind,
  LAYOUT_SCHEMA_VERSION,
  MAX_LAYOUTS,
  MAX_PANELS,
  MODULE_PANELS,
  PANEL_KINDS,
  parseLayoutState,
  serializeLayoutState,
  type Layout,
  type Panel,
} from "./model";
import parityRaw from "./fixtures/layout-parity.json?raw";

const ids = (layouts: Layout[]) => layouts.map((l) => l.id);
const BUILTINS = [BUILTIN_LAYOUT_IDS.interview, BUILTIN_LAYOUT_IDS.research, BUILTIN_LAYOUT_IDS.transit, BUILTIN_LAYOUT_IDS.prashna];

function overlaps(a: Panel, b: Panel) {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

test("catalog has the seven v1 panel kinds and the module panels", () => {
  expect(PANEL_KINDS).toEqual([
    "chart",
    "graha_table",
    "dasha_tree",
    "dasha_strip",
    "transit_table",
    "notes",
    "birth_info",
    "panchang",
    "ashtakavarga",
    "drishti",
    "jaimini_karakas",
    "arudha",
    "argala",
    "shadbala",
    "bhava_bala",
    "yogas",
    "compat",
    "muhurta_day",
    "muhurta_search",
    "prashna_facts",
  ]);
  expect(isKnownPanelKind("chart")).toBe(true);
  expect(isKnownPanelKind("drishti")).toBe(true);
  expect(isKnownPanelKind("yoga_list")).toBe(false);
});

test("module panels are known kinds, listed once, with a size the grid accepts", () => {
  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts"]);
  for (const m of MODULE_PANELS) {
    expect(isKnownPanelKind(m.kind)).toBe(true);
    expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
    expect(m.h).toBeGreaterThanOrEqual(minSize(m.kind).h);
    expect(m.w).toBeLessThanOrEqual(GRID_COLUMNS);
  }
  // v1'in üç hazır düzeni değişmedi: modül panelleri paletten eklenir. Modül paneli taşıyan tek
  // hazır düzen Prashna'dır (ileri modüller spec'i §11).
  for (const l of builtinLayouts()) {
    expect(l.panels.some((p) => MODULE_PANELS.some((m) => m.kind === p.kind))).toBe(l.id === BUILTIN_LAYOUT_IDS.prashna);
  }
});

test("default state has the four builtin layouts with interview active", () => {
  const s = defaultLayoutState();
  expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
  expect(s.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
  expect(ids(s.layouts)).toEqual(BUILTINS);
  expect(s.layouts.map((l) => l.name)).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna"]);
  expect(s.layouts.every((l) => l.builtin)).toBe(true);
});

test("builtin presets follow the spec text and fit the 12-column grid without overlap", () => {
  const kinds = (id: string) => builtinLayout(id)!.panels.map((p) => `${p.kind}:${p.settings.varga ?? ""}`);
  expect(kinds(BUILTIN_LAYOUT_IDS.interview)).toEqual(["chart:D1", "chart:D9", "dasha_strip:", "notes:"]);
  expect(kinds(BUILTIN_LAYOUT_IDS.research)).toEqual([
    "chart:D1",
    "chart:D9",
    "chart:D10",
    "graha_table:",
    "dasha_tree:",
  ]);
  expect(kinds(BUILTIN_LAYOUT_IDS.transit)).toEqual(["chart:D1", "transit_table:", "dasha_strip:"]);
  expect(builtinLayout(BUILTIN_LAYOUT_IDS.transit)!.panels[0].settings.transit_ring).toBe(true);
  // Prashna düzeni (ileri modüller spec'i §11): D1, olgular, anın panchangı ve notlar.
  expect(kinds(BUILTIN_LAYOUT_IDS.prashna)).toEqual(["chart:D1", "prashna_facts:", "panchang:", "notes:"]);

  for (const id of BUILTINS) {
    const panels = builtinLayout(id)!.panels;
    for (const p of panels) {
      expect(p.x + p.w, `${id}/${p.id}`).toBeLessThanOrEqual(12);
      const [minW, minH] = p.kind === "chart" ? [4, 6] : [3, 4];
      expect(p.w).toBeGreaterThanOrEqual(minW);
      expect(p.h).toBeGreaterThanOrEqual(minH);
    }
    for (const a of panels) for (const b of panels) if (a !== b) expect(overlaps(a, b), `${a.id}/${b.id}`).toBe(false);
  }
});

test("builtinLayout returns fresh copies", () => {
  const a = builtinLayout(BUILTIN_LAYOUT_IDS.interview)!;
  a.panels[0].x = 5;
  expect(builtinLayout(BUILTIN_LAYOUT_IDS.interview)!.panels[0].x).toBe(0);
  expect(builtinLayout("user:1")).toBeUndefined();
});

test.each([null, undefined, 42, "x", [], { layouts: "nope" }])("garbage %j parses to the default state", (raw) => {
  expect(parseLayoutState(raw)).toEqual(defaultLayoutState());
});

test("serialize and parse round-trip", () => {
  const s = defaultLayoutState();
  s.layouts.push({
    id: "user:a",
    name: "Benim",
    builtin: false,
    panels: [{ id: "p1", kind: "notes", x: 0, y: 0, w: 4, h: 4, settings: { a: 1 } }],
  });
  s.active_layout_id = "user:a";
  const text = serializeLayoutState(s);
  expect(JSON.parse(text).schema_version).toBe(1);
  expect(parseLayoutState(JSON.parse(text))).toEqual(s);
});

test("unknown fields are ignored, invalid panels dropped, unknown kinds kept", () => {
  const raw = {
    schema_version: 1,
    extra: true,
    active_layout_id: "user:a",
    layouts: [
      {
        id: "user:a",
        name: "  A  ",
        builtin: false,
        color: "red",
        panels: [
          { id: "ok", kind: "chart", x: 0, y: 0, w: 4, h: 6, settings: { style: "south" }, junk: 1 },
          { id: "future", kind: "yoga_list", x: 4, y: 0, w: 3, h: 4 },
          { id: "neg", kind: "notes", x: -1, y: 0, w: 3, h: 4, settings: {} },
          { id: "frac", kind: "notes", x: 0, y: 0.5, w: 3, h: 4, settings: {} },
          { id: "zero", kind: "notes", x: 0, y: 0, w: 0, h: 4, settings: {} },
          { id: "", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
          { id: "nokind", x: 0, y: 0, w: 3, h: 4, settings: {} },
          { id: "ok", kind: "notes", x: 0, y: 20, w: 3, h: 4, settings: {} },
          "garbage",
        ],
      },
    ],
  };
  const s = parseLayoutState(raw);
  const a = s.layouts.find((l) => l.id === "user:a")!;
  expect(a.name).toBe("A");
  expect(a).not.toHaveProperty("color");
  expect(a.panels).toEqual([
    { id: "ok", kind: "chart", x: 0, y: 0, w: 4, h: 6, settings: { style: "south" } },
    { id: "future", kind: "yoga_list", x: 4, y: 0, w: 3, h: 4, settings: {} },
  ]);
  expect(s.active_layout_id).toBe("user:a");
});

test("missing builtins are re-added in canonical order before user layouts", () => {
  const s = parseLayoutState({
    active_layout_id: "builtin:research",
    layouts: [
      { id: "user:x", name: "X", builtin: false, panels: [] },
      { id: "builtin:research", name: "Renamed", builtin: true, panels: [] },
    ],
  });
  expect(ids(s.layouts)).toEqual([...BUILTINS, "user:x"]);
  const research = s.layouts[1];
  expect(research.name).toBe("Araştırma");
  expect(research.builtin).toBe(true);
  expect(research.panels).toEqual([]);
  expect(s.active_layout_id).toBe("builtin:research");
});

test("a layout claiming builtin with a non-builtin id becomes a user layout", () => {
  const s = parseLayoutState({ layouts: [{ id: "user:y", name: "Y", builtin: true, panels: [] }] });
  expect(s.layouts.find((l) => l.id === "user:y")!.builtin).toBe(false);
});

test("bad or missing active id falls back to the first builtin", () => {
  expect(parseLayoutState({ active_layout_id: "gone", layouts: [] }).active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
  expect(parseLayoutState({ active_layout_id: 7, layouts: [] }).active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
});

test("duplicate layout ids keep the first occurrence", () => {
  const s = parseLayoutState({
    layouts: [
      { id: "user:d", name: "First", builtin: false, panels: [] },
      { id: "user:d", name: "Second", builtin: false, panels: [] },
    ],
  });
  expect(s.layouts.filter((l) => l.id === "user:d").map((l) => l.name)).toEqual(["First"]);
});

test("a newer schema version is read best-effort", () => {
  const s = parseLayoutState({ schema_version: 99, active_layout_id: "builtin:transit", layouts: [] });
  expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
  expect(s.active_layout_id).toBe("builtin:transit");
});

// Rust `store::parse_panel_layout` ile ortak fikstür (Plan 09): iki ayrıştırıcı aynı
// düzenleri ve panelleri tutar. Arayüz ayrıca eksik hazır düzenleri ekler; onlar fikstürde
// yoktur ve karşılaştırmaya girmez.
interface ParityCase {
  name: string;
  input: unknown;
  layouts: { id: string; panels: string[] }[];
}
const parity = JSON.parse(parityRaw) as { max_layouts: number; max_panels: number; cases: ParityCase[] };

test("caps are the same numbers as the Rust store", () => {
  expect([MAX_LAYOUTS, MAX_PANELS]).toEqual([parity.max_layouts, parity.max_panels]);
});

test.each(parity.cases.map((c) => [c.name, c] as const))("parity: %s", (_, c) => {
  const keep = new Set(c.layouts.map((l) => l.id));
  const got = parseLayoutState(c.input)
    .layouts.filter((l) => keep.has(l.id) || !l.builtin)
    .map((l) => ({ id: l.id, panels: l.panels.map((p) => p.id) }));
  const byId = (a: { id: string }, b: { id: string }) => a.id.localeCompare(b.id);
  expect(got.sort(byId)).toEqual([...c.layouts].sort(byId));
});

test("layout and panel counts are capped like the Rust store", () => {
  const panels = Array.from({ length: MAX_PANELS + 10 }, (_, i) => ({ id: `p${i}`, kind: "chart", x: 0, y: i, w: 3, h: 4 }));
  const layouts = Array.from({ length: MAX_LAYOUTS + 6 }, (_, i) => ({ id: `user:${i}`, name: "L", panels }));
  const all = parseLayoutState({ layouts }).layouts;
  const users = all.filter((l) => !l.builtin);
  // Hazır düzenler sınırın içindedir: toplam `MAX_LAYOUTS`.
  expect(all).toHaveLength(MAX_LAYOUTS);
  expect(users).toHaveLength(MAX_LAYOUTS - BUILTINS.length);
  expect(users.every((l) => l.panels.length === MAX_PANELS)).toBe(true);
  expect(users[MAX_LAYOUTS - BUILTINS.length - 1].id).toBe(`user:${MAX_LAYOUTS - BUILTINS.length - 1}`);
});
