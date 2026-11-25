// Görünüm kalıcılığı (spec §4 otomatik kayıt): açık harita varsa `save_view_state`
// (düzen + haritanın dasha yolu), yoksa `save_panel_layout` (yalnızca düzen). Kayıtlar
// sıralıdır: aynı anda tek istek uçar, sıradaki aynı hedefe giden kayıt yalnızca en sonuyla
// gider (carried rule C5). Hatalar `onError` ile bildirilir ve loglanmaz (parametreler
// kişisel veri içerebilir).
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

export interface PersistenceEvents {
  onError?(error: CommandError): void;
  /** Bir kayıt başarıyla yazıldı (ör. önceki kayıt hatası bildirimi kaldırılır). */
  onSaved?(): void;
}

export interface StorePersistenceOptions {
  /**
   * Kayıtlı düzen okunamadıysa `true`: hiçbir görünüm kaydı gönderilmez. Bellekteki hazır
   * düzenler okunamayan kaydın üzerine yazılmaz (Plan 09 incelemesi M6-3).
   */
  hold?: boolean;
}

export function storePersistence(events: PersistenceEvents = {}, { hold = false }: StorePersistenceOptions = {}): ViewPersistence & { idle(): Promise<void> } {
  const queue: ViewSnapshot[] = [];
  let running: Promise<void> | null = null;

  const send = (s: ViewSnapshot) => (s.chartId === null ? api.savePanelLayout(s.viewState.panel_layout) : api.saveViewState(s.chartId, s.viewState));

  const pump = async () => {
    for (let next = queue.shift(); next; next = queue.shift()) {
      const r = await send(next);
      if (r.status === "error") events.onError?.(r.error);
      else events.onSaved?.();
    }
    running = null;
  };

  return {
    save(snapshot) {
      if (hold) return;
      // Uçmakta olan istek kuyrukta değildir; yalnızca bekleyen son kayıt birleştirilir.
      // Farklı haritanın kaydı birleştirilmez: ikisi de düzeni yazdığı için sıra korunur.
      const last = queue[queue.length - 1];
      if (last && last.chartId === snapshot.chartId) queue[queue.length - 1] = snapshot;
      else queue.push(snapshot);
      running ??= pump();
    },
    idle: () => running ?? Promise.resolve(),
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

export interface LoadedLayout {
  /** Kayıtlı düzen (ham; `initialViewModel` toleranslı okur) ya da `null`. */
  layout: unknown | null;
  /** `load_panel_layout` hatası: hazır düzenler kullanılır, kayıtlar tutulur. */
  error: CommandError | null;
}

/** Hata açılışı engellemez: hazır düzenler kullanılır (spec §7). */
export async function loadLayout(storage: Storage = window.localStorage): Promise<LoadedLayout> {
  const stored = await api.loadPanelLayout();
  if (stored.status === "error") return { layout: null, error: stored.error };
  if (stored.data !== null) return { layout: stored.data, error: null };
  const legacy = readLegacy(storage);
  if (legacy === null) return { layout: null, error: null };
  const parsed = parseLayoutState(legacy);
  const moved = await api.savePanelLayout({ active_layout_id: parsed.active_layout_id, layouts: parsed.layouts });
  if (moved.status === "ok") {
    try {
      storage.removeItem(LEGACY_LAYOUT_KEY);
    } catch {
      // Silinemezse bir sonraki açılışta store'daki düzen önce gelir; zararı yok.
    }
  }
  return { layout: parsed, error: null };
}
