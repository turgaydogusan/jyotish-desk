import { useTranslation } from "react-i18next";
import { chartDataFromPlanetRows } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { isChartStyle } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { chartLabels } from "./labels";
import "./panels.css";
import { useSky } from "./sky";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export function chartVarga(settings: Record<string, unknown>): string {
  return typeof settings.varga === "string" ? settings.varga : "D1";
}

/**
 * Harita paneli. Lagna ve varga komutları M2–M3'te gelir; şimdilik yalnızca D1 çizilir ve
 * şimdiki gökyüzü gösterilir (1. ev Koç). Diğer vargalar açık bir bilgi metni gösterir.
 */
export function ChartPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const sky = useSky();
  const highlight = useHighlight();
  const phase = usePanelLoadPhase(sky.state.status === "loading");
  const varga = chartVarga(panel.settings);
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";

  if (varga !== "D1") return <p className="panel-message">{t("chart.vargaUnavailable", { varga })}</p>;
  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;

  const data = chartDataFromPlanetRows(sky.state.rows);
  return (
    <div className="chart-panel">
      {data.ascSign === null && (
        <p className="panel-notice" role="note">
          {t("chart.noLagna")}
        </p>
      )}
      <div className="chart-panel-figure">
        <ChartSvg
          data={data}
          style={style}
          labels={chartLabels(t, varga)}
          highlight={highlight.active}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
    </div>
  );
}
