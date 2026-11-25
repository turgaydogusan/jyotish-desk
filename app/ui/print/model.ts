// Baskı belgesi (Plan 12 D4): açık haritadan sayfa sayfa düz veri. Bütün metinler burada
// çözülür; çizen katman (PrintDocumentView) yalnızca dizer. Böylece düzen başka bir çıktı
// yoluna taşınabilir (v1 spec §12: baskı düzenleri soyut tutulur). Hesap yoktur (C4): burç,
// derece, retro, dasa dönemleri ve uyarılar Rust'tan gelir.
import type { TFunction } from "i18next";
import type { OpenedClient, Varga } from "../bindings";
import { chartDataFromChart, VARGA_NAMES } from "../chart/adapter";
import type { ChartData, ChartLabels, ChartStyle } from "../chart/types";
import { childrenOf, formatJdDate } from "../dasha/tree";
import { birthInfoRows } from "../panels/BirthInfoPanel";
import { formatDegree } from "../panels/format";
import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS } from "../panels/grahaCells";
import { GRAHA_COLUMNS, type GrahaColumn } from "../panels/grahaColumns";
import { chartLabels } from "../panels/labels";
import { timeUnknown } from "../session/birth";
import { ayanamsaKey } from "../settings/model";
import type { PrintOptions, PrintPageId } from "./options";

export type PrintRow = readonly [label: string, value: string];

interface PageBase {
  id: PrintPageId;
  heading: string;
}

export interface CoverPage extends PageBase {
  id: "cover";
  birth: PrintRow[];
  settingsHeading: string;
  settings: PrintRow[];
  warningsHeading: string;
  warnings: { name: string; hint: string }[];
  /** Uygulama ve hesap motoru sürümleri. */
  credit: string;
}

export interface PrintChart {
  varga: Varga;
  caption: string;
  /** Varga haritada yoksa `null`; `missing` metni basılır. */
  data: ChartData | null;
  labels: ChartLabels;
  missing: string;
}

export interface ChartsPage extends PageBase {
  id: "charts";
  style: ChartStyle;
  charts: PrintChart[];
  /** Doğum saati bilinmiyorsa lagna ve evlerin neden çizilmediği. */
  note: string | null;
  /** Harita içindeki işaretlerin açıklaması (R, c, altı çizili). */
  legend: string;
}

export interface DashaBlock {
  lord: string;
  start: string;
  end: string;
  rows: { lord: string; start: string; end: string }[];
}

export interface DashaPage extends PageBase {
  id: "dasha";
  note: string | null;
  columns: { lord: string; start: string; end: string };
  blocks: DashaBlock[];
}

export interface GrahasPage extends PageBase {
  id: "grahas";
  /** Doğum saati bilinmiyorsa "≈" işaretinin ve eksik lagna satırı ile ev sütununun açıklaması. */
  note: string | null;
  rowHeader: string;
  columns: { id: GrahaColumn; label: string; numeric: boolean }[];
  rows: { key: string; name: string; cells: string[] }[];
}

export type PrintPage = CoverPage | ChartsPage | DashaPage | GrahasPage;

export interface PrintDocument {
  client: string;
  chart: string;
  /** Yazdırma işinin ve "PDF olarak kaydet"in önerdiği dosyanın adı. */
  title: string;
  pages: PrintPage[];
}

/** Kâğıtta basılan vargalar (UI spec §7: D1+D9). */
export const PRINT_VARGAS = ["D1", "D9"] as const satisfies readonly Varga[];

function coverPage(t: TFunction, opened: OpenedClient): CoverPage {
  const s = opened.chart.meta.settings;
  const ayanamsa = t(`settings.ayanamsa.${ayanamsaKey(s.ayanamsa)}`);
  const deg = opened.chart.ayanamsa_deg;
  const settings: PrintRow[] = [
    [t("settings.field.zodiac"), t(`settings.zodiac.${s.zodiac}`)],
    [t("settings.field.ayanamsa"), deg === null ? ayanamsa : `${ayanamsa} ${formatDegree(deg)}`],
    [t("settings.field.node"), t(`settings.node.${s.node}`)],
    [t("settings.field.house_system"), t(`settings.house.${s.house_system}`)],
    [t("settings.field.dasha_year"), s.dasha_year === null ? "—" : t("settings.dashaYearOption", { days: s.dasha_year })],
  ];
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  return {
    id: "cover",
    heading: opened.client.name,
    birth: birthInfoRows(t, opened),
    settingsHeading: t("settings.title"),
    settings,
    warningsHeading: t("birthInfo.warnings"),
    warnings: warnings.map((w) => ({ name: t(`badge.${w}`), hint: t(`badgeHint.${w}`) })),
    credit: t("print.credit", { app: t("app.title"), engine: opened.chart.meta.engine_version, swe: opened.chart.meta.swe_version }),
  };
}

