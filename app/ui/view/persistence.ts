// Görünüm kalıcılığı (spec §4 otomatik kayıt): açık harita varsa `save_view_state`
// (düzen + haritanın dasha yolu), yoksa `save_panel_layout` (yalnızca düzen). Hatalar
// `onError` ile bildirilir ve loglanmaz (parametreler kişisel veri içerebilir).
import { api } from "../api";
import type { CommandError, ViewState } from "../bindings";
import { parseLayoutState } from "../layout/model";

export interface ViewSnapshot {
  chartId: number | null;
  viewState: ViewState;
}

export interface ViewPersistence {
  save(snapshot: ViewSnapshot): void;
}

export function storePersistence(onError: (error: CommandError) => void = () => {}): ViewPersistence {
  return {
    save({ chartId, viewState }) {
      const call = chartId === null ? api.savePanelLayout(viewState.panel_layout) : api.saveViewState(chartId, viewState);
      void call.then((r) => {
        if (r.status === "error") onError(r.error);
      });
    },
  };
}

/** Bellek içi kalıcılık: testler için. `saved` her kaydı sırayla tutar. */
export function memoryPersistence(): ViewPersistence & { saved: ViewSnapshot[] } {
  const saved: ViewSnapshot[] = [];
  return { saved, save: (s) => void saved.push(structuredClone(s)) };
}

/** Plan 04'te düzen localStorage'daydı; store'da düzen yoksa bir kez oraya taşınır. */
export const LEGACY_LAYOUT_KEY = "jyotish-desk.layout.v1";

function readLegacy(storage: Storage): unknown | null {
  try {
    const text = storage.getItem(LEGACY_LAYOUT_KEY);
    return text === null ? null : (JSON.parse(text) as unknown);
  } catch {
    return null;
  }
}

/**
 * Kayıtlı düzen (ham; `initialViewModel` toleranslı okur) ya da `null`. Hata açılışı
 * engellemez: hazır düzenler kullanılır (spec §7).
 */
export async function loadLayout(storage: Storage = window.localStorage): Promise<unknown | null> {
  const stored = await api.loadPanelLayout();
  if (stored.status === "error") return null;
  if (stored.data !== null) return stored.data;
  const legacy = readLegacy(storage);
  if (legacy === null) return null;
  const parsed = parseLayoutState(legacy);
  const moved = await api.savePanelLayout({ active_layout_id: parsed.active_layout_id, layouts: parsed.layouts });
  if (moved.status === "ok") {
    try {
      storage.removeItem(LEGACY_LAYOUT_KEY);
    } catch {
      // Silinemezse bir sonraki açılışta store'daki düzen önce gelir; zararı yok.
    }
  }
  return parsed;
}
