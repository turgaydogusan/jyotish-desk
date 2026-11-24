// Düzen kalıcılığı. Geçici çözüm: M4'te `ViewState.panel_layout`'u okuyup yazan store
// komutu gelene kadar düzen localStorage'da tutulur. O komut geldiğinde yalnızca bu
// arayüzün yeni bir uygulaması yazılır; store ve pano değişmez.
import { serializeLayoutState, type LayoutState } from "./model";

export interface LayoutPersistence {
  /** Kayıtlı ham değer (ayrıştırılmamış) ya da kayıt yoksa `null`. */
  load(): unknown | null;
  save(state: LayoutState): void;
}

export const LAYOUT_STORAGE_KEY = "jyotish-desk.layout.v1";

export function localStoragePersistence(storage: Storage = window.localStorage): LayoutPersistence {
  return {
    load() {
      try {
        const text = storage.getItem(LAYOUT_STORAGE_KEY);
        return text === null ? null : (JSON.parse(text) as unknown);
      } catch (e) {
        // Bozuk JSON açılışı engellemez; varsayılan düzen kullanılır (v1 spec §7).
        console.warn("stored layout unreadable; using defaults", e);
        return null;
      }
    },
    save(state) {
      try {
        storage.setItem(LAYOUT_STORAGE_KEY, serializeLayoutState(state));
      } catch (e) {
        console.warn("layout not saved", e);
      }
    },
  };
}

/** Bellek içi kalıcılık: testler ve önizleme için. `saved` her kaydı sırayla tutar. */
export function memoryPersistence(initial: unknown = null): LayoutPersistence & { saved: LayoutState[] } {
  const saved: LayoutState[] = [];
  return {
    saved,
    load: () => (saved.length > 0 ? JSON.parse(serializeLayoutState(saved[saved.length - 1])) : initial),
    save: (state) => {
      saved.push(state);
    },
  };
}
