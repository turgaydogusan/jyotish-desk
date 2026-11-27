// Efemeris paneli (yol haritası Plan 26, §4.9): seçilen ayın her günü için dokuz grahanın burcu,
// derecesi ve retro işareti; isteğe bağlı nakshatra ve pada. Tablo `ephemeris_month`'tan gelir
// ve panel açıkken istenir (K1); burada hesap yoktur (C4): "bu ay", önceki ve sonraki ay Rust'tan
// gelir. Satırın anı günün başıdır (seçilen yerin saatiyle 00:00); gün, transit tarihini o ana
// götürür. O gün içindeki burç girişi ve yön dönüşü işaretlidir; işaret renkle değil, simge ve
// sözle verilir (C9). Ay saklanmaz; "Nakshatra" kutusu panelin ayarıdır ve düzenle birlikte
// saklanır. Retro düz `R`'dir ve düğümlerde yazılmaz. CSV ve baskı yoktur (K10).
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, EphemerisCell, EphemerisMonthView, MonthId, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { GRAHA_IDS, RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
import type { Panel } from "../layout/model";
import { useEphemerisMonth } from "../modules/useEphemeris";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import "./ephemeris.css";
import { formatDegree } from "./format";
import { nakshatraName } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { parseYear, YEAR_MAX, YEAR_MIN } from "./SankrantiPanel";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Burç girişi ve yön dönüşü işaretleri; anlamları tablonun altında yazılıdır. */
export const INGRESS_MARK = "→";
export const STATION_MARK = "S";
const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;

/** Ayın adı ve yılı arayüz dilinde ("Kasım 2026"); biçimlemedir, hesap değil. */
export function formatMonth(month: MonthId, language: string): string {
  const d = new Date(Date.UTC(2000, month.month - 1, 1));
  // `Date.UTC` 0–99 arası yılı 1900'lere taşır: yıl ayrıca yazılır.
  d.setUTCFullYear(month.year);
  return new Intl.DateTimeFormat(language, { month: "long", year: "numeric", timeZone: "UTC" }).format(d);
}

/** Ayın adı ("Kasım"). */
function monthName(month: number, language: string): string {
  return new Intl.DateTimeFormat(language, { month: "long", timeZone: "UTC" }).format(new Date(Date.UTC(2000, month - 1, 1)));
}

/** `YYYY-MM-DD` → gün ve hafta günü ("01 Paz"); biçimlemedir. Okunamayan tarih olduğu gibi yazılır. */
export function formatDay(date: string, language: string): string {
  const d = new Date(`${date}T00:00:00Z`);
  if (Number.isNaN(d.getTime())) return date;
  return new Intl.DateTimeFormat(language, { day: "2-digit", weekday: "short", timeZone: "UTC" }).format(d);
}

export function EphemerisPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Body panel={panel} opened={opened} />;
}

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

function Body({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  const nakshatra = panel.settings.nakshatra === true;
  // `null`: bu ay (Rust okur). Ay oturumluktur; düzenle birlikte saklanmaz.
  const [month, setMonth] = useState<MonthId | null>(null);
  const table = useEphemerisMonth(opened, place.place, month, panel.id);
  const view = table.shown;
  const phase = usePanelLoadPhase(table.loading);
  const [year, setYear] = useState("");
  const [monthNo, setMonthNo] = useState(1);
  const [editing, setEditing] = useState(false);
  // Gösterilen ay değişince kutular onu gösterir; kullanıcı seçerken değişmez (`TransitDate`
  // ile aynı kalıp: eşitleme çizim sırasında).
  const target = editing || view === null ? null : `${view.month.year}-${view.month.month}`;
  const [synced, setSynced] = useState(target);
  if (target !== synced) {
    setSynced(target);
    if (view !== null && target !== null) {
      setYear(String(view.month.year));
      setMonthNo(view.month.month);
    }
  }
  const typed = parseYear(year);
  const go = (next: MonthId | null) => {
    setEditing(false);
    setMonth(next);
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (typed === null) return;
    go({ year: typed, month: monthNo });
  };
  const shownError = table.error === null ? null : fieldError(table.error, (f) => (i18n.exists(`ephemeris.field.${f}`) ? t(`ephemeris.field.${f}`) : f));
  const prev = view?.prev ?? null;
  const next = view?.next ?? null;

  return (
    <div className="module-panel">
      <form className="muhurta-form" aria-label={t("ephemeris.form")} onSubmit={submit}>
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odak düğmede durur (ay değişirken de). */}
          <button type="button" aria-disabled={prev === null ? true : undefined} onClick={() => prev !== null && go(prev)}>
            {t("ephemeris.prev")}
          </button>
          <button type="button" aria-disabled={next === null ? true : undefined} onClick={() => next !== null && go(next)}>
            {t("ephemeris.next")}
          </button>
          <button type="button" aria-pressed={month === null} onClick={() => go(null)}>
            {t("ephemeris.thisMonth")}
          </button>
          <label htmlFor={`${id}-month`}>{t("ephemeris.month")}</label>
          <select
            id={`${id}-month`}
            value={monthNo}
            onChange={(e) => {
              setEditing(true);
              setMonthNo(Number(e.target.value));
            }}
          >
            {MONTHS.map((m) => (
              <option key={m} value={m}>
                {monthName(m, i18n.language)}
              </option>
            ))}
          </select>
          <label htmlFor={`${id}-year`}>{t("ephemeris.year")}</label>
          <input
            id={`${id}-year`}
            type="number"
            inputMode="numeric"
            min={YEAR_MIN}
            max={YEAR_MAX}
            step={1}
            value={year}
            aria-invalid={editing && typed === null ? true : undefined}
            onChange={(e) => {
              setEditing(true);
              setYear(e.target.value);
            }}
          />
          <button type="submit">{t("ephemeris.show")}</button>
          <label>
            <input type="checkbox" checked={nakshatra} onChange={(e) => setPanelSettings(panel.id, { nakshatra: e.target.checked })} />
            {t("ephemeris.nakshatra")}
          </label>
        </div>
      </form>
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={table.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <Table view={view} nakshatra={nakshatra} />
      )}
    </div>
  );
}

