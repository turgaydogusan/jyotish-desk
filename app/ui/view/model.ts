// Görünüm durumu (spec §5 `ViewState`, §11 M5): pano düzeni ve dasha ağacının açık dalı
// tek bir modelde tutulur ve tek bir yerden (`toViewState`) serileştirilir. Hesap ayarları
// (`CalcSettings`) burada değildir; ikisi ayrıdır (spec §4 "Ayar katmanları").
import type { Graha, ViewState } from "../bindings";
import { isGrahaId } from "../chart/types";
import { parseLayoutState, type LayoutState } from "../layout/model";

export const VIEW_STATE_SCHEMA_VERSION = 1;
/** Rust `MAX_DASHA_LEVEL`. */
export const MAX_DASHA_PATH = 5;

export interface ViewModel {
  /** Görünümün yazıldığı harita; danışan açık değilse `null` (yalnızca düzen kaydedilir). */
  chartId: number | null;
  layout: LayoutState;
  /** Dasha ağacında açık dal: kökten lord yolu (`ViewState.dasha_expanded_path`). */
  dashaPath: Graha[];
}

/** Toleranslı: dizi değilse, geçersiz lord içeriyorsa ya da 5'ten uzunsa boş yol (Rust ile aynı). */
export function parseDashaPath(raw: unknown): Graha[] {
  if (!Array.isArray(raw) || raw.length > MAX_DASHA_PATH) return [];
  return raw.every((g) => typeof g === "string" && isGrahaId(g)) ? ([...raw] as Graha[]) : [];
}

/** Açılış modeli: düzen ham kayıttan (toleranslı), dasha yolu açılan haritadan. */
export function initialViewModel(rawLayout: unknown, chart: { id: number; dashaPath: unknown } | null = null): ViewModel {
  return {
    chartId: chart?.id ?? null,
    layout: parseLayoutState(rawLayout),
    dashaPath: chart ? parseDashaPath(chart.dashaPath) : [],
  };
}

/** IPC biçimi (`save_view_state`, `save_panel_layout`). */
export function toViewState(m: ViewModel): ViewState {
  return {
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: m.layout.active_layout_id, layouts: m.layout.layouts },
    dasha_expanded_path: [...m.dashaPath],
  };
}
