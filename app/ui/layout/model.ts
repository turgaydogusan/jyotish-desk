// Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
// M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).

/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8, §9, §10, §11, §12). */
export const PANEL_KINDS = [
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
  "bhava_table",
  "sudarshana",
  "yogas",
  "avastha",
  "compat",
  "muhurta_day",
  "muhurta_search",
  "calendar",
  "prashna_facts",
  "event_search",
  "navatara",
  "vimshopaka",
] as const;
export type KnownPanelKind = (typeof PANEL_KINDS)[number];

/**
 * Paletten etkin düzene eklenip kaldırılan paneller ve eklenirken aldıkları boyut (Plan 14 D9).
 * Sonraki alt projeler kendi panellerini buraya ekler.
 */
export const MODULE_PANELS = [
  { kind: "panchang", w: 4, h: 9 },
  { kind: "ashtakavarga", w: 6, h: 11 },
  { kind: "drishti", w: 6, h: 10 },
  // Jaimini çekirdeği (Plan 15).
  { kind: "jaimini_karakas", w: 5, h: 14 },
  { kind: "arudha", w: 4, h: 12 },
  { kind: "argala", w: 6, h: 9 },
  // Shadbala ve Bhava bala (Plan 17).
  { kind: "shadbala", w: 8, h: 12 },
  { kind: "bhava_bala", w: 6, h: 14 },
  // Bhava tablosu (yol haritası Plan 25).
  { kind: "bhava_table", w: 7, h: 20 },
  // Sudarshana chakra (yol haritası Plan 25).
  { kind: "sudarshana", w: 6, h: 24 },
  // Yogalar (Plan 18).
  { kind: "yogas", w: 7, h: 14 },
  // Avasthalar (Plan 24).
  { kind: "avastha", w: 7, h: 16 },
  // Uyum (Plan 19).
  { kind: "compat", w: 7, h: 16 },
  // Muhurta (Plan 20).
  { kind: "muhurta_day", w: 7, h: 18 },
  { kind: "muhurta_search", w: 8, h: 18 },
  // Transit takvimi (yol haritası Plan 29).
  { kind: "calendar", w: 9, h: 24 },
  // Prashna (Plan 21).
  { kind: "prashna_facts", w: 7, h: 20 },
  // Olay araması (Plan 22).
  { kind: "event_search", w: 7, h: 18 },
  // Doğum anı ekleri (Plan 23).
  { kind: "navatara", w: 7, h: 16 },
  { kind: "vimshopaka", w: 7, h: 12 },
] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];

export type PanelSettings = Record<string, unknown>;

/**
 * `kind` bilinmeyen bir değer de olabilir (ör. daha yeni bir sürümün kaydettiği panel).
 * Bu paneller atılmaz; pano onları "tanınmıyor" yedeğiyle çizer ve düzen korunur.
 */
export interface Panel {
  id: string;
  kind: string;
  x: number;
  y: number;
  w: number;
  h: number;
  settings: PanelSettings;
}

export interface Layout {
  id: string;
  name: string;
  builtin: boolean;
  panels: Panel[];
}

export interface LayoutState {
  schema_version: number;
  active_layout_id: string;
  layouts: Layout[];
}

export const LAYOUT_SCHEMA_VERSION = 1;
/** En çok düzen (hazır düzenler dahil) ve düzen başına panel; Rust `store`
 *  `MAX_LAYOUTS`/`MAX_PANELS` ile aynı (fixtures/layout-parity.json). Sınır yazarken de
 *  uygulanır (reducer `saveAs`/`duplicate`): arayüz Rust'ın okuyacağından fazlasını yazmaz. */
export const MAX_LAYOUTS = 64;
export const MAX_PANELS = 48;
const MAX_U32 = 0xffff_ffff;

export const BUILTIN_LAYOUT_IDS = {
  interview: "builtin:interview",
  research: "builtin:research",
  transit: "builtin:transit",
  prashna: "builtin:prashna",
  classic: "builtin:classic",
} as const;
export type BuiltinLayoutId = (typeof BUILTIN_LAYOUT_IDS)[keyof typeof BUILTIN_LAYOUT_IDS];

