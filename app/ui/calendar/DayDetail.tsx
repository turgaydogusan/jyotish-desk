// Gün ayrıntısı (Plan 29): seçili takvim gününün gün doğumu olguları ve o güne düşen bütün
// olaylar, saatleriyle. Değerler `month_calendar`'dan gelir; burada hesap yoktur (C4). "Gün
// panchangını aç" transit tarihini Rust'ın verdiği ana götürür (o gün doğumuyla başlayan günün
// içinde) ve Gün panchangı paneli düzende yoksa ekler; olayın saati transit tarihini o ana
// götürür. Olay bir olgudur: hüküm ve açıklama metni yoktur.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarDayView, CalendarView } from "../bindings";
import { formatJdClock } from "../panels/format";
import { nakshatraName, tithiInPaksha, tithiName } from "../panels/labels";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { eventLong, eventWhere } from "./model";

export function DayDetail({ view, day }: { view: CalendarView; day: CalendarDayView | null }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const { active, togglePanel } = useLayoutStore();
  const titleId = useId();
  const captionId = useId();
  if (day === null) return <p className="panel-message">{t("calendar.pickDay")}</p>;
  const { facts } = day;
  const title = t("calendar.dayTitle", { day: day.day, month: t(`calendar.month.${view.month.month}`), year: view.month.year, weekday: t(`panchang.weekday.${day.weekday}`) });
  const tithi = t("calendar.tithi", { paksha: facts.paksha, number: tithiInPaksha(facts.tithi), name: tithiName(facts.tithi) });
  const openDay = () => {
    const at = day.panchang_jd_ut;
    if (at === null) return;
    transit.setAt({ Jd: at });
    if (!active.panels.some((p) => p.kind === "muhurta_day")) togglePanel("muhurta_day");
  };
  return (
    <section className="calendar-detail" aria-labelledby={titleId} data-detail={day.date}>
      <p id={titleId} className="calendar-detail-title" aria-live="polite">
        {title}
      </p>
      <p className="panel-message">
        {t(facts.sunrise_jd_ut === null ? "calendar.fallbackFacts" : "calendar.sunriseFacts", { time: formatJdClock(facts.jd_ut, day.offset_secs, null), tithi, nakshatra: nakshatraName(facts.nakshatra) })}
      </p>
      <div className="module-toolbar">
        <button type="button" title={t("calendar.openDayHint")} onClick={openDay}>
          {t("calendar.openDay")}
        </button>
      </div>
      {day.events.length === 0 ? (
        <p className="panel-message">{t("calendar.noEvents")}</p>
      ) : (
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={captionId}>
          <table className="module-table">
            <caption id={captionId}>{t("calendar.events", { date: title })}</caption>
            <thead>
              <tr>
                <th scope="col">{t("calendar.col.when")}</th>
                <th scope="col">{t("calendar.col.event")}</th>
                <th scope="col">{t("calendar.col.where")}</th>
              </tr>
            </thead>
            <tbody>
              {day.events.map((row, n) => {
                const jd = row.event.jd_ut;
                const when = formatJdClock(jd, row.offset_secs, null);
                return (
                  <tr key={`${n}:${jd}`} data-event={row.event.kind}>
                    <th scope="row">
                      {jd === null ? (
                        when
                      ) : (
                        <button type="button" className="link-button" aria-label={t("calendar.goTo", { when })} onClick={() => transit.setAt({ Jd: jd })}>
                          {when}
                        </button>
                      )}
                    </th>
                    <td>{eventLong(t, row.event)}</td>
                    <td>{eventWhere(t, row.event)}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
