// Görünüm durumu (spec §5 `ViewState`, §11 M5): pano düzeni ve dasha ağacının açık dalı
// tek bir modelde tutulur ve tek bir yerden (`toViewState`) serileştirilir. Hesap ayarları
// (`CalcSettings`) burada değildir; ikisi ayrıdır (spec §4 "Ayar katmanları"). Açık dal dasa
// sistemi başına ayrıdır (ileri modüller spec'i §6): Vimshottari'nin lord yolu ve chara
// dashanın burç yolu.
import type { Graha, Sign, SignDashaPath, ViewState } from "../bindings";
import { SIGNS } from "../chart/adapter";
import { isGrahaId } from "../chart/types";
import { parseLayoutState, type LayoutState } from "../layout/model";

export const VIEW_STATE_SCHEMA_VERSION = 1;
/** Rust `MAX_DASHA_LEVEL`. */
export const MAX_DASHA_PATH = 5;
/** Rust `MAX_SIGN_DASHA_LEVEL`. */
export const MAX_CHARA_PATH = 3;

/** Boş chara dasha yolu: ilk çevrim, açık dal yok (Rust `empty_chara_path`). */
export function emptyCharaPath(): SignDashaPath {
  return { cycle: 1, signs: [] };
}

export interface ViewModel {
  /** Görünümün yazıldığı harita; danışan açık değilse `null` (yalnızca düzen kaydedilir). */
  chartId: number | null;
  layout: LayoutState;
  /** Vimshottari ağacında açık dal: kökten lord yolu (`ViewState.dasha_expanded_path`). */
  dashaPath: Graha[];
  /** Chara dasha ağacında açık dal: çevrim ve burç yolu (`ViewState.chara_expanded_path`). */
  charaPath: SignDashaPath;
}

/** Toleranslı: dizi değilse, geçersiz lord içeriyorsa ya da 5'ten uzunsa boş yol (Rust ile aynı). */
export function parseDashaPath(raw: unknown): Graha[] {
  if (!Array.isArray(raw) || raw.length > MAX_DASHA_PATH) return [];
  return raw.every((g) => typeof g === "string" && isGrahaId(g)) ? ([...raw] as Graha[]) : [];
}

/**
 * Toleranslı: nesne değilse, çevrim 1 ya da 2 değilse, geçersiz burç içeriyorsa ya da 3'ten
 * uzunsa boş yol (Rust `parse_chara_path` ile aynı). Alanı taşımayan eski kayıt da boş yoldur.
 */
export function parseCharaPath(raw: unknown): SignDashaPath {
  if (typeof raw !== "object" || raw === null) return emptyCharaPath();
  const { cycle, signs } = raw as { cycle?: unknown; signs?: unknown };
  if ((cycle !== 1 && cycle !== 2) || !Array.isArray(signs) || signs.length > MAX_CHARA_PATH) return emptyCharaPath();
  if (!signs.every((s) => typeof s === "string" && (SIGNS as readonly string[]).includes(s))) return emptyCharaPath();
  return { cycle, signs: [...signs] as Sign[] };
}

export function sameCharaPath(a: SignDashaPath, b: SignDashaPath): boolean {
  return a.cycle === b.cycle && a.signs.length === b.signs.length && a.signs.every((s, i) => s === b.signs[i]);
}

/** Açılan haritanın görünüm kısmı: kimlik ve iki dasha yolu (ham; toleranslı okunur). */
export interface ChartView {
  id: number;
  dashaPath: unknown;
  charaPath?: unknown;
}

/** `OpenedClient`'tan görünüm kısmı. */
export function chartViewOf(opened: { chart_record: { id: number }; view_state: ViewState }): ChartView {
  return { id: opened.chart_record.id, dashaPath: opened.view_state.dasha_expanded_path, charaPath: opened.view_state.chara_expanded_path };
}

/** Açılış modeli: düzen ham kayıttan (toleranslı), dasha yolları açılan haritadan. */
export function initialViewModel(rawLayout: unknown, chart: ChartView | null = null): ViewModel {
  return {
    chartId: chart?.id ?? null,
    layout: parseLayoutState(rawLayout),
    dashaPath: chart ? parseDashaPath(chart.dashaPath) : [],
    charaPath: chart ? parseCharaPath(chart.charaPath) : emptyCharaPath(),
  };
}

/** IPC biçimi (`save_view_state`, `save_panel_layout`). */
export function toViewState(m: ViewModel): ViewState {
  return {
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: m.layout.active_layout_id, layouts: m.layout.layouts },
    dasha_expanded_path: [...m.dashaPath],
    chara_expanded_path: { cycle: m.charaPath.cycle, signs: [...m.charaPath.signs] },
  };
}
