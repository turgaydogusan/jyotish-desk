import { useTranslation } from "react-i18next";
import { signAndDegree } from "../chart/adapter";
import { isGrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { formatDegree } from "./format";
import "./panels.css";
import { useSky } from "./sky";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Graha tablosu: graha, burç, burç içi derece, retro. Sütun seçimi (`settings.columns`) sonraki planda. */
export function GrahaTablePanel(_: PanelProps) {
  const { t } = useTranslation();
  const sky = useSky();
  const highlight = useHighlight();
  const phase = usePanelLoadPhase(sky.state.status === "loading");

  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (sky.state.rows.length === 0) return <p className="panel-message">{t("table.empty")}</p>;

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
        {sky.state.rows.map((row) => {
          const pos = row.lon === null ? null : signAndDegree(row.lon);
          const target: HighlightTarget | null = isGrahaId(row.body) ? { type: "graha", id: row.body } : null;
          const on = target !== null && sameTarget(highlight.active, target);
          const pinned = target !== null && sameTarget(highlight.pinned, target);
          return (
            <tr
              key={row.body}
              data-graha={row.body}
              className={on ? "is-highlighted" : undefined}
              onMouseEnter={target ? () => highlight.hover(target) : undefined}
              onMouseLeave={target ? () => highlight.hover(null) : undefined}
            >
              <th scope="row">
                {target ? (
                  <button
                    type="button"
                    className="link-button"
                    aria-pressed={pinned}
                    onClick={() => highlight.togglePin(target)}
                    onFocus={() => highlight.hover(target)}
                    onBlur={() => highlight.hover(null)}
                  >
                    {t(`grahaName.${row.body}`)}
                  </button>
                ) : (
                  row.body
                )}
              </th>
              <td>{pos ? t(`sign.${pos.sign}`) : "—"}</td>
              <td className="num">{pos ? formatDegree(pos.degInSign) : "—"}</td>
              <td>{row.retrograde ? "℞" : ""}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
