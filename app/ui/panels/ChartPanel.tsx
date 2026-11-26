import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { KutaReport, KutaRole, Varga } from "../bindings";
import { chartDataFromChart, isVarga, signIndex, transitPlanets, VARGA_NAMES, VARGAS } from "../chart/adapter";
import { ChartSvg, type ChartSvgProps } from "../chart/ChartSvg";
import { CHART_STYLES, isChartStyle, isGrahaId, showsRetro, type ChartData, type ChartMark, type ChartPlanet } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useCompat } from "../modules/useCompat";
import { useJaimini } from "../modules/useJaimini";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransit } from "../transit/store";
import { useLayoutStore, useViewStore } from "../view/store";
import { arudhaLabel, chartLabels, signAbbreviations } from "./labels";
import "./panels.css";
import { PanelError } from "./states";
import type { PanelProps } from "./types";

export function chartVarga(settings: Record<string, unknown>): Varga {
  return isVarga(settings.varga) ? settings.varga : "D1";
}

/**
 * Uyum raporundan eşin dokuz grahası, çizim verisi olarak: açık harita damat rolündeyse eşin
 * satırları raporun ikinci yarısıdır. Burç, derece ve retro Rust'tan gelir; dönüşüm yalnızca
 * biçimdir (C4).
 */
export function partnerPlanets(report: KutaReport, openRole: KutaRole): ChartPlanet[] {
  return report.overlay.flatMap((row) =>
    row.owner !== openRole && isGrahaId(row.graha) && row.deg_in_sign !== null ? [{ id: row.graha, sign: signIndex(row.sign), degInSign: row.deg_in_sign, retrograde: row.retrograde }] : [],
  );
}

/**
 * Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde.
 * Varga ve biçim seçimi panel ayarına yazılır; düzenle birlikte kaydedilir.
 * `settings.arudha_marks` açıkken (yalnızca D1, doğum saati biliniyorsa) arudha padalar
 * burçlarına "AL", "A2" … "UL" olarak yazılır; padalar `compute_jaimini`'den gelir.
 * `settings.partner_ring` açıkken (yalnızca D1) eşin grahaları transit halkasının çizimiyle dış
 * halkaya yazılır (ileri modüller spec'i §9); iki halka aynı anda açık olamaz: biri açılınca
 * diğeri kapanır, kayıtta ikisi de açıksa transit halkası çizilir. Eş haritanındır, düzen
 * kullanıcınındır: kutu yalnızca açık haritanın bir eşi varken gösterilir; eşi olmayan haritada
 * ayar açık olsa da hiçbir şey istenmez ve çizilmez.
 */