const BUILTIN_ORDER: readonly BuiltinLayoutId[] = [
  BUILTIN_LAYOUT_IDS.interview,
  BUILTIN_LAYOUT_IDS.research,
  BUILTIN_LAYOUT_IDS.transit,
  BUILTIN_LAYOUT_IDS.prashna,
  BUILTIN_LAYOUT_IDS.classic,
];

export function isKnownPanelKind(kind: string): kind is KnownPanelKind {
  return (PANEL_KINDS as readonly string[]).includes(kind);
}

export function isBuiltinLayoutId(id: string): id is BuiltinLayoutId {
  return (BUILTIN_ORDER as readonly string[]).includes(id);
}

function chart(id: string, varga: string, x: number, y: number, w: number, h: number, transitRing = false): Panel {
  return { id, kind: "chart", x, y, w, h, settings: { varga, style: "north", transit_ring: transitRing } };
}

function panel(id: string, kind: KnownPanelKind, x: number, y: number, w: number, h: number, settings: PanelSettings = {}): Panel {
  return { id, kind, x, y, w, h, settings };
}

// 12 sütun, 32 px satır. Tarih kaydırıcı ayrı bir panel türü değildir; transit tablosunun
// `date` ayarıdır (null = şimdi).
const BUILTIN_FACTORIES: Record<BuiltinLayoutId, () => Layout> = {
  [BUILTIN_LAYOUT_IDS.interview]: () => ({
    id: BUILTIN_LAYOUT_IDS.interview,
    name: "Görüşme",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 6, 12),
      chart("d9", "D9", 6, 0, 6, 12),
      panel("dasha", "dasha_strip", 0, 12, 8, 6, { level: "antar" }),
      panel("notes", "notes", 8, 12, 4, 6),
    ],
  }),
  [BUILTIN_LAYOUT_IDS.research]: () => ({
    id: BUILTIN_LAYOUT_IDS.research,
    name: "Araştırma",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 4, 10),
      chart("d9", "D9", 4, 0, 4, 10),
      chart("d10", "D10", 8, 0, 4, 10),
      panel("grahas", "graha_table", 0, 10, 6, 9, { columns: ["lon", "nakshatra", "pada", "speed", "retro"] }),
      panel("dasha", "dasha_tree", 6, 10, 6, 9, { system: "vimshottari", open: [] }),
    ],
  }),
  [BUILTIN_LAYOUT_IDS.transit]: () => ({
    id: BUILTIN_LAYOUT_IDS.transit,
    name: "Transit",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 7, 12, true),
      panel("transits", "transit_table", 7, 0, 5, 12, { date: null }),
      panel("dasha", "dasha_strip", 0, 12, 12, 4, { level: "antar" }),
    ],
  }),
  // Prashna (ileri modüller spec'i §11): D1, olgular, anın panchangı ve notlar. Modül paneli
  // taşıyan iki hazır düzenden biridir (öteki Klasik).
  [BUILTIN_LAYOUT_IDS.prashna]: () => ({
    id: BUILTIN_LAYOUT_IDS.prashna,
    name: "Prashna",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 5, 12),
      panel("facts", "prashna_facts", 5, 0, 7, 21),
      panel("panchang", "panchang", 0, 12, 5, 9),
      panel("notes", "notes", 0, 21, 12, 5),
    ],
  }),
  // Klasik masaüstü dizilimi: solda büyük D1, sağda günün panchangı ve D9; altta Vimshottari,
  // chara dasha ve Jaimini karaka tablosu yan yana.
  [BUILTIN_LAYOUT_IDS.classic]: () => ({
    id: BUILTIN_LAYOUT_IDS.classic,
    name: "Klasik",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 7, 18),
      panel("panchang", "panchang", 7, 0, 5, 8),
      chart("d9", "D9", 7, 8, 5, 10),
      panel("vimshottari", "dasha_tree", 0, 18, 4, 12, { system: "vimshottari", open: [] }),
      panel("chara", "dasha_tree", 4, 18, 4, 12, { system: "chara", open: [] }),
      panel("karakas", "jaimini_karakas", 8, 18, 4, 12),
    ],
  }),
};

/** Hazır düzenin taze kopyası; bilinmeyen kimlikte `undefined`. */
export function builtinLayout(id: string): Layout | undefined {
  return isBuiltinLayoutId(id) ? BUILTIN_FACTORIES[id]() : undefined;
}

