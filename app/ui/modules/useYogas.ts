// Yoga raporunu (`compute_yogas`) isteyen kanca. Rapor saklanmaz ve arayüzde hesap yoktur
// (K1, C4); panel açıkken istenir. Doğum saati bilinmiyorsa rapor yine döner: lagna isteyen
// kurallar Rust'ta atlanmış, diğer kayıtlar `approximate` işaretlidir.
import { api } from "../api";
import type { OpenedClient, YogaReport } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useYogas(opened: OpenedClient | null): ModuleState<YogaReport> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeYogas(chartInput(opened as OpenedClient)));
}
