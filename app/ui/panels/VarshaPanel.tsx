// Varshaphala paneli (yol haritası §4.14): seçilen yılın dönüş anı, yıllık haritası ve munthası.
// Değerler `compute_varsha`'dan gelir; burada hesap yoktur (C4). Yıl seçimi ortak depodadır
// (`varsha/yearStore`): dasa ağacının mudda görünümü aynı yılı gösterir. Yer seçimi panelin
// ayarıdır ve düzenle saklanır. Doğum saati bilinmiyorsa yıllık harita ve muntha Rust'tan boş
// gelir (K2): çizilmez, yerine bir açıklama çıkar. Yıllık harita var olan olay haritası yoluyla
// kaydedilebilir (`save_varsha_chart`); kayıt açılmaz, yalnızca bildirilir.
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { Chart, CommandError, OpenedClient, Sign, VarshaView } from "../bindings";
import { chartDataFromChart, signIndex } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { CHART_STYLES, isChartStyle, RETRO_MARK, showsRetro, type ChartData } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { errorMessage } from "../i18n/errors";
import type { Panel } from "../layout/model";
import { inputKey } from "../modules/input";
import { placeLabel } from "../modules/muhurta";
import { useLastData } from "../modules/useModule";
import { useVarsha, VARSHA_PLACES, varshaPlace, varshaPlaceChoice, type VarshaPlaceChoice } from "../modules/useVarsha";
import { useSession } from "../session/store";
import { usePrefs } from "../settings/prefs";
import { VarshaYearControls } from "../varsha/YearControls";
import { useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import { formatDegree, formatJdMinute } from "./format";
import { HomePlaceDialog } from "./HomePlaceDialog";
import { chartLabels, nakshatraName, VARA_NAMES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";
import "./varsha.css";

/** Munthanın çizimdeki işareti (transliterasyon; dile göre değişmez, K11). */
export const MUNTHA_MARK = "Mu";

export function VarshaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Varsha panel={panel} opened={opened} />;
}

function Varsha({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { prefs } = usePrefs();
  const { setPanelSettings } = useLayoutStore();
  const { age: selected, setAge } = useVarshaAge(opened.chart_record.id);
  const id = useId();
  const [editingHome, setEditingHome] = useState(false);
  const choice = varshaPlaceChoice(panel.settings);
  const place = varshaPlace(choice, opened, prefs);
  const module = useVarsha(opened, selected, place);
  // Yıl değişirken önceki yılın görünümü yerinde durur; başka harita ya da yer eskiyi unutturur.
  const view = useLastData(module, place === null ? null : `${inputKey(opened)}|${choice}`);
  const phase = usePanelLoadPhase(module.loading);
  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <VarshaYearControls age={selected ?? view?.report.age ?? null} current={selected === null} onChange={setAge} />
        <label htmlFor={`${id}-place`}>{t("varsha.place.label")}</label>
        <select id={`${id}-place`} value={choice} onChange={(e) => setPanelSettings(panel.id, { place: varshaPlaceChoice({ place: e.target.value }) })}>
          {VARSHA_PLACES.map((c) => (
            <option key={c} value={c}>
              {t(`varsha.place.${c}`)}
            </option>
          ))}
        </select>
        {choice === "home" && (
          <button type="button" onClick={() => setEditingHome(true)}>
            {t(place ? "muhurta.place.changeHome" : "muhurta.place.pickHome")}
          </button>
        )}
      </div>
      {place === null && (
        <p className="panel-notice" role="note">
          {t("varsha.place.missingHome")}
        </p>
      )}
      {module.error && <PanelError error={module.error} onRetry={module.retry} />}
      {place !== null && module.error === null && view === null && phase === "skeleton" && <PanelSkeleton />}
      {place !== null && module.error === null && view !== null && <Report key={`${view.report.age}|${choice}`} panel={panel} opened={opened} view={view} choice={choice} placeName={placeLabel(place)} />}
      {editingHome && <HomePlaceDialog opened={opened} onClose={() => setEditingHome(false)} />}
    </div>
  );
}

function Report({ panel, opened, view, choice, placeName }: { panel: Panel; opened: OpenedClient; view: VarshaView; choice: VarshaPlaceChoice; placeName: string }) {
  const { t } = useTranslation();
  const { report, offset_secs: offset } = view;
  const captionId = useId();
  const number = report.age + 1;
  const zone = zoneLabel(offset);
  const moment = (jd: number | null) => t("varsha.moment", { moment: formatJdMinute(jd, offset), zone });
  const sign = (s: Sign) => t(`sign.${signIndex(s)}`);
  const { facts, chart } = report;
  const rows: { key: string; label: string; value: string }[] = [
    { key: "year", label: t("varsha.year"), value: t("varsha.yearValue", { number, age: report.age }) },
    { key: "start", label: t("varsha.start"), value: moment(report.start_jd_ut) },
    { key: "end", label: t("varsha.end"), value: moment(report.end_jd_ut) },
  ];
  if (chart !== null) {
    rows.push({ key: "place", label: t("varsha.placeRow"), value: placeName });
    rows.push({ key: "vara", label: t("varsha.vara"), value: VARA_NAMES[chart.panchang.vara] });
  }
  if (facts !== null && chart !== null) {
    rows.push({
      key: "lagna",
      label: t("varsha.lagna"),
      value: t("varsha.lagnaValue", { sign: sign(facts.lagna_sign), degree: chart.lagna.deg_in_sign === null ? "—" : formatDegree(chart.lagna.deg_in_sign), lord: t(`grahaName.${facts.lagna_lord}`) }),
    });
    rows.push({
      key: "muntha",
      label: t("varsha.muntha"),
      value: t("varsha.munthaValue", { sign: sign(facts.muntha.sign), annual: facts.muntha.annual_house, natal: facts.muntha.natal_house, lord: t(`grahaName.${facts.muntha.lord}`) }),
    });
    rows.push({ key: "natal-lagna", label: t("varsha.natalLagna"), value: t("varsha.natalLagnaValue", { sign: sign(facts.natal_lagna_sign), lord: t(`grahaName.${facts.natal_lagna_lord}`) }) });
  }
  rows.push({ key: "mudda", label: t("varsha.muddaFirst"), value: t(`grahaName.${report.mudda.first_lord}`) });
  return (
    <>
      {!report.time_known && (
        <p className="panel-notice" role="note">
          {t("varsha.timeUnknown")}
        </p>
      )}
      {report.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          {t("badgeHint.LowPrecision")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId} className="visually-hidden">
            {t("varsha.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("varsha.fact")}</th>
              <th scope="col">{t("varsha.value")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.key} data-fact={r.key}>
                <th scope="row">{r.label}</th>
                <td>{r.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {chart !== null && facts !== null && (
        <>
          <AnnualChart panel={panel} chart={chart} number={number} munthaSign={signIndex(facts.muntha.sign)} />
          <Positions chart={chart} />
          <SaveAsEvent opened={opened} age={report.age} home={choice === "home"} />
        </>
      )}
      <p className="panel-message">{t("varsha.reading")}</p>
      <p className="panel-message">{t("varsha.method", { method: report.method })}</p>
    </>
  );
}

/** Yıllık haritanın D1 çizimi; muntha burcu "Mu" ile işaretlenir. Biçim panelin ayarıdır. */
function AnnualChart({ panel, chart, number, munthaSign }: { panel: Panel; chart: Chart; number: number; munthaSign: number }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
  const base = chartDataFromChart(chart, "D1", { showLagna: true });
  if (base === null) return null;
  const data: ChartData = { ...base, marks: [{ sign: munthaSign, text: MUNTHA_MARK }] };
  return (
    <>
      <div className="module-toolbar">
        <label htmlFor={`${id}-style`}>{t("chart.style")}</label>
        <select id={`${id}-style`} value={style} onChange={(e) => isChartStyle(e.target.value) && setPanelSettings(panel.id, { style: e.target.value })}>
          {CHART_STYLES.map((s) => (
            <option key={s} value={s}>
              {t(`chartStyle.${s}`)}
            </option>
          ))}
        </select>
      </div>
      <div className="varsha-figure">
        <ChartSvg
          data={data}
          style={style}
          labels={{ ...chartLabels(t, "D1"), title: t("varsha.chartTitle", { number }) }}
          highlight={highlight.active}
          pressed={highlight.pinned}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
      {/* Çizimdeki işaret `aria-hidden`'dır: muntha burada sözcükle okunur. */}
      <p className="visually-hidden">{t("varsha.munthaMark", { sign: t(`sign.${munthaSign}`), mark: MUNTHA_MARK })}</p>
    </>
  );
}

/** Çizimin tablo karşılığı: lagna ve dokuz grahanın yıllık haritadaki yeri. */
function Positions({ chart }: { chart: Chart }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const captionId = useId();
  const degree = (d: number | null) => (d === null ? "—" : formatDegree(d));
  return (
    <BalaScroll labelledBy={captionId}>
      <table className="module-table">
        <caption id={captionId} className="visually-hidden">
          {t("varsha.positions")}
        </caption>
        <thead>
          <tr>
            <th scope="col">{t("varsha.col.graha")}</th>
            <th scope="col">{t("varsha.col.sign")}</th>
            <th scope="col" className="num">
              {t("varsha.col.degree")}
            </th>
            <th scope="col">{t("varsha.col.nakshatra")}</th>
            <th scope="col" className="num">
              {t("varsha.col.house")}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr data-graha="Lagna">
            <th scope="row">{t("table.lagnaRow")}</th>
            <td>{t(`sign.${signIndex(chart.lagna.sign)}`)}</td>
            <td className="num">{degree(chart.lagna.deg_in_sign)}</td>
            <td>{nakshatraName(chart.lagna.nakshatra)}</td>
            <td className="num">1</td>
          </tr>
          {chart.grahas.map((g) => (
            <tr key={g.graha} data-graha={g.graha} className={active?.type === "graha" && active.id === g.graha ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                {showsRetro(g.graha, g.retrograde) && (
                  <>
                    {" "}
                    <span aria-hidden="true">{RETRO_MARK}</span>
                    <span className="visually-hidden">{t("chart.retro")}</span>
                  </>
                )}
              </th>
              <td>{t(`sign.${signIndex(g.sign)}`)}</td>
              <td className="num">{degree(g.deg_in_sign)}</td>
              <td>{nakshatraName(g.nakshatra)}</td>
              <td className="num">{g.house}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </BalaScroll>
  );
}

/**
 * "Olay haritası olarak kaydet": kaynak harita, yaş ve yer seçimi gider; an ve yer Rust'ta
 * çözülür. Kayıt açılmaz: etiketi bildirilir ve harita listeleri yenilenir. Bileşen yıl ve yer
 * başına yeniden kurulur (`key`): başka yılın bildirimi bu yıla taşınmaz.
 */
function SaveAsEvent({ opened, age, home }: { opened: OpenedClient; age: number; home: boolean }) {
  const { t, i18n } = useTranslation();
  const { touch } = useSession();
  const [state, setState] = useState<{ status: "idle" | "saving" } | { status: "saved"; label: string } | { status: "error"; error: CommandError }>({ status: "idle" });
  const busy = state.status === "saving" || state.status === "saved";
  const save = async () => {
    if (busy) return;
    setState({ status: "saving" });
    const r = await api.saveVarshaChart(opened.chart_record.id, age, home);
    if (r.status === "error") return setState({ status: "error", error: r.error });
    setState({ status: "saved", label: r.data.label });
    touch();
  };
  return (
    <div className="varsha-save">
      {/* Kayıttan sonra düğme odağı tutar ama yeniden kaydetmez: aynı yıl iki kez eklenmez. */}
      <button type="button" aria-disabled={busy || undefined} onClick={() => void save()}>
        {t("varsha.save")}
      </button>
      <p role="status">{state.status === "saved" ? t("varsha.saved", { label: state.label }) : ""}</p>
      {state.status === "error" && (
        <p role="alert" className="panel-notice">
          {errorMessage(t, i18n, state.error)}
        </p>
      )}
    </div>
  );
}