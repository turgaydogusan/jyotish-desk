// Graha tablosu hücrelerinin metni. Pano paneli (GrahaTablePanel) ve baskı belgesi
// (print/model.ts) aynı metni kullanır; değerler Rust'tan hazır gelir, burada yalnızca biçimlenir.
import type { TFunction } from "i18next";
import type { GrahaPosition, Lagna } from "../bindings";
import { signIndex } from "../chart/adapter";
import { RETRO_MARK, showsRetro } from "../chart/types";
import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
import type { GrahaColumn } from "./grahaColumns";
import { nakshatraName } from "./labels";

export const NUMERIC_COLUMNS: ReadonlySet<GrahaColumn> = new Set(["degree", "lon", "lat", "speed", "pada", "house"]);
/** Doğum saati bilinmiyorsa Ay'da "≈" ile işaretlenen (saate bağlı) sütunlar. */
export const APPROX_COLUMNS: ReadonlySet<GrahaColumn> = new Set(["sign", "degree", "lon", "nakshatra", "pada"]);
export const DASH = "—";

function deg(v: number | null): string {
  return v === null ? DASH : formatDegree(v);
}

/** Retro sütunu düz "R"dir ve Rahu ile Ketu'da boştur (`showsRetro`). */
export function grahaCellText(t: TFunction, g: GrahaPosition, column: GrahaColumn): string {
  switch (column) {
    case "sign":
      return t(`sign.${signIndex(g.sign)}`);
    case "degree":
      return deg(g.deg_in_sign);
    case "lon":
      return deg(g.lon);
    case "lat":
      return g.lat === null ? DASH : formatSignedDegree(g.lat);
    case "speed":
      return g.speed === null ? DASH : formatSpeed(g.speed);
    case "retro":
      return showsRetro(g.graha, g.retrograde) ? RETRO_MARK : "";
    case "nakshatra":
      return nakshatraName(g.nakshatra);
    case "pada":
      return String(g.pada);
    case "combust":
      return g.combust ? t("chart.combust") : "";
    case "dignity":
      return g.dignity === null ? DASH : t(`dignity.${g.dignity}`);
    case "house":
      return String(g.house);
  }
}

export function lagnaCellText(t: TFunction, l: Lagna, column: GrahaColumn): string {
  switch (column) {
    case "sign":
      return t(`sign.${signIndex(l.sign)}`);
    case "degree":
      return deg(l.deg_in_sign);
    case "lon":
      return deg(l.lon);
    case "nakshatra":
      return nakshatraName(l.nakshatra);
    case "pada":
      return String(l.pada);
    case "house":
      return "1";
    default:
      return "";
  }
}