function Cell({ cell, nakshatra }: { cell: EphemerisCell; nakshatra: boolean }) {
  const { t } = useTranslation();
  const retro = showsRetro(cell.graha as GrahaId, cell.retrograde);
  return (
    <td data-graha={cell.graha}>
      {t(`sign.${signIndex(cell.sign)}`)} {cell.deg_in_sign === null ? "—" : formatDegree(cell.deg_in_sign)}
      {retro && (
        <>
          {" "}
          <span aria-hidden="true">{RETRO_MARK}</span>
          <span className="visually-hidden">{t("chart.retro")}</span>
        </>
      )}
      {cell.ingress !== null && (
        <span className="ephemeris-mark" data-mark="ingress">
          {" "}
          <span aria-hidden="true">
            {INGRESS_MARK} {t(`sign.${signIndex(cell.ingress)}`)}
          </span>
          <span className="visually-hidden">{t("ephemeris.ingress", { sign: t(`sign.${signIndex(cell.ingress)}`) })}</span>
        </span>
      )}
      {cell.station !== null && (
        <span className="ephemeris-mark" data-mark="station">
          {" "}
          <span aria-hidden="true">
            {STATION_MARK}
            {cell.station === "Retrograde" ? "R" : "D"}
          </span>
          <span className="visually-hidden">{t(`ephemeris.station.${cell.station}`)}</span>
        </span>
      )}
      {nakshatra && (
        <span className="ephemeris-nakshatra">
          {nakshatraName(cell.nakshatra)} {cell.pada}
        </span>
      )}
    </td>
  );
}

function Table({ view, nakshatra }: { view: EphemerisMonthView; nakshatra: boolean }) {
  const { t, i18n } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  const title = formatMonth(view.month, i18n.language);
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table ephemeris-table">
          <caption id={captionId}>
            <span className="ephemeris-month">{title}</span>: {t("ephemeris.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("ephemeris.day")}</th>
              {GRAHA_IDS.map((g) => (
                <th key={g} scope="col">
                  <abbr title={t(`grahaName.${g}`)} aria-hidden="true">
                    {t(`graha.${g}`)}
                  </abbr>
                  <span className="visually-hidden">{t(`grahaName.${g}`)}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {view.days.map((day) => {
              const jd = day.row.jd_ut;
              const label = formatDay(day.date, i18n.language);
              return (
                <tr key={day.date} data-date={day.date}>
                  <th scope="row">
                    {jd === null ? (
                      label
                    ) : (
                      <button type="button" className="link-button" aria-label={t("ephemeris.goTo", { date: day.date })} onClick={() => transit.setAt({ Jd: jd })}>
                        {label}
                      </button>
                    )}
                  </th>
                  {day.row.cells.map((cell) => (
                    <Cell key={cell.graha} cell={cell} nakshatra={nakshatra} />
                  ))}
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("ephemeris.legend", { ingress: INGRESS_MARK, station: STATION_MARK })}</p>
      <p className="panel-message">{t("ephemeris.method", { method: view.method })}</p>
    </>
  );
}
