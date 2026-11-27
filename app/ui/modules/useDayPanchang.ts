// Gün panchangını (`day_panchang`) isteyen kanca (ileri modüller spec'i §10). Sonuç saklanmaz ve
// arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Gün, verilen anı içeren gündür: "şimdi"
// Rust'ta okunur. Yeni bir gün okunurken aynı panelin son günü gösterilmeyi sürdürür (içerik ve
// odak yerinde durur). Yer ve doğum Ay'ı kişisel veridir: anahtar ve girdiler loglanmaz.
import { api } from "../api";
import type { DayPanchangView, HomePlace, OpenedClient, TransitAt } from "../bindings";
import { muhurtaPlace, natalMoon } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface DayPanchangState extends ModuleState<DayPanchangView> {
  /** Gösterilecek gün: yeni yanıt gelene dek bir önceki. */
  shown: DayPanchangView | null;
}

/**
 * `place === null`: yer seçilmemiş, komut çağrılmaz. `epoch` aynı anın yeniden okunmasını ister
 * ("Şimdi" yeniden seçildi). `scope` panelin kimliğidir.
 */
export function useDayPanchang(opened: OpenedClient, place: HomePlace | null, at: TransitAt, epoch: number, scope: string): DayPanchangState {
  const settings = opened.chart.meta.settings;
  const natal = natalMoon(opened);
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), at, epoch, settings, natal]) : null;
  const state = useModule(key, () => api.dayPanchang(at, muhurtaPlace(place as HomePlace), settings, natal));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
