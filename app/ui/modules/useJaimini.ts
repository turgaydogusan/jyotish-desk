// Jaimini raporunu (`compute_jaimini`) isteyen ortak kanca. Karaka, arudha ve argala panelleri,
// harita panelinin arudha işaretleri, Bakışlar panelinin rashi türü ve graha tablosunun karaka
// sütunu aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her tüketici
// kendi isteğini yapar (komut 1 ms'nin altındadır).
import { api } from "../api";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

/** `enabled === false`: rapor gerekmiyor (ör. işaretler kapalı), komut çağrılmaz. */
export function useJaimini(opened: OpenedClient | null, enabled = true): ModuleState<JaiminiReport> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeJaimini(chartInput(opened as OpenedClient)));
}
