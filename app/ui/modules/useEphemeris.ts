// Efemeris tablosunu (`ephemeris_month`) isteyen kanca (yol haritası Plan 26, §4.9). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `month === null` yerin
// bugünkü ayıdır: "bugün" Rust'ta okunur; önceki ve sonraki ay yanıtla gelir. Yeni bir ay
// okunurken aynı panelin son tablosu gösterilmeyi sürdürür (tablo ve odak yerinde durur). Yer
// kişisel veridir: anahtar ve girdiler loglanmaz.
import { api } from "../api";
import type { EphemerisMonthView, HomePlace, MonthId, OpenedClient } from "../bindings";
import { muhurtaPlace } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface EphemerisState extends ModuleState<EphemerisMonthView> {
  /** Gösterilecek tablo: yeni yanıt gelene dek bir önceki. */
  shown: EphemerisMonthView | null;
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. `scope` panelin kimliğidir. */
export function useEphemerisMonth(opened: OpenedClient, place: HomePlace | null, month: MonthId | null, scope: string): EphemerisState {
  const settings = opened.chart.meta.settings;
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), month, settings]) : null;
  const state = useModule(key, () => api.ephemerisMonth(month, muhurtaPlace(place as HomePlace), settings));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
