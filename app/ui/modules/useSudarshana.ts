// Sudarshana chakrayı (`compute_sudarshana`) isteyen kanca. Sonuç saklanmaz ve arayüzde hesap
// yoktur (K1, C4); panel açıkken istenir. Doğum saati bilinmiyorsa sonuç yine döner: lagna
// halkasının evleri `null`, sonuç `approximate` işaretlidir (K2).
import { api } from "../api";
import type { OpenedClient, SudarshanaChakra } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useSudarshana(opened: OpenedClient | null): ModuleState<SudarshanaChakra> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeSudarshana(chartInput(opened as OpenedClient)));
}
