// Paneller arası vurgu hedefi (spec §3.4).
import type { GrahaId } from "../chart/types";

export type HighlightTarget =
  | { type: "graha"; id: GrahaId }
  | { type: "house"; house: number }
  | { type: "sign"; sign: number };

export function sameTarget(a: HighlightTarget | null, b: HighlightTarget | null): boolean {
  if (a === null || b === null) return a === b;
  if (a.type === "graha" && b.type === "graha") return a.id === b.id;
  if (a.type === "house" && b.type === "house") return a.house === b.house;
  if (a.type === "sign" && b.type === "sign") return a.sign === b.sign;
  return false;
}
