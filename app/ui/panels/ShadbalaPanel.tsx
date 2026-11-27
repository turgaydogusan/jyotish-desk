// Shadbala paneli (ileri modüller spec'i §7): yedi grahanın altı gücü, toplamı, gerekli en az
// değeri, oranı ve sırası; "Kalemler" görünümünde sthana balanın ve zaman gücünün alt kalemleri;
// altta zaman lordları. Değerler `compute_shadbala`'dan gelir ve panel açıkken istenir (K1);
// burada hesap yoktur (C4). Oran 1'in altındaysa hücrede `<1` metin işareti durur; graha savaşı
// düz metinle yazılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Graha, GrahaShadbala, OpenedClient, ShadbalaReport, SthanaBala, TemporalBala, TimeLords } from "../bindings";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useShadbala } from "../modules/useShadbala";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaNumberPrashna, BalaScroll, BalaUnavailable, balaUnit, formatBala, formatFixed, formatRatio, UnitSelect, type BalaUnit } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const SHADBALA_DETAILS = ["summary", "items"] as const;
export type ShadbalaDetail = (typeof SHADBALA_DETAILS)[number];
/** Oranı 1'in altında olan grahanın metin işareti. */
export const BELOW_MARK = "<1";

/** Panelin ayrıntı ayarı; tanınmayan değer ve eksik anahtar özettir. */
export function shadbalaDetail(settings: Record<string, unknown>): ShadbalaDetail {
  return settings.detail === "items" ? "items" : "summary";
}

const STHANA_ITEMS = ["uccha", "saptavargaja", "ojayugma", "kendradi", "drekkana"] as const satisfies readonly (keyof SthanaBala)[];
const TEMPORAL_ITEMS = [
  ["day_night", "dayNight"],
  ["paksha", "paksha"],
  ["tribhaga", "tribhaga"],
  ["year_lord", "yearLord"],
  ["month_lord", "monthLord"],
  ["weekday_lord", "weekdayLord"],
  ["hora_lord", "horaLord"],
  ["ayana", "ayana"],
] as const satisfies readonly (readonly [keyof TemporalBala, string])[];
const TIME_LORDS = ["year", "month", "weekday", "hora", "tribhaga"] as const satisfies readonly (keyof TimeLords)[];

export function ShadbalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  // Sayı prashnasında meridyen yoktur: rapor istenmez.
  if (opened.prashna?.number != null) return <BalaNumberPrashna />;
  return <Shadbala panel={panel} opened={opened} />;
}

function Shadbala({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const module = useShadbala(opened);
  const phase = usePanelLoadPhase(module.loading);
  const detail = shadbalaDetail(panel.settings);
  const unit = balaUnit(panel.settings);
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "rapor yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-detail`}>{t("shadbala.detail")}</label>
        <select id={`${id}-detail`} value={detail} onChange={(e) => setPanelSettings(panel.id, { detail: e.target.value })}>
          {SHADBALA_DETAILS.map((d) => (
            <option key={d} value={d}>
              {t(`shadbala.details.${d}`)}
            </option>
          ))}
        </select>
        <UnitSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <BalaUnavailable opened={opened} />
      ) : (
        <Report report={module.data} detail={detail} unit={unit} />
      )}
    </div>
  );
}

