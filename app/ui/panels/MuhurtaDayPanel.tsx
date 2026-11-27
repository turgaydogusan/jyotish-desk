// Gün panchangı paneli (ileri modüller spec'i §10): seçilen anı içeren günün (gün doğumundan
// sonraki gün doğumuna) şeridi (sekiz dilim, işaretli üçü ve Abhijit), beş uzvun değişim saatli
// listesi, işaretler, hora tablosu ve açık haritanın Ay'ına göre tara ve chandra bala. Değerler
// `day_panchang`'tan gelir; burada hesap yoktur (C4). "Uygun" ya da "uygunsuz" hükmü ve
// açıklama metni yoktur; adlar etikettir. Saatler seçilen yerin saatiyle gösterilir.
import { useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { DayPanchang, DayPanchangView, Interval, OpenedClient, TransitAt } from "../bindings";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useDayPanchang } from "../modules/useDayPanchang";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { TransitDate } from "../transit/TransitDate";
import "../transit/transit.css";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import { formatJdClock, formatJdDate } from "./format";
import { NAKSHATRA_NATURE_NAMES, nakshatraName, nityaYogaName, taraName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const DAY_LINKS = ["transit", "today"] as const;
export type DayLink = (typeof DAY_LINKS)[number];

/** Panelin tarih bağı; tanınmayan değer ve eksik anahtar transit tarihidir. */
export function dayLink(settings: Record<string, unknown>): DayLink {
  return settings.link === "today" ? "today" : "transit";
}

/** Dilimin şeritteki kısa yazısı (renk tek başına anlam taşımaz). */
const MARKS = ["rahu", "yamaganda", "gulika"] as const;

export function MuhurtaDayPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Day panel={panel} opened={opened} />;
}