function chartsPage(t: TFunction, opened: OpenedClient, style: ChartStyle): ChartsPage {
  const unknown = timeUnknown(opened);
  return {
    id: "charts",
    heading: t("print.page.charts"),
    style,
    charts: PRINT_VARGAS.map((varga) => ({
      varga,
      caption: `${varga} · ${VARGA_NAMES[varga]}`,
      data: chartDataFromChart(opened.chart, varga, { showLagna: !unknown }),
      labels: chartLabels(t, varga),
      missing: t("chart.vargaMissing", { varga }),
    })),
    note: unknown ? `${t("chart.timeUnknown")} ${t("chart.timeUnknownVarga")}` : null,
    legend: t("print.legend"),
  };
}

function dashaPage(t: TFunction, opened: OpenedClient): DashaPage {
  const { periods } = opened.chart.dasha;
  const offset = opened.chart_record.resolved.offset_secs;
  const date = (jd: number | null) => formatJdDate(jd, offset);
  return {
    id: "dasha",
    heading: t("print.page.dasha"),
    note: timeUnknown(opened) ? t("dasha.approx") : null,
    columns: { lord: t("print.dasha.lord"), start: t("print.dasha.start"), end: t("print.dasha.end") },
    blocks: childrenOf(periods, []).map((maha) => ({
      lord: t(`grahaName.${maha.lords[0]}`),
      start: date(maha.start_jd_ut),
      end: date(maha.end_jd_ut),
      rows: childrenOf(periods, maha.lords).map((antar) => ({
        lord: t(`grahaName.${antar.lords[1]}`),
        start: date(antar.start_jd_ut),
        end: date(antar.end_jd_ut),
      })),
    })),
  };
}

function grahasPage(t: TFunction, opened: OpenedClient): GrahasPage {
  const unknown = timeUnknown(opened);
  const columns = GRAHA_COLUMNS.filter((c) => !(unknown && c === "house"));
  const lagna = { key: "Lagna", name: t("table.lagnaRow"), cells: columns.map((c) => lagnaCellText(t, opened.chart.lagna, c)) };
  const grahas = opened.chart.grahas.map((g) => ({
    key: g.graha,
    name: t(`grahaName.${g.graha}`),
    cells: columns.map((c) => {
      const text = grahaCellText(t, g, c);
      return unknown && g.graha === "Moon" && APPROX_COLUMNS.has(c) ? `≈ ${text}` : text;
    }),
  }));
  return {
    id: "grahas",
    heading: t("print.page.grahas"),
    note: unknown ? t("print.grahasNote") : null,
    rowHeader: t("table.graha"),
    columns: columns.map((c) => ({ id: c, label: t(`table.${c}`), numeric: NUMERIC_COLUMNS.has(c) })),
    rows: unknown ? grahas : [lagna, ...grahas],
  };
}

/** Seçilen sayfalar, `options.pages` sırasında (her zaman `PRINT_PAGES` sırası). */
export function buildPrintDocument(t: TFunction, opened: OpenedClient, options: PrintOptions): PrintDocument {
  const chart = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const build: Record<PrintPageId, () => PrintPage> = {
    cover: () => coverPage(t, opened),
    charts: () => chartsPage(t, opened, options.style),
    dasha: () => dashaPage(t, opened),
    grahas: () => grahasPage(t, opened),
  };
  return {
    client: opened.client.name,
    chart,
    title: `${opened.client.name} – ${chart}`,
    pages: options.pages.map((id) => build[id]()),
  };
}
