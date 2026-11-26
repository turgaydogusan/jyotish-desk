// Shadbala raporunu (`compute_shadbala`) isteyen ortak kanca. Shadbala ve Bhava bala panelleri
// aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her panel kendi isteğini
// yapar (komut 1 ms'nin altındadır). `data === null` (yükleme bittiğinde): doğum saati
// bilinmiyor ya da doğum yerinde o gün Güneş doğmuyor ya da batmıyor.
import { api } from "../api";
import type { OpenedClient, ShadbalaReport } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useShadbala(opened: OpenedClient | null): ModuleState<ShadbalaReport | null> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeShadbala(chartInput(opened as OpenedClient)));
}
