import { useTranslation } from "react-i18next";
import type { Varga } from "../bindings";
import { chartDataFromChart, isVarga } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { isChartStyle } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useSession } from "../session/store";
import { chartLabels } from "./labels";
import "./panels.css";
import type { PanelProps } from "./types";

export function chartVarga(settings: Record<string, unknown>): Varga {
  return isVarga(settings.varga) ? settings.varga : "D1";
}

/** Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde. */
export function ChartPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const highlight = useHighlight();
  const varga = chartVarga(panel.settings);
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const data = chartDataFromChart(opened.chart, varga, { showLagna: true });
  if (!data) return <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>;
  return (
    <div className="chart-panel">
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
    </div>
  );
}
