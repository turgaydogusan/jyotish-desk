import type { TFunction } from "i18next";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { GrahaPosition } from "../bindings";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { useAvasthas } from "../modules/useAvasthas";
import { useJaimini } from "../modules/useJaimini";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { timeUnknown } from "../session/birth";
import { Approx } from "./Approx";
import { BALADI_NAMES } from "./avasthaLabels";
import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
import { KARAKA_ROLES } from "./labels";
import { nakshatraLord } from "./nakshatraLord";
import "./panels.css";
import { PanelError } from "./states";
import type { PanelProps } from "./types";

/** Hücre metni `grahaCells.ts`'ten; burada yalnızca erişilebilirlik sarmalayıcıları eklenir. */
function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx: boolean): ReactNode {
  const text = grahaCellText(t, g, column);
  if (column === "retro" && text !== "") {
    return (
      <>
        <span aria-hidden="true">{text}</span>
        <span className="visually-hidden">{t("chart.retro")}</span>
      </>
    );
  }
  if (!APPROX_COLUMNS.has(column)) return text;
  return (
    <Approx on={approx} label={t("table.approx")}>
      {text}
    </Approx>
  );
}

/**
 * Graha tablosu (spec §3.3): sütunlar `settings.columns`'tan, değerler `Chart.grahas` ve
 * `Chart.lagna`'dan. Saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate
 * bağlı değerleri "≈" ile işaretlenir. İsteğe bağlı "Karaka" sütunu (`settings.karaka`)
 * chara karaka rolünü `compute_jaimini`'den okur; açık değilken komut çağrılmaz. Sütun
 * `columns` listesinde değildir: baskı tablosu o listeyi kullanır ve değişmez (K10). İsteğe
 * bağlı "Baladi" sütunu (`settings.baladi`) aynı kalıptır: değer `compute_avasthas`'tan okunur,
 * açık değilken komut çağrılmaz; Rahu, Ketu ve lagna satırında boştur. İsteğe bağlı "Nakshatra
 * lordu" sütunu (`settings.nakshatraLord`; Plan 26) komut çağırmaz: lord, satırın nakshatra
 * numarasıyla Rust'ın tablosundan okunur (`nakshatraLord`); lagna satırında da doludur.
 */
export function GrahaTablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const karakaOn = panel.settings.karaka === true;
  const jaimini = useJaimini(opened, karakaOn);
  const baladiOn = panel.settings.baladi === true;
  const avasthas = useAvasthas(opened, baladiOn);
  const lordOn = panel.settings.nakshatraLord === true;
  const lordName = (nakshatra: number) => {
    const lord = nakshatraLord(nakshatra);
    return lord === null ? "—" : t(`grahaName.${lord}`);
  };

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
  const roles = new Map((jaimini.data?.karakas ?? []).map((k) => [k.graha, KARAKA_ROLES[k.role]]));
  const baladi = new Map((avasthas.data?.grahas ?? []).map((a) => [a.graha, BALADI_NAMES[a.baladi]]));

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
          <label>
            <input type="checkbox" checked={karakaOn} onChange={(e) => setPanelSettings(panel.id, { karaka: e.target.checked })} />
            {t("table.karaka")}
          </label>
          <label>
            <input type="checkbox" checked={baladiOn} onChange={(e) => setPanelSettings(panel.id, { baladi: e.target.checked })} />
            {t("table.baladi")}
          </label>
          <label>
            <input type="checkbox" checked={lordOn} onChange={(e) => setPanelSettings(panel.id, { nakshatraLord: e.target.checked })} />
            {t("table.nakshatraLord")}
          </label>
        </fieldset>
      </details>
      {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
      {avasthas.error && <PanelError error={avasthas.error} onRetry={avasthas.retry} />}
      <table className="graha-table">
        <thead>
          <tr>
            <th scope="col">{t("table.graha")}</th>
            {columns.map((c) => (
              <th key={c} scope="col" className={NUMERIC.has(c) ? "num" : undefined}>
                {t(`table.${c}`)}
              </th>
            ))}
            {karakaOn && <th scope="col">{t("table.karaka")}</th>}
            {baladiOn && <th scope="col">{t("table.baladi")}</th>}
            {lordOn && <th scope="col">{t("table.nakshatraLord")}</th>}
          </tr>
        </thead>
        <tbody>
          {!unknown && (
            <tr data-lagna="">
              <th scope="row">{t("table.lagnaRow")}</th>
              {columns.map((c) => (
                <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
                  {lagnaCellText(t, opened.chart.lagna, c)}
                </td>
              ))}
              {karakaOn && <td />}
              {baladiOn && <td />}
              {lordOn && <td data-nakshatra-lord="lagna">{lordName(opened.chart.lagna.nakshatra)}</td>}
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
                {karakaOn && (
                  <td data-karaka={g.graha}>
                    {/* Ay'ın rolü yaklaşık dereceden çıkar: Ay'ın diğer hücreleri gibi "≈" taşır (K2). */}
                    {roles.has(g.graha) && (
                      <Approx on={approx} label={t("table.approx")}>
                        <abbr title={roles.get(g.graha)!.name}>{roles.get(g.graha)!.short}</abbr>
                        <span className="visually-hidden"> {roles.get(g.graha)!.name}</span>
                      </Approx>
                    )}
                  </td>
                )}
                {baladiOn && (
                  <td data-baladi={g.graha}>
                    {/* Ay'ın basamağı yaklaşık dereceden çıkar: "≈" taşır (K2). Düğümlerde boştur. */}
                    {baladi.has(g.graha) && (
                      <Approx on={approx} label={t("table.approx")}>
                        {baladi.get(g.graha)}
                      </Approx>
                    )}
                  </td>
                )}
                {lordOn && (
                  <td data-nakshatra-lord={g.graha}>
                    {/* Ay'ın nakshatrası yaklaşık dereceden çıkar: lordu da "≈" taşır (K2). */}
                    <Approx on={approx} label={t("table.approx")}>
                      {lordName(g.nakshatra)}
                    </Approx>
                  </td>
                )}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