function Report({ report, detail, unit }: { report: ShadbalaReport; detail: ShadbalaDetail; unit: BalaUnit }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  const num = (v: number | null) => <td className="num">{formatBala(v, unit)}</td>;
  const name = (row: GrahaShadbala) => (
    <th scope="row">
      <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
      {row.war && (
        <span className="bala-war">
          {" "}
          <span aria-hidden="true">{t("shadbala.war", { with: t(`graha.${row.war.with}`) })}</span>
          <span className="visually-hidden">{t("shadbala.warFull", { with: t(`grahaName.${row.war.with}`), winner: t(`grahaName.${row.war.winner}`) })}</span>
        </span>
      )}
    </th>
  );
  // Karşılaştırma gösterim eşiğidir: oran Rust'tan gelir.
  const below = (row: GrahaShadbala) => row.ratio !== null && row.ratio < 1;
  const anyBelow = report.grahas.some(below);

  return (
    <>
      <BalaScroll labelledBy={`${id}-summary`}>
        <table className="module-table">
          <caption id={`${id}-summary`} className="visually-hidden">{t("shadbala.caption")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("shadbala.graha")}</th>
              {(["sthana", "dig", "temporal", "cheshta", "naisargika", "drik"] as const).map((k) => (
                <th key={k} scope="col" className="num">
                  {t(`shadbala.${k}`)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("bala.total")}
              </th>
              <th scope="col" className="num">
                {t("shadbala.required")}
              </th>
              <th scope="col" className="num">
                {t("shadbala.ratio")}
              </th>
              <th scope="col" className="num">
                {t("bala.rank")}
              </th>
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((row) => (
              <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                {name(row)}
                {num(row.sthana.total)}
                {num(row.dig)}
                {num(row.temporal.total)}
                {num(row.cheshta)}
                {num(row.naisargika)}
                {num(row.drik)}
                {/* Rupa toplamı Rust'ın `rupas` alanıdır. */}
                <td className="num">{unit === "rupa" ? formatFixed(row.rupas, 2) : formatBala(row.total, unit)}</td>
                {num(row.required)}
                <td className="num">
                  {formatRatio(row.ratio)}
                  {below(row) && (
                    <>
                      {" "}
                      <span aria-hidden="true">{BELOW_MARK}</span>
                      <span className="visually-hidden">{t("shadbala.below")}</span>
                    </>
                  )}
                </td>
                <td className="num">{row.rank}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {detail === "items" && (
        <>
          <BalaScroll labelledBy={`${id}-sthana`}>
            <table className="module-table">
              <caption id={`${id}-sthana`}>{t("shadbala.sthanaCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("shadbala.graha")}</th>
                  {STHANA_ITEMS.map((k) => (
                    <th key={k} scope="col" className="num">
                      {t(`shadbala.${k}`)}
                    </th>
                  ))}
                  <th scope="col" className="num">
                    {t("bala.total")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {report.grahas.map((row) => (
                  <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                    <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                    {STHANA_ITEMS.map((k) => (
                      <td key={k} className="num">
                        {formatBala(row.sthana[k], unit)}
                      </td>
                    ))}
                    {num(row.sthana.total)}
                  </tr>
                ))}
              </tbody>
            </table>
          </BalaScroll>
          <BalaScroll labelledBy={`${id}-temporal`}>
            <table className="module-table">
              <caption id={`${id}-temporal`}>{t("shadbala.temporalCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("shadbala.graha")}</th>
                  {TEMPORAL_ITEMS.map(([k, label]) => (
                    <th key={k} scope="col" className="num">
                      {t(`shadbala.${label}`)}
                    </th>
                  ))}
                  <th scope="col" className="num">
                    {t("bala.total")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {report.grahas.map((row) => (
                  <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                    <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                    {TEMPORAL_ITEMS.map(([k]) => (
                      <td key={k} className="num">
                        {formatBala(row.temporal[k], unit)}
                      </td>
                    ))}
                    {num(row.temporal.total)}
                  </tr>
                ))}
              </tbody>
            </table>
          </BalaScroll>
        </>
      )}
      <ul className="bala-lords" aria-label={t("shadbala.lords")}>
        {TIME_LORDS.map((k) => (
          <li key={k} data-lord={k}>
            {t(`shadbala.lord.${k}`)}: <PinButton target={{ type: "graha", id: report.time_lords[k] }}>{t(`grahaName.${report.time_lords[k]}`)}</PinButton>
          </li>
        ))}
      </ul>
      {anyBelow && <p className="panel-message">{t("shadbala.belowLegend", { mark: BELOW_MARK })}</p>}
      <p className="panel-message">{t(`bala.legend.${unit}`)}</p>
      <p className="panel-message">{t("bala.method", { method: report.method })}</p>
    </>
  );
}