export function ChartPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const varga = chartVarga(panel.settings);
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
  const ringOn = panel.settings.transit_ring === true;
  const ringOff = varga !== "D1";
  const ringDrawn = ringOn && !ringOff;
  const partnerOn = panel.settings.partner_ring === true && !ringOn;
  // Eş bağı açık haritanın görünüm kaydındadır; rapor yalnızca halka çizilecekse istenir.
  const hasPartner = useViewStore().model.compat.partnerUuid !== null;
  const partnerOff = varga !== "D1";
  const compat = useCompat(opened, hasPartner && partnerOn && !partnerOff);
  const marksOn = panel.settings.arudha_marks === true;
  const unknown = opened !== null && timeUnknown(opened);
  // Padalar lagnaya bağlıdır: saat bilinmiyorsa istenmez (Rust zaten boş döner, K2).
  const marksOff = varga !== "D1" ? "arudhaMarksOnlyD1" : unknown ? "arudhaMarksTimeUnknown" : null;
  const jaimini = useJaimini(opened, marksOn && marksOff === null);

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const base = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
  const marks: ChartMark[] = (jaimini.data?.arudhas ?? []).map((p) => ({ sign: signIndex(p.sign), text: arudhaLabel(p.house) }));
  const data = base && marks.length > 0 ? { ...base, marks } : base;
  const signs = signAbbreviations(t);
  const markList = marks.map((m) => `${m.text} ${signs[m.sign]}`).join(", ");
  const partner = compat.report !== null ? partnerPlanets(compat.report, compat.openRole) : [];
  // Çizimdeki "R" işareti `aria-hidden` etiketin içindedir: retro burada sözcükle okunur (düğümlerde yok).
  const partnerList = partner
    .map((p) => `${t(`grahaName.${p.id}`)} ${t(`sign.${p.sign}`)} ${Math.floor(p.degInSign)}°${showsRetro(p.id, p.retrograde) ? ` ${t("chart.retro")}` : ""}`)
    .join(", ");
  const svgProps = {
    style,
    labels: chartLabels(t, varga),
    highlight: highlight.active,
    pressed: highlight.pinned,
    onHover: highlight.hover,
    onPin: highlight.togglePin,
  };
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
        {/* Kapalı kutu yerel `disabled` değildir: odaklanır, nedeni görünür ve `aria-describedby`
            ile bağlıdır; tıklamak ayarı değiştirmez (Plan 11 D22'nin kalıbı). */}
        <label className={ringOff ? "chart-ring-toggle is-off" : "chart-ring-toggle"}>
          <input
            type="checkbox"
            checked={ringOn}
            aria-disabled={ringOff || undefined}
            aria-describedby={ringOff ? `${id}-ring` : undefined}
            onChange={(e) => !ringOff && setPanelSettings(panel.id, e.target.checked && panel.settings.partner_ring === true ? { transit_ring: true, partner_ring: false } : { transit_ring: e.target.checked })}
          />
          {t("chart.transitRing")}
        </label>
        {hasPartner && (
          <label className={partnerOff ? "chart-ring-toggle is-off" : "chart-ring-toggle"}>
            <input
              type="checkbox"
              checked={partnerOn}
              aria-disabled={partnerOff || undefined}
              aria-describedby={partnerOff ? `${id}-partner` : undefined}
              onChange={(e) => !partnerOff && setPanelSettings(panel.id, e.target.checked && ringOn ? { partner_ring: true, transit_ring: false } : { partner_ring: e.target.checked })}
            />
            {t("chart.partnerRing")}
          </label>
        )}
        <label className={marksOff !== null ? "chart-ring-toggle is-off" : "chart-ring-toggle"}>
          <input
            type="checkbox"
            checked={marksOn}
            aria-disabled={marksOff !== null || undefined}
            aria-describedby={marksOff !== null ? `${id}-marks` : undefined}
            onChange={(e) => marksOff === null && setPanelSettings(panel.id, { arudha_marks: e.target.checked })}
          />
          {t("chart.arudhaMarks")}
        </label>
      </div>
      {(ringOff || marksOff !== null) && (
        <p className="chart-toolbar-hint">
          {ringOff && <span id={`${id}-ring`}>{t("chart.transitRingOnlyD1")}</span>}
          {hasPartner && partnerOff && (
            <>
              {" "}
              <span id={`${id}-partner`}>{t("chart.partnerRingOnlyD1")}</span>
            </>
          )}
          {ringOff && marksOff !== null && " "}
          {marksOff !== null && <span id={`${id}-marks`}>{t(`chart.${marksOff}`)}</span>}
        </p>
      )}
      {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
      {compat.error && <PanelError error={compat.error} onRetry={compat.retry} />}
      {unknown && (
        <p className="panel-notice" role="note">
          {t("chart.timeUnknown")}
          {/* Bölüm haritalarında her yerleşim saate bağlıdır (D9'da Ay, D30–D60'ta hepsi). */}
          {varga !== "D1" && <> {t("chart.timeUnknownVarga")}</>}
        </p>
      )}
      {data ? (
        <div className="chart-panel-figure">
          {ringDrawn ? <RingChart data={data} {...svgProps} /> : <ChartSvg data={partner.length > 0 ? { ...data, transits: partner } : data} {...svgProps} />}
        </div>
      ) : (
        <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>
      )}
      {/* Çizimdeki işaretler `aria-hidden`'dır: padalar ve eşin grahaları burada sözcükle okunur. */}
      {data && marks.length > 0 && <p className="visually-hidden">{t("chart.arudhaList", { list: markList })}</p>}
      {data && partner.length > 0 && <p className="visually-hidden">{t("chart.partnerList", { name: compat.partner?.brief.client_name ?? "", list: partnerList })}</p>}
    </div>
  );
}

/** Halka açıkken: transitler okunur (tüketici kaydı) ve natal çizime eklenir. */
function RingChart({ data, ...props }: { data: ChartData } & Omit<ChartSvgProps, "data">) {
  const { t } = useTranslation();
  const { view } = useTransit();
  const transits = view ? transitPlanets(view) : [];
  // Çizimdeki "R" işareti `aria-hidden` etiketin içindedir: retro burada sözcükle okunur (düğümlerde yok).
  const list = transits
    .map((p) => `${t(`grahaName.${p.id}`)} ${t(`sign.${p.sign}`)} ${Math.floor(p.degInSign)}°${showsRetro(p.id, p.retrograde) ? ` ${t("chart.retro")}` : ""}`)
    .join(", ");
  return (
    <>
      <ChartSvg data={{ ...data, transits }} {...props} />
      {view && <p className="visually-hidden">{t("chart.transitList", { list })}</p>}
    </>
  );
}
