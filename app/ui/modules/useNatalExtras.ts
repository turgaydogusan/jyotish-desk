// Doğum anı eklerinin (Plan 23) kancaları: `compute_panchang_extras`, `compute_navatara` ve
// `compute_vimshopaka`. Sonuç saklanmaz, panel açıkken istenir (K1); arayüzde hesap yoktur
// (C4). Üçü de lagna istemez: doğum saati bilinmiyorsa sonuç yine gelir ve `approximate` işaretlidir.
import { api } from "../api";
import type { Navatara, OpenedClient, PanchangExtras, VimshopakaReport } from "../bindings";
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

export function useVimshopaka(opened: OpenedClient | null): ModuleState<VimshopakaReport> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeVimshopaka(chartInput(opened as OpenedClient)));
}
