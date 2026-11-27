// Bhava tablosunu (`compute_bhava_table`) isteyen kanca. Tablo saklanmaz ve arayüzde hesap yoktur
// (K1, C4). `data === null` (yükleme bittiğinde): doğum saati bilinmiyor (Rust `None` döner, K2).
import { api } from "../api";
import type { BhavaTable, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useBhavaTable(opened: OpenedClient | null): ModuleState<BhavaTable | null> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeBhavaTable(chartInput(opened as OpenedClient)));
}
