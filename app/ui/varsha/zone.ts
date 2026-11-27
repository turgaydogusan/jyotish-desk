// Varshaphala anlarının gösterildiği ofsetin yazımı. Ofset Rust'tan gelir (yıllık haritanın
// yerinin yıl başındaki UTC ofseti); burada yalnızca biçimlenir.
import { formatFixedOffset } from "../birth/model";

/** "UTC+05:30", "UTC-03:30". */
export function zoneLabel(offsetSecs: number): string {
  return `UTC${formatFixedOffset(offsetSecs)}`;
}