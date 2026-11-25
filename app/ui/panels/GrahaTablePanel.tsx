import type { TFunction } from "i18next";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { GrahaPosition, Lagna } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { timeUnknown } from "./ChartPanel";
import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
import { nakshatraName } from "./labels";
import "./panels.css";
import type { PanelProps } from "./types";

const NUMERIC: ReadonlySet<GrahaColumn> = new Set(["degree", "lon", "lat", "speed", "pada", "house"]);
const dash = "—";

function deg(v: number | null): string {
  return v === null ? dash : formatDegree(v);
}

/** Yaklaşık değer: "≈" işareti ve ekran okuyucu için metin (renk tek başına taşımaz). */
function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
  if (!on) return <>{children}</>;
  return (
    <span title={label}>
      <span aria-hidden="true">≈</span>
      <span className="visually-hidden">{label} </span>
      {children}
    </span>
  );
}

function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx: boolean): ReactNode {
  const a = (node: ReactNode) => (
    <Approx on={approx} label={t("table.approx")}>
      {node}
    </Approx>
  );
  switch (column) {
    case "sign":
      return a(t(`sign.${signIndex(g.sign)}`));
    case "degree":
      return a(deg(g.deg_in_sign));
    case "lon":
      return a(deg(g.lon));
    case "lat":
      return g.lat === null ? dash : formatSignedDegree(g.lat);
    case "speed":
      return g.speed === null ? dash : formatSpeed(g.speed);
    case "retro":
      return g.retrograde ? (
        <>
          <span aria-hidden="true">℞</span>
          <span className="visually-hidden">{t("chart.retro")}</span>
        </>
      ) : (
        ""
      );
    case "nakshatra":
      return a(nakshatraName(g.nakshatra));
    case "pada":
      return a(String(g.pada));
    case "combust":
      return g.combust ? t("chart.combust") : "";
    case "dignity":
      return g.dignity === null ? dash : t(`dignity.${g.dignity}`);
    case "house":
      return String(g.house);
  }
}

function lagnaCell(t: TFunction, l: Lagna, column: GrahaColumn): ReactNode {
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

/**
 * Graha tablosu (spec §3.3): sütunlar `settings.columns`'tan, değerler `Chart.grahas` ve
 * `Chart.lagna`'dan. Saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate
 * bağlı değerleri "≈" ile işaretlenir.
 */
export function GrahaTablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;

  const unknown = timeUnknown(opened);
  const chosen = grahaColumns(panel.settings);
  const columns = unknown ? chosen.filter((c) => c !== "house") : chosen;
  const setColumns = (next: GrahaColumn[]) => setPanelSettings(panel.id, { columns: next });

  return (
    <div className="graha-table-panel">
      <details className="column-chooser">
        <summary>{t("table.columns")}</summary>
        <fieldset>
          <legend className="visually-hidden">{t("table.columns")}</legend>
          {GRAHA_COLUMNS.map((c) => (
            <label key={c}>
              <input
                type="checkbox"
                checked={chosen.includes(c)}
                disabled={chosen.length === 1 && chosen.includes(c)}
                onChange={() => setColumns(toggleColumn(chosen, c))}
              />
              {t(`table.${c}`)}
            </label>
          ))}
        </fieldset>
      </details>
      <table className="graha-table">
        <thead>
          <tr>
            <th scope="col">{t("table.graha")}</th>
            {columns.map((c) => (
              <th key={c} scope="col" className={NUMERIC.has(c) ? "num" : undefined}>
                {t(`table.${c}`)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {!unknown && (
            <tr data-lagna="">
              <th scope="row">{t("table.lagnaRow")}</th>
              {columns.map((c) => (
                <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
                  {lagnaCell(t, opened.chart.lagna, c)}
                </td>
              ))}
            </tr>
          )}
          {opened.chart.grahas.map((g) => {
            const target: HighlightTarget = { type: "graha", id: g.graha };
            const on = sameTarget(highlight.active, target);
            const pinned = sameTarget(highlight.pinned, target);
            const approx = unknown && g.graha === "Moon";
            return (
              <tr
                key={g.graha}
                data-graha={g.graha}
                className={on ? "is-highlighted" : undefined}
                onMouseEnter={() => highlight.hover(target)}
                onMouseLeave={() => highlight.hover(null)}
              >
                <th scope="row">
                  <button
                    type="button"
                    className="link-button"
                    aria-pressed={pinned}
                    onClick={() => highlight.togglePin(target)}
                    onFocus={() => highlight.hover(target)}
                    onBlur={() => highlight.hover(null)}
                  >
                    {t(`grahaName.${g.graha}`)}
                  </button>
                </th>
                {columns.map((c) => (
                  <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
                    {grahaCell(t, g, c, approx)}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