export function builtinLayouts(): Layout[] {
  return BUILTIN_ORDER.map((id) => BUILTIN_FACTORIES[id]());
}

export function defaultLayoutState(): LayoutState {
  return {
    schema_version: LAYOUT_SCHEMA_VERSION,
    active_layout_id: BUILTIN_LAYOUT_IDS.interview,
    layouts: builtinLayouts(),
  };
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isNonEmptyString(v: unknown): v is string {
  return typeof v === "string" && v.trim().length > 0;
}

function isInt(v: unknown, min: number): v is number {
  return typeof v === "number" && Number.isInteger(v) && v >= min && v <= MAX_U32;
}

function parsePanel(raw: unknown): Panel | null {
  if (!isRecord(raw)) return null;
  const { id, kind, x, y, w, h, settings } = raw;
  if (!isNonEmptyString(id) || !isNonEmptyString(kind)) return null;
  if (!isInt(x, 0) || !isInt(y, 0) || !isInt(w, 1) || !isInt(h, 1)) return null;
  return { id, kind, x, y, w, h, settings: isRecord(settings) ? { ...settings } : {} };
}

function parsePanels(raw: unknown): Panel[] {
  if (!Array.isArray(raw)) return [];
  const seen = new Set<string>();
  const out: Panel[] = [];
  for (const item of raw) {
    if (out.length === MAX_PANELS) break;
    const p = parsePanel(item);
    if (p === null || seen.has(p.id)) continue;
    seen.add(p.id);
    out.push(p);
  }
  return out;
}

function parseLayout(raw: unknown): Layout | null {
  if (!isRecord(raw)) return null;
  const { id, name, panels } = raw;
  if (!isNonEmptyString(id) || !isNonEmptyString(name)) return null;
  const fresh = builtinLayout(id);
  if (fresh) {
    // Hazır düzenin adı sabittir; panelleri kullanıcı değiştirmiş olabilir.
    return { ...fresh, panels: Array.isArray(panels) ? parsePanels(panels) : fresh.panels };
  }
  return { id, name: name.trim(), builtin: false, panels: parsePanels(panels) };
}

/**
 * Toleranslı ayrıştırma (v1 spec §7: bozuk görünüm durumu açılışı asla engellemez):
 * - bilinmeyen alanlar yok sayılır, geçersiz panel ve düzenler atılır,
 * - bilinmeyen panel türleri korunur,
 * - eksik hazır düzenler yeniden eklenir; hazır düzenler her zaman başta ve sabit sıradadır,
 * - geçersiz `active_layout_id` ilk hazır düzene döner,
 * - en çok `MAX_LAYOUTS` düzen (hazır düzenler dahil) ve düzen başına `MAX_PANELS` panel okunur.
 * `schema_version` yoksa 1 kabul edilir. Daha yeni bir sürüm de aynı kurallarla okunur.
 */
export function parseLayoutState(raw: unknown): LayoutState {
  if (!isRecord(raw)) return defaultLayoutState();
  const parsed = Array.isArray(raw.layouts) ? raw.layouts.map(parseLayout) : [];
  const byId = new Map<string, Layout>();
  for (const l of parsed) {
    if (byId.size === MAX_LAYOUTS) break;
    if (l !== null && !byId.has(l.id)) byId.set(l.id, l);
  }

  const builtins = BUILTIN_ORDER.map((id) => byId.get(id) ?? BUILTIN_FACTORIES[id]());
  const users = [...byId.values()].filter((l) => !l.builtin);
  const layouts = [...builtins, ...users].slice(0, MAX_LAYOUTS);

  const active = raw.active_layout_id;
  const activeId =
    typeof active === "string" && layouts.some((l) => l.id === active) ? active : BUILTIN_LAYOUT_IDS.interview;
  return { schema_version: LAYOUT_SCHEMA_VERSION, active_layout_id: activeId, layouts };
}

export function serializeLayoutState(state: LayoutState): string {
  return JSON.stringify({
    schema_version: LAYOUT_SCHEMA_VERSION,
    active_layout_id: state.active_layout_id,
    layouts: state.layouts,
  });
}
