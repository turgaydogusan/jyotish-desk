// Transit tablosu (UI spec §3.3). Konumlar ve natal evler Rust'tan (`transits_at`); saat
// bilinmiyorsa natal ev gösterilmez (spec §7). Retro düz "R", düğümlerde yok (spec §2.3).
import { useTranslation } from "react-i18next";
import { signIndex } from "../chart/adapter";
import { RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
import { formatDegree } from "../panels/format";
import { nakshatraName } from "../panels/labels";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { browserTimeZone, formatMoment, useTransit } from "./store";
import { TransitDate } from "./TransitDate";
import "./transit.css";

export function TransitTablePanel(_: PanelProps) {
  const { t, i18n } = useTranslation();
  const { opened } = useSession();
  const transit = useTransit();
  usePanelLoadPhase(opened !== null && transit.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const unknown = timeUnknown(opened);
  const view = transit.view;
  return (
    <div className="transit-panel">
      <TransitDate />
      {transit.error ? (
        <PanelError error={transit.error} onRetry={transit.retry} />
      ) : (
        view && (
          <table className="transit-table">
            <caption className="visually-hidden">{t("transit.caption", { when: formatMoment(view.utc, i18n.language, browserTimeZone()) })}</caption>
            <thead>
              <tr>
                <th scope="col">{t("transit.col.graha")}</th>
                <th scope="col">{t("transit.col.sign")}</th>
                <th scope="col">{t("transit.col.degree")}</th>
                <th scope="col">{t("transit.col.nakshatra")}</th>
                <th scope="col">{t("transit.col.house")}</th>
                <th scope="col">{t("transit.col.retro")}</th>
              </tr>
            </thead>
            <tbody>
              {view.snapshot.grahas.map((g) => (
                <tr key={g.graha}>
                  <th scope="row">{t(`grahaName.${g.graha}`)}</th>
                  <td>{t(`sign.${signIndex(g.sign)}`)}</td>
                  <td className="num">{g.deg_in_sign === null ? "—" : formatDegree(g.deg_in_sign)}</td>
                  <td>
                    {nakshatraName(g.nakshatra)} {g.pada}
                  </td>
                  <td className="num">{unknown ? "—" : g.natal_house}</td>
                  <td>{showsRetro(g.graha as GrahaId, g.retrograde) ? RETRO_MARK : ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      )}
    </div>
  );
}
