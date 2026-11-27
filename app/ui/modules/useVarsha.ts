// Varshaphala raporunu (`compute_varsha`) isteyen kanca ve yıllık haritanın yer seçimi. Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `age === null` "bu yıl"dır:
// süren yılı Rust bulur. Doğum saati bilinmiyorsa rapor yine döner: yıllık harita ve muntha
// `null`'dır (K2). Yer kişisel veridir: loglanmaz.
import { api } from "../api";
import type { AppPrefs, HomePlace, OpenedClient, VarshaView } from "../bindings";
import { chartInput, inputKey } from "./input";
import { chartPlace, muhurtaPlace } from "./muhurta";
import { useModule, type ModuleState } from "./useModule";

/** Yıllık haritanın yeri: doğum yeri ya da tercihlerdeki bulunulan yer. */
export const VARSHA_PLACES = ["birth", "home"] as const;
export type VarshaPlaceChoice = (typeof VARSHA_PLACES)[number];

/** Panelin yer seçimi; tanınmayan değer ve eksik anahtar doğum yeridir. */
export function varshaPlaceChoice(settings: Record<string, unknown>): VarshaPlaceChoice {
  return settings.place === "home" ? "home" : "birth";
}

/** Seçime göre yer; bulunulan yer seçilmemişse `null`. */
export function varshaPlace(choice: VarshaPlaceChoice, opened: OpenedClient, prefs: AppPrefs | null): HomePlace | null {
  return choice === "home" ? (prefs?.home_place ?? null) : chartPlace(opened);
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. */
export function useVarsha(opened: OpenedClient | null, age: number | null, place: HomePlace | null): ModuleState<VarshaView> {
  const at = place === null ? null : muhurtaPlace(place);
  const key = opened !== null && at !== null ? `${inputKey(opened)}|${age ?? "now"}|${JSON.stringify(at)}` : null;
  return useModule(key, () => api.computeVarsha(chartInput(opened as OpenedClient), age, at as NonNullable<typeof at>));
}
