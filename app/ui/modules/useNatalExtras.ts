// Doğum anı eklerinin (Plan 23) kancaları: `compute_panchang_extras` ve `compute_navatara`.
// Sonuç saklanmaz, panel açıkken istenir (K1); arayüzde hesap yoktur (C4). İkisi de lagna
// istemez: doğum saati bilinmiyorsa sonuç yine gelir ve `approximate` işaretlidir.
import { api } from "../api";
import type { Navatara, OpenedClient, PanchangExtras } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function usePanchangExtras(opened: OpenedClient | null): ModuleState<PanchangExtras> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computePanchangExtras(chartInput(opened as OpenedClient)));
}

/**
 * `transitJdUt` transit anıdır (`TransitView.snapshot.jd_ut`); `null` ise yalnızca doğum tablosu
 * istenir. An değişince rapor yeniden istenir; arayüz an hesabı yapmaz.
 */
export function useNavatara(opened: OpenedClient | null, transitJdUt: number | null): ModuleState<Navatara> {
  return useModule(opened !== null ? JSON.stringify([inputKey(opened), transitJdUt]) : null, () => api.computeNavatara(chartInput(opened as OpenedClient), transitJdUt));
}
