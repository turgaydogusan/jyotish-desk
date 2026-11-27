// Avastha raporunu (`compute_avasthas`) isteyen ortak kanca. Avastha paneli ve graha tablosunun
// Baladi sütunu aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her
// tüketici kendi isteğini yapar (komut 1 ms'nin altındadır). Doğum saati bilinmiyorsa rapor
// yine döner: Lajjita değerlendirilmemiş, Shayanadi hesaplanmamıştır (`time_known: false`).
import { api } from "../api";
import type { AvasthaReport, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

/** `enabled === false`: rapor gerekmiyor (ör. sütun kapalı), komut çağrılmaz. */
export function useAvasthas(opened: OpenedClient | null, enabled = true): ModuleState<AvasthaReport> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeAvasthas(chartInput(opened as OpenedClient)));
}
