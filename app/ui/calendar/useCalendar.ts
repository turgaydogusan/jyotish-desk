// Geçiş takvimini (`month_calendar`) isteyen kanca (yol haritası §4.11, Plan 29). Sonuç saklanmaz
// ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Ay, yer, süzgeç ya da ayar değişince
// yeni ay istenir ve eski isteğin geç gelen yanıtı düşer; yeni ay okunurken aynı panelin son ayı
// gösterilmeyi sürdürür (ızgara ve odak yerinde durur). `epoch` aynı isteğin yeniden okunmasını
// ister ("Bu ay" yeniden seçildi: "bugün" Rust'ta okunur). İstek yer ve (doğum türü seçiliyse)
// doğum verisi taşır: anahtar yalnızca bellekte karşılaştırılır, loglanmaz.
import { api } from "../api";
import type { CalendarRequest, CalendarView } from "../bindings";
import { useLastData, useModule, type ModuleState } from "../modules/useModule";

export interface CalendarState extends ModuleState<CalendarView> {
  /** Gösterilecek ay: yeni yanıt gelene dek bir önceki. */
  shown: CalendarView | null;
}

/** `request === null`: yer seçilmemiş, komut çağrılmaz. `scope` haritanın ve panelin kimliğidir. */
export function useCalendar(request: CalendarRequest | null, epoch: number, scope: string): CalendarState {
  const key = request === null ? null : JSON.stringify([request, epoch]);
  const state = useModule(key, () => api.monthCalendar(request as CalendarRequest));
  const shown = useLastData(state, key === null ? null : scope);
  return { ...state, shown };
}
