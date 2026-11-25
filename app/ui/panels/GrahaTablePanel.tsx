import { useTranslation } from "react-i18next";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { useSession } from "../session/store";
import { formatDegree } from "./format";
import "./panels.css";
import type { PanelProps } from "./types";

/** Graha tablosu: graha, burç, burç içi derece, retro. Değerler `Chart.grahas`'tan gelir. */
export function GrahaTablePanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const highlight = useHighlight();

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;

  return (
    <table className="graha-table">
      <thead>
        <tr>
          <th scope="col">{t("table.graha")}</th>
          <th scope="col">{t("table.sign")}</th>
          <th scope="col">{t("table.degree")}</th>
          <th scope="col">{t("table.retro")}</th>
        </tr>
      </thead>
      <tbody>
        {opened.chart.grahas.map((g) => {
          const target: HighlightTarget = { type: "graha", id: g.graha };
          const on = sameTarget(highlight.active, target);
          const pinned = sameTarget(highlight.pinned, target);
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
              <td>{t(`sign.${signIndex(g.sign)}`)}</td>
              <td className="num">{g.deg_in_sign === null ? "—" : formatDegree(g.deg_in_sign)}</td>
              <td>{g.retrograde ? "℞" : ""}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