function Day({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const transit = useTransitControl();
  const id = useId();
  const link = dayLink(panel.settings);
  const place = usePanelPlace(panel, opened);
  const at: TransitAt = link === "today" ? "Now" : transit.at;
  const module = useDayPanchang(opened, place.place, at, transit.epoch, panel.id);
  const phase = usePanelLoadPhase(module.loading);
  const view = module.shown;
  return (
    <div className="module-panel muhurta-day" aria-busy={module.loading && view !== null ? true : undefined}>
      {/* Yer ve tarih bağı yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PlaceControls panel={panel} opened={opened} at={place} />
        <label htmlFor={`${id}-link`}>{t("muhurtaDay.linkLabel")}</label>
        <select id={`${id}-link`} value={link} onChange={(e) => setPanelSettings(panel.id, { link: e.target.value })}>
          {DAY_LINKS.map((l) => (
            <option key={l} value={l}>
              {t(`muhurtaDay.link.${l}`)}
            </option>
          ))}
        </select>
      </div>
      {link === "transit" && <TransitDate />}
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <DayBody view={view} unknown={timeUnknown(opened)} />
      )}
    </div>
  );
}

/** Kırpılan uç: aralık günün dışında başlıyor ya da sürüyor. İşaret ve ekran okuyucu metni. */
function Clipped({ on, mark, label, children }: { on: boolean; mark: string; label: string; children: ReactNode }) {
  if (!on) return <>{children}</>;
  return (
    <span>
      {children}{" "}
      <span aria-hidden="true" title={label}>
        {mark}
      </span>
      <span className="visually-hidden">{label}</span>
    </span>
  );
}

function DayBody({ view, unknown }: { view: DayPanchangView; unknown: boolean }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const limbsId = useId();
  const personalId = useId();
  const horasId = useId();
  const day = view.day;
  const clock = (jd: number | null) => formatJdClock(jd, view.offset_secs, day.day_start_jd_ut);
  const span = <T,>(i: Interval<T>) => ({
    start: (
      <Clipped on={i.start_jd_ut === day.day_start_jd_ut} mark="←" label={t("muhurtaDay.fromBefore")}>
        {clock(i.start_jd_ut)}
      </Clipped>
    ),
    end: (
      <Clipped on={i.end_jd_ut === day.day_end_jd_ut} mark="→" label={t("muhurtaDay.goesOn")}>
        {clock(i.end_jd_ut)}
      </Clipped>
    ),
  });
  const limbRow = <T,>(limb: string, i: Interval<T>, index: number, value: ReactNode) => {
    const { start, end } = span(i);
    return (
      <tr key={`${limb}:${index}`} data-limb={limb}>
        <th scope="row">{t(`panchang.${limb}`)}</th>
        <td>{value}</td>
        <td className="num">{start}</td>
        <td className="num">{end}</td>
      </tr>
    );
  };
  const noSunrise = day.warnings.includes("SunriseUnavailable");
  const personal = day.personal;
  return (
    <>
      <p className="muhurta-day-title">
        {t("muhurtaDay.title", { date: formatJdDate(day.day_start_jd_ut, view.offset_secs), vara: VARA_NAMES[day.vara], weekday: t(`panchang.weekday.${day.vara}`) })}
      </p>
      {noSunrise ? (
        <p className="panel-notice muhurta-warning" role="note">
          <span className="badge">
            <span className="badge-mark" aria-hidden="true">
              !
            </span>
            {t("badge.SunriseUnavailable")}
          </span>{" "}
          {t("muhurtaDay.noSunrise", { start: clock(day.day_start_jd_ut) })}
        </p>
      ) : (
        <p className="panel-message">
          {t("muhurtaDay.sun", { sunrise: clock(day.sunrise_jd_ut), sunset: clock(day.sunset_jd_ut), next: clock(day.next_sunrise_jd_ut) })}
        </p>
      )}
      {day.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {day.segments && <Strip day={day} clock={clock} />}
      <BalaScroll labelledBy={limbsId}>
        <table className="module-table muhurta-table">
          <caption id={limbsId}>{t("muhurtaDay.limbs")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("panchang.item")}</th>
              <th scope="col">{t("panchang.value")}</th>
              <th scope="col" className="num">
                {t("muhurtaDay.from")}
              </th>
              <th scope="col" className="num">
                {t("muhurtaDay.to")}
              </th>
            </tr>
          </thead>
          <tbody>
            {day.tithis.map((i, n) =>
              limbRow("tithi", i, n, `${i.value.paksha} ${tithiInPaksha(i.value.no)} · ${tithiName(i.value.no)} · ${t("muhurtaDay.group", { group: i.value.group })}`),
            )}
            {day.nakshatras.map((i, n) =>
              limbRow(
                "nakshatra",
                i,
                n,
                `${nakshatraName(i.value.no)} · ${t("panchang.pada", { pada: i.value.pada })} · ${NAKSHATRA_NATURE_NAMES[i.value.nature]} (${t(`muhurta.nature.${i.value.nature}`)})`,
              ),
            )}
            {day.yogas.map((i, n) => limbRow("yoga", i, n, nityaYogaName(i.value)))}
            {day.karanas.map((i, n) => limbRow("karana", i, n, i.value.name))}
          </tbody>
        </table>
      </BalaScroll>
      {day.flags.length === 0 ? (
        <p className="panel-message">{t("muhurtaDay.noFlags")}</p>
      ) : (
        <ul className="muhurta-flags" aria-label={t("muhurtaDay.flags")}>
          {day.flags.map((f, n) => {
            const { start, end } = span(f);
            return (
              <li key={`${f.value}:${n}`} data-flag={f.value}>
                <span className="muhurta-flag-name">{t(`muhurta.flag.${f.value}`)}</span> {start}–{end}
              </li>
            );
          })}
        </ul>
      )}
      {personal && (
        <BalaScroll labelledBy={personalId}>
          <table className="module-table muhurta-table">
            <caption id={personalId}>
              <Approx on={unknown} label={t("muhurtaDay.natalApprox")}>
                {t("muhurtaDay.personal")}
              </Approx>
            </caption>
            <thead>
              <tr>
                <th scope="col">{t("panchang.item")}</th>
                <th scope="col">{t("panchang.value")}</th>
                <th scope="col">{t("muhurtaDay.favourableColumn")}</th>
                <th scope="col" className="num">
                  {t("muhurtaDay.from")}
                </th>
                <th scope="col" className="num">
                  {t("muhurtaDay.to")}
                </th>
              </tr>
            </thead>
            <tbody>
              {personal.tara.map((i, n) => {
                const { start, end } = span(i);
                return (
                  <tr key={`tara:${n}`} data-personal="tara">
                    <th scope="row">{t("muhurta.tara")}</th>
                    <td>{t("muhurtaDay.taraValue", { number: i.value.number, name: taraName(i.value.number) })}</td>
                    <td>{t(i.value.favourable ? "muhurtaDay.favourable" : "muhurtaDay.notFavourable")}</td>
                    <td className="num">{start}</td>
                    <td className="num">{end}</td>
                  </tr>
                );
              })}
              {personal.chandra.map((i, n) => {
                const { start, end } = span(i);
                return (
                  <tr key={`chandra:${n}`} data-personal="chandra">
                    <th scope="row">{t("muhurta.chandra")}</th>
                    <td>{t("muhurtaDay.chandraValue", { house: i.value.house })}</td>
                    <td>{t(i.value.favourable ? "muhurtaDay.favourable" : "muhurtaDay.notFavourable")}</td>
                    <td className="num">{start}</td>
                    <td className="num">{end}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </BalaScroll>
      )}
      {day.segments && (
        <BalaScroll labelledBy={horasId}>
          <table className="module-table muhurta-table">
            <caption id={horasId}>{t("muhurtaDay.horas")}</caption>
            <thead>
              <tr>
                <th scope="col" className="num">
                  {t("muhurtaDay.horaNo")}
                </th>
                <th scope="col">{t("muhurtaDay.horaLord")}</th>
                <th scope="col" className="num">
                  {t("muhurtaDay.from")}
                </th>
                <th scope="col" className="num">
                  {t("muhurtaDay.to")}
                </th>
              </tr>
            </thead>
            <tbody>
              {day.segments.horas.map((h, n) => (
                <tr key={n} data-hora={n + 1} className={active?.type === "graha" && active.id === h.value ? "is-highlighted" : undefined}>
                  <td className="num">{n + 1}</td>
                  <th scope="row">
                    <PinButton target={{ type: "graha", id: h.value }}>{t(`grahaName.${h.value}`)}</PinButton>
                  </th>
                  <td className="num">{clock(h.start_jd_ut)}</td>
                  <td className="num">{clock(h.end_jd_ut)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </BalaScroll>
      )}
      <p className="panel-message">{t("muhurtaDay.note")}</p>
      <p className="panel-message">{t("muhurta.method", { method: day.method })}</p>
    </>
  );
}

/** Gün şeridi: gündüzün sekiz dilimi, işaretli üçü ve altında Abhijit. */
function Strip({ day, clock }: { day: DayPanchang; clock: (jd: number | null) => string }) {
  const { t } = useTranslation();
  const segments = day.segments;
  if (segments === null) return null;
  const markOf = (no: number) => MARKS.find((m) => segments[m].value === no) ?? null;
  // Abhijit çubuğunun yeri gündüzün içindeki payıdır (yalnızca çizim).
  const daylight = (day.sunset_jd_ut ?? 0) - (day.sunrise_jd_ut ?? 0);
  const share = (jd: number | null) => (daylight > 0 && jd !== null ? Math.min(100, Math.max(0, (((jd - (day.sunrise_jd_ut ?? 0)) / daylight) * 100))) : 0);
  const left = share(segments.abhijit.start_jd_ut);
  const width = share(segments.abhijit.end_jd_ut) - left;
  return (
    <div className="day-strip-box">
      <ol className="day-strip" aria-label={t("muhurtaDay.strip")}>
        {segments.eighths.map((e) => {
          const mark = markOf(e.value);
          return (
            <li key={e.value} data-segment={e.value} data-mark={mark ?? undefined} className={mark ? "is-marked" : undefined}>
              <span className="day-strip-no">{e.value}</span>
              {mark && <strong className="day-strip-mark">{t(`muhurtaDay.mark.${mark}`)}</strong>}
              <span className="day-strip-time">
                {clock(e.start_jd_ut)}–{clock(e.end_jd_ut)}
              </span>
            </li>
          );
        })}
      </ol>
      <div className="day-strip-abhijit" aria-hidden="true">
        <span style={{ marginLeft: `${left}%`, width: `${width}%` }}>{t("muhurtaDay.mark.abhijit")}</span>
      </div>
      <p className="panel-message" data-abhijit>
        {t("muhurtaDay.abhijit", { start: clock(segments.abhijit.start_jd_ut), end: clock(segments.abhijit.end_jd_ut) })}
        {segments.abhijit_weekday_exception && ` ${t("muhurtaDay.abhijitWednesday")}`}
      </p>
    </div>
  );
}
