// Panchang paneli (ileri modüller spec'i §4): doğum anının beş uzvu, bitiş saatleri, gün doğumu
// ve batımı. Değerler `Chart.panchang`'tan hazır gelir (v1'den beri hesaplanır). Altındaki
// panchang ekleri (yogi, avayogi, dagdha rasi; Plan 23) ayrı bir komuttan gelir: `PanchangExtras`.
// Saatler doğumun çözülmüş ofsetinde gösterilir. Ortak vurgu: Ay vurgulanınca Ay'a bağlı
// satırlar, Güneş vurgulanınca Güneş'e bağlı satırlar `--select` zemin alır.
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import type { Panchang } from "../bindings";
import { formatFixedOffset } from "../birth/model";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { formatJdMinute } from "./format";
import { nakshatraName, nityaYogaName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PanchangExtrasSection } from "./PanchangExtras";
import "./panels.css";
import type { PanelProps } from "./types";

export interface PanchangRow {
  key: "tithi" | "vara" | "nakshatra" | "yoga" | "karana" | "sunrise" | "sunset";
  label: string;
  value: string;
  /** Bitiş anı; öğenin bitişi yoksa `null`. */
  ends: string | null;
  /** Değerin türediği grahalar (ortak vurgu için). */
  grahas: readonly GrahaId[];
  /** Doğum saati bilinmiyorsa "≈" ile işaretlenir (saate bağlı değer). */
  timeBound: boolean;
}

const SUN_MOON: readonly GrahaId[] = ["Sun", "Moon"];

/** Panchang satırları; `offsetSecs` doğumun çözülmüş ofsetidir. */
export function panchangRows(t: TFunction, p: Panchang, offsetSecs: number): PanchangRow[] {
  const at = (jd: number | null) => formatJdMinute(jd, offsetSecs);
  return [
    {
      key: "tithi",
      label: t("panchang.tithi"),
      value: `${p.tithi.paksha} ${tithiInPaksha(p.tithi.no)} · ${tithiName(p.tithi.no)}`,
      ends: at(p.tithi.ends_jd_ut),
      grahas: SUN_MOON,
      timeBound: true,
    },
    { key: "vara", label: t("panchang.vara"), value: `${VARA_NAMES[p.vara]} · ${t(`panchang.weekday.${p.vara}`)}`, ends: null, grahas: [], timeBound: true },
    {
      key: "nakshatra",
      label: t("panchang.nakshatra"),
      value: `${nakshatraName(p.nakshatra.no)} · ${t("panchang.pada", { pada: p.nakshatra.pada })}`,
      ends: at(p.nakshatra.ends_jd_ut),
      grahas: ["Moon"],
      timeBound: true,
    },
    { key: "yoga", label: t("panchang.yoga"), value: nityaYogaName(p.yoga.no), ends: at(p.yoga.ends_jd_ut), grahas: SUN_MOON, timeBound: true },
    { key: "karana", label: t("panchang.karana"), value: p.karana.name, ends: at(p.karana.ends_jd_ut), grahas: SUN_MOON, timeBound: true },
    { key: "sunrise", label: t("panchang.sunrise"), value: at(p.sunrise_jd_ut), ends: null, grahas: ["Sun"], timeBound: false },
    { key: "sunset", label: t("panchang.sunset"), value: at(p.sunset_jd_ut), ends: null, grahas: ["Sun"], timeBound: false },
  ];
}

export function PanchangPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { active } = useHighlight();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;

  const panchang = opened.chart.panchang;
  const resolved = opened.chart_record.resolved;
  const unknown = timeUnknown(opened);
  const noSunrise = panchang.sunrise_jd_ut === null || panchang.sunset_jd_ut === null;
  const rows = panchangRows(t, panchang, resolved.offset_secs);
  const offset = `${resolved.abbrev} ${formatFixedOffset(resolved.offset_secs)}`.trim();

  return (
    <div className="module-panel">
      {unknown && <p className="panel-notice">{t("panchang.timeUnknown")}</p>}
      <table className="module-table">
        <caption className="visually-hidden">{t("panchang.caption", { offset })}</caption>
        <thead>
          <tr>
            <th scope="col">{t("panchang.item")}</th>
            <th scope="col">{t("panchang.value")}</th>
            <th scope="col" className="num">
              {t("panchang.ends")}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => {
            const on = active?.type === "graha" && r.grahas.includes(active.id);
            return (
              <tr key={r.key} data-row={r.key} className={on ? "is-highlighted" : undefined}>
                <th scope="row">{r.label}</th>
                <td>
                  <Approx on={unknown && r.timeBound} label={t("table.approx")}>
                    {r.value}
                  </Approx>
                </td>
                <td className="num">{r.ends ?? ""}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {noSunrise && (
        <p className="panel-notice">
          <strong>{t("badge.SunriseUnavailable")}</strong>: {t("badgeHint.SunriseUnavailable")}
        </p>
      )}
      <PanchangExtrasSection opened={opened} />
    </div>
  );
}
