// Seçili Varshaphala yılı: Varshaphala paneli ile dasa ağacının mudda görünümü aynı yılı
// gösterir. Seçim açık haritaya aittir ve yalnızca bellekte durur (yol haritası Y6: yeni
// kalıcı veri yok): başka harita açılınca ya da uygulama yeniden başlayınca "bu yıl"a döner.
// `null` "bu yıl"dır: süren yılı Rust bulur, arayüz saat okumaz (C4).
import { useCallback, useSyncExternalStore } from "react";

/** Rust `MAX_VARSHA_AGE`: istenebilen en büyük yaş (tamamlanan yıl). */
export const MAX_VARSHA_AGE = 120;

let selected: { chartId: number; age: number } | null = null;
const listeners = new Set<() => void>();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function write(next: { chartId: number; age: number } | null): void {
  selected = next;
  for (const listener of listeners) listener();
}

/** Yaş istenebilir mi: 0…`MAX_VARSHA_AGE` arasında bir tamsayı. */
export function isVarshaAge(age: number): boolean {
  return Number.isInteger(age) && age >= 0 && age <= MAX_VARSHA_AGE;
}

export interface VarshaAge {
  /** Seçili yaş; `null` "bu yıl"dır. */
  age: number | null;
  /** Geçersiz yaş yok sayılır; `null` "bu yıl"a döner. */
  setAge(age: number | null): void;
}

/** `chartId` haritasının seçili yılı; seçim başka bir haritaya aitse "bu yıl". */
export function useVarshaAge(chartId: number): VarshaAge {
  const age = useSyncExternalStore(subscribe, () => (selected !== null && selected.chartId === chartId ? selected.age : null));
  const setAge = useCallback(
    (next: number | null) => {
      if (next === null) write(null);
      else if (isVarshaAge(next)) write({ chartId, age: next });
    },
    [chartId],
  );
  return { age, setAge };
}

/** Testler için: seçimi unutur. */
export function resetVarshaAge(): void {
  write(null);
}
