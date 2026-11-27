// Sankranti listesini (`sankranti_list`) isteyen kanca (yol haritası Plan 26, §4.8). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `year === null` yerin
// bugünkü yılıdır: "bugün" Rust'ta okunur. Yeni bir yıl okunurken aynı panelin son listesi
// gösterilmeyi sürdürür (tablo ve odak yerinde durur). Yer kişisel veridir: anahtar ve
// girdiler loglanmaz.
import { api } from "../api";
import type { HomePlace, OpenedClient, SankrantiListView } from "../bindings";
import { muhurtaPlace } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface SankrantiState extends ModuleState<SankrantiListView> {
  /** Gösterilecek liste: yeni yanıt gelene dek bir önceki. */
  shown: SankrantiListView | null;
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. `scope` panelin kimliğidir. */
export function useSankrantiList(opened: OpenedClient, place: HomePlace | null, year: number | null, scope: string): SankrantiState {
  const settings = opened.chart.meta.settings;
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), year, settings]) : null;
  const state = useModule(key, () => api.sankrantiList(year, muhurtaPlace(place as HomePlace), settings));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
