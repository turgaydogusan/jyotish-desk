import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Varga } from "../bindings";
import { chartDataFromChart, isVarga, VARGA_NAMES, VARGAS } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { CHART_STYLES, isChartStyle } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { chartLabels } from "./labels";
import "./panels.css";
import type { PanelProps } from "./types";

export function chartVarga(settings: Record<string, unknown>): Varga {
  return isVarga(settings.varga) ? settings.varga : "D1";
}

/**
 * Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde.
 * Varga ve biçim seçimi panel ayarına yazılır; düzenle birlikte kaydedilir.
 */
export function ChartPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const varga = chartVarga(panel.settings);
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const unknown = timeUnknown(opened);
  const data = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
  return (
    <div className="chart-panel">
      <div className="chart-panel-toolbar">
        <label htmlFor={`${id}-varga`}>{t("chart.varga")}</label>
        <select
          id={`${id}-varga`}
          value={varga}
          onChange={(e) => isVarga(e.target.value) && setPanelSettings(panel.id, { varga: e.target.value })}
        >
          {VARGAS.map((v) => (
            <option key={v} value={v}>
              {v} · {VARGA_NAMES[v]}
            </option>
          ))}
        </select>
        <label htmlFor={`${id}-style`}>{t("chart.style")}</label>
        <select
          id={`${id}-style`}
          value={style}
          onChange={(e) => isChartStyle(e.target.value) && setPanelSettings(panel.id, { style: e.target.value })}
        >
          {CHART_STYLES.map((s) => (
            <option key={s} value={s}>
              {t(`chartStyle.${s}`)}
            </option>
          ))}
        </select>
      </div>
      {unknown && (
        <p className="panel-notice" role="note">
          {t("chart.timeUnknown")}
          {/* Bölüm haritalarında her yerleşim saate bağlıdır (D9'da Ay, D30–D60'ta hepsi). */}
          {varga !== "D1" && <> {t("chart.timeUnknownVarga")}</>}
        </p>
      )}
      {data ? (
        <div className="chart-panel-figure">
          <ChartSvg
            data={data}
            style={style}
            labels={chartLabels(t, varga)}
            highlight={highlight.active}
            pressed={highlight.pinned}
            onHover={highlight.hover}
            onPin={highlight.togglePin}
          />
        </div>
      ) : (
        <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>
      )}
    </div>
  );
}
