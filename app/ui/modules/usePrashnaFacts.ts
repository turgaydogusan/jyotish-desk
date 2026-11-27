// Prashna olgularını (`compute_prashna_facts`) isteyen kanca (ileri modüller spec'i §11). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Yalnızca prashna
// haritasında çağrılır: başka haritada anahtar `null`'dır ve komut çağrılmaz. Odak evi
// değişince olgular yeniden okunur; panel o sırada son okunan olguları göstermeyi sürdürür
// (`useLastData`). Girdi kişisel veridir: loglanmaz.
import { api } from "../api";
import type { OpenedClient, PrashnaFacts } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function usePrashnaFacts(opened: OpenedClient | null, focusHouse: number | null): ModuleState<PrashnaFacts> {
  const key = opened !== null && opened.prashna !== null ? JSON.stringify([inputKey(opened), focusHouse]) : null;
  return useModule(key, () => api.computePrashnaFacts(chartInput(opened as OpenedClient), focusHouse));
}
