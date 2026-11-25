// Açık haritanın doğum verisinden türeyen gösterim kararları (hesap değil; veri Rust'tan).

import type { OpenedClient } from "../bindings";

/** Doğum saati bilinmiyor: lagna, evler ve saate bağlı değerler gösterilmez (spec §7). */
export function timeUnknown(opened: OpenedClient): boolean {
  return opened.chart_record.birth.time_certainty === "Unknown";
}
