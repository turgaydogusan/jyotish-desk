// Ashtakavarga paneli (ileri modüller spec'i §4): yedi grahanın BAV'ı ve SAV. Değerler
// `Chart.ashtakavarga`'dan hazır gelir (v1'den beri hesaplanır); komut çağırmaz. Burç sütunları
// Koç'tan Balık'a sabittir. Görünüm (SAV / tek graha BAV) ve biçim (tablo / harita içi) panel
// ayarıdır. Satır toplamı gösterilen hücrelerin toplamıdır (Plan 14 D11); sütun toplamı SAV
// satırıdır ve Rust'tan gelir. Bindular lagnaya da bağlıdır: doğum saati bilinmiyorsa tablo
// gösterilmez. Ortak vurgu: graha satırı, burç ve ev sütunu `--select` zemin alır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Ashtakavarga } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import "./panels.css";
import type { PanelProps } from "./types";

/** BAV satır sırası (Rust `ASHTAKAVARGA_GRAHAS`). */
export const BAV_GRAHAS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] as const satisfies readonly GrahaId[];
export type BavGraha = (typeof BAV_GRAHAS)[number];
export type AshtakavargaView = "sav" | BavGraha;
export const ASHTAKAVARGA_FORMATS = ["table", "chart"] as const;
export type AshtakavargaFormat = (typeof ASHTAKAVARGA_FORMATS)[number];

export function ashtakavargaView(settings: Record<string, unknown>): AshtakavargaView {
  const v = settings.view;
  return typeof v === "string" && (BAV_GRAHAS as readonly string[]).includes(v) ? (v as BavGraha) : "sav";
}

export function ashtakavargaFormat(settings: Record<string, unknown>): AshtakavargaFormat {
  return settings.format === "chart" ? "chart" : "table";
}

const SIGNS = Array.from({ length: 12 }, (_, i) => i);
const total = (row: readonly number[]) => row.reduce((a, b) => a + b, 0);

/** Güney Hint düzeni: burçlar sabit kutularda, Balık sol üstte, saat yönünde. `[satır, sütun]`, 1 tabanlı. */
const SOUTH_CELLS: readonly (readonly [number, number])[] = [
  [1, 2],
  [1, 3],
  [1, 4],
  [2, 4],
  [3, 4],
  [4, 4],
  [4, 3],
  [4, 2],
  [4, 1],
  [3, 1],
  [2, 1],
  [1, 1],
];

export function AshtakavargaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const view = ashtakavargaView(panel.settings);
  const format = ashtakavargaFormat(panel.settings);

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (timeUnknown(opened)) return <p className="panel-notice">{t("ashtakavarga.timeUnknown")}</p>;

  const av: Ashtakavarga = opened.chart.ashtakavarga;
  const lagna = signIndex(opened.chart.lagna.sign);
  const viewLabel = (v: AshtakavargaView) => (v === "sav" ? t("ashtakavarga.sav") : t("ashtakavarga.bavOf", { graha: t(`grahaName.${v}`) }));
  const active = highlight.active;
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burcun sütunu.
  const signOn = (sign: number) => (active?.type === "sign" && active.sign === sign) || (active?.type === "house" && (lagna + active.house - 1) % 12 === sign);
  const grahaOn = (g: BavGraha) => active?.type === "graha" && active.id === g;
  const pinButton = (target: HighlightTarget, label: string) => (
    <button
      type="button"
      className="link-button"
      aria-pressed={sameTarget(highlight.pinned, target)}
      onClick={() => highlight.togglePin(target)}
      onMouseEnter={() => highlight.hover(target)}
      onMouseLeave={() => highlight.hover(null)}
      onFocus={() => highlight.hover(target)}
      onBlur={() => highlight.hover(null)}
    >
      {label}
    </button>
  );
  const signHeader = (sign: number) => (
    <>
      {pinButton({ type: "sign", sign }, t(`sign.${sign}`))}
      {sign === lagna && <span className="av-lagna"> {t("chart.lagna")}</span>}
    </>
  );
  const bavRows = view === "sav" ? BAV_GRAHAS : [view];
  const values = view === "sav" ? av.sav : av.bav[BAV_GRAHAS.indexOf(view)];

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-view`}>{t("ashtakavarga.view")}</label>
        <select id={`${id}-view`} value={view} onChange={(e) => setPanelSettings(panel.id, { view: e.target.value })}>
          {(["sav", ...BAV_GRAHAS] as const).map((v) => (
            <option key={v} value={v}>
              {viewLabel(v)}
            </option>
          ))}
        </select>
        <label htmlFor={`${id}-format`}>{t("ashtakavarga.format")}</label>
        <select id={`${id}-format`} value={format} onChange={(e) => setPanelSettings(panel.id, { format: e.target.value })}>
          {ASHTAKAVARGA_FORMATS.map((f) => (
            <option key={f} value={f}>
              {t(`ashtakavarga.formats.${f}`)}
            </option>
          ))}
        </select>
      </div>
      {format === "table" ? (
        <table className="module-table av-table">
          <caption className="visually-hidden">{t("ashtakavarga.caption", { view: viewLabel(view) })}</caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              {SIGNS.map((s) => (
                <th key={s} scope="col" data-sign={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                  {signHeader(s)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("ashtakavarga.total")}
              </th>
            </tr>
          </thead>
          <tbody>
            {bavRows.map((g) => {
              const row = av.bav[BAV_GRAHAS.indexOf(g)];
              return (
                <tr key={g} data-graha={g} className={grahaOn(g) ? "is-highlighted" : undefined}>
                  <th scope="row">{pinButton({ type: "graha", id: g }, t(`grahaName.${g}`))}</th>
                  {row.map((n, s) => (
                    <td key={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                      {n}
                    </td>
                  ))}
                  <td className="num">{total(row)}</td>
                </tr>
              );
            })}
          </tbody>
          {view === "sav" && (
            <tfoot>
              <tr data-row="sav">
                <th scope="row">{t("ashtakavarga.sav")}</th>
                {av.sav.map((n, s) => (
                  <td key={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                    {n}
                  </td>
                ))}
                <td className="num">{total(av.sav)}</td>
              </tr>
            </tfoot>
          )}
        </table>
      ) : (
        <div className="av-chart" role="group" aria-label={t("ashtakavarga.caption", { view: viewLabel(view) })}>
          <ul className="av-chart-grid">
            {SIGNS.map((s) => (
              <li key={s} data-sign={s} className={signOn(s) ? "av-cell is-highlighted" : "av-cell"} style={{ gridRow: SOUTH_CELLS[s][0], gridColumn: SOUTH_CELLS[s][1] }}>
                <span className="av-cell-sign">{signHeader(s)}</span>
                <span className="av-cell-value">{values[s]}</span>
              </li>
            ))}
            <li className="av-chart-centre" aria-hidden="true">
              {viewLabel(view)} · {total(values)}
            </li>
          </ul>
          <p className="visually-hidden">{t("ashtakavarga.totalOf", { view: viewLabel(view), total: total(values) })}</p>
        </div>
      )}
    </div>
  );
}
