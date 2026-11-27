// Ay ızgarası (Plan 29): hafta günleri sütun, haftalar satırdır. Hücre takvim günüdür: ayın günü,
// gün doğumundaki tithi ve Ay nakshatrası ve o güne düşen olayların ilk birkaçı. Değerler
// `month_calendar`'dan gelir; burada hesap yoktur (C4). Kısaltmaların tam adı ekran okuyucuya
// ayrıca verilir; işaretler yazıdır (renk tek başına anlam taşımaz, C9).
//
// Izgara tek bir sekme durağıdır (gezici `tabindex`): ok tuşları günler arasında gezer, Home ve
// End haftanın, Ctrl+Home ve Ctrl+End ayın ucuna gider, Page Up ve Page Down ayı değiştirir,
// Enter ya da boşluk günü seçer. Gezinme yerleşimdir, takvim hesabı değil: komşu gün, Rust'ın
// verdiği günlerin satır ve sütunundan bulunur.
import { useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarDayView, CalendarView } from "../bindings";
import { currentLanguage } from "../i18n";
import { formatJdClock } from "../panels/format";
import { nakshatraName, tithiInPaksha, tithiName, tithiShort } from "../panels/labels";
import { MAIN_ID } from "../shell/focus";
import { CELL_EVENTS, eventLong, eventShort, eventWhere, WEEK_START, weekdayOrder, weekRows } from "./model";

/** Gün doğumu olmayan günün işareti (dipnotla açıklanır). */
export const NO_SUNRISE_MARK = "*";

export interface CalendarGridProps {
  view: CalendarView;
  /** Seçili günün tarihi (`YYYY-MM-DD`); bu ayda değilse hiçbir hücre seçili değildir. */
  selected: string | null;
  onSelect(date: string): void;
  /** Page Up (−1) ya da Page Down (+1): ay değişir ve odak yeni ayda `day` gününe taşınır. */
  onPage(step: -1 | 1, day: number): void;
  /** Ay değiştikten sonra odağın taşınacağı gün; o gün yoksa ayın daha küçük en yakın günü. */
  focusDay: number | null;
  onFocused(): void;
}

export function CalendarGrid({ view, selected, onSelect, onPage, focusDay, onFocused }: CalendarGridProps) {
  const { t } = useTranslation();
  const captionId = useId();
  const hintId = useId();
  const wrap = useRef<HTMLDivElement>(null);
  const start = WEEK_START[currentLanguage()];
  const order = weekdayOrder(start);
  const rows = weekRows(view.days, start);
  const month = t(`calendar.month.${view.month.month}`);
  // Gezici sekme durağı: son odaklanan gün; yoksa seçili gün, bugün ya da ayın ilk günü.
  const [cursor, setCursor] = useState<string | null>(null);
  const has = (date: string | null) => date !== null && view.days.some((d) => d.date === date);
  const todayDate = view.days.find((d) => d.day === view.today)?.date ?? null;
  const stop = has(cursor) ? cursor : has(selected) ? selected : (todayDate ?? view.days[0]?.date ?? null);

  const focus = (day: CalendarDayView | null | undefined) => {
    if (!day) return;
    setCursor(day.date);
    wrap.current?.querySelector<HTMLElement>(`td[data-date="${day.date}"]`)?.focus();
  };
  // Ay değişti (Page Up ya da Page Down): odak aynı güne, o gün yoksa ayın son gününe taşınır.
  const monthKey = `${view.month.year}-${view.month.month}`;
  useLayoutEffect(() => {
    if (focusDay === null) return;
    const target = [...view.days].reverse().find((d) => d.day <= focusDay) ?? view.days[0];
    focus(target);
    onFocused();
    // Yalnızca ay ya da istenen gün değişince.
  }, [monthKey, focusDay]);
  // Izgara odaktayken DOM'dan çıkarsa (hata) odak gövdeye düşmez: panelin çerçevesine taşınır.
  useLayoutEffect(() => {
    const el = wrap.current;
    return () => {
      if (el === null || !el.contains(document.activeElement)) return;
      const target = el.closest<HTMLElement>(".panel") ?? document.getElementById(MAIN_ID);
      target?.focus({ preventScroll: true });
    };
  }, []);

  const onKeyDown = (e: KeyboardEvent<HTMLTableCellElement>, day: CalendarDayView) => {
    const r = rows.findIndex((row) => row.includes(day));
    const c = rows[r].indexOf(day);
    const i = view.days.indexOf(day);
    const inRow = rows[r].filter((d) => d !== null);
    let target: CalendarDayView | null | undefined;
    switch (e.key) {
      case "ArrowLeft":
        target = view.days[i - 1];
        break;
      case "ArrowRight":
        target = view.days[i + 1];
        break;
      case "ArrowUp":
        target = rows[r - 1]?.[c];
        break;
      case "ArrowDown":
        target = rows[r + 1]?.[c];
        break;
      case "Home":
        target = e.ctrlKey || e.metaKey ? view.days[0] : inRow[0];
        break;
      case "End":
        target = e.ctrlKey || e.metaKey ? view.days.at(-1) : inRow.at(-1);
        break;
      case "PageUp":
      case "PageDown":
        e.preventDefault();
        onPage(e.key === "PageUp" ? -1 : 1, day.day);
        return;
      case "Enter":
      case " ":
        e.preventDefault();
        onSelect(day.date);
        return;
      default:
        return;
    }
    // Ok tuşu sayfayı kaydırmaz; ayın ucunda odak yerinde durur.
    e.preventDefault();
    focus(target);
  };

  return (
    <>
      <div ref={wrap} className="bala-scroll calendar-scroll">
        <table className="calendar-grid" role="grid" aria-labelledby={captionId} aria-describedby={hintId}>
          <caption id={captionId}>{t("calendar.caption", { month, year: view.month.year })}</caption>
          <thead>
            <tr>
              {order.map((v) => (
                <th key={v} scope="col" abbr={t(`panchang.weekday.${v}`)}>
                  <span aria-hidden="true">{t(`calendar.weekdayShort.${v}`)}</span>
                  <span className="visually-hidden">{t(`panchang.weekday.${v}`)}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, r) => (
              <tr key={row.find((d) => d !== null)?.date ?? r}>
                {row.map((day, c) =>
                  day === null ? (
                    <td key={`empty-${c}`} className="calendar-empty" />
                  ) : (
                    <DayCell
                      key={day.date}
                      day={day}
                      today={view.today === day.day}
                      selected={selected === day.date}
                      tabStop={stop === day.date}
                      onPick={() => {
                        setCursor(day.date);
                        onSelect(day.date);
                      }}
                      onFocus={() => setCursor(day.date)}
                      onKeyDown={(e) => onKeyDown(e, day)}
                    />
                  ),
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p id={hintId} className="chart-toolbar-hint">
        {t("calendar.gridHint")}
      </p>
    </>
  );
}

interface DayCellProps {
  day: CalendarDayView;
  today: boolean;
  selected: boolean;
  tabStop: boolean;
  onPick(): void;
  onFocus(): void;
  onKeyDown(e: KeyboardEvent<HTMLTableCellElement>): void;
}

function DayCell({ day, today, selected, tabStop, onPick, onFocus, onKeyDown }: DayCellProps) {
  const { t } = useTranslation();
  const { facts } = day;
  const shown = day.events.slice(0, CELL_EVENTS);
  const more = day.events.length - shown.length;
  const classes = ["calendar-day", today ? "is-today" : null, selected ? "is-selected" : null].filter((c) => c !== null).join(" ");
  return (
    <td className={classes} data-date={day.date} tabIndex={tabStop ? 0 : -1} aria-selected={selected} onClick={onPick} onFocus={onFocus} onKeyDown={onKeyDown}>
      <div className="calendar-day-head">
        <span className="calendar-day-number">{day.day}</span>
        {today && <span className="calendar-today">{t("calendar.today")}</span>}
      </div>
      <div className="calendar-tithi">
        <span aria-hidden="true">
          {tithiShort(facts.tithi)} {tithiName(facts.tithi)}
        </span>
        <span className="visually-hidden">{t("calendar.tithi", { paksha: facts.paksha, number: tithiInPaksha(facts.tithi), name: tithiName(facts.tithi) })}</span>
        {facts.sunrise_jd_ut === null && (
          <>
            <span aria-hidden="true" title={t("calendar.noSunrise")}>
              {NO_SUNRISE_MARK}
            </span>
            <span className="visually-hidden"> {t("calendar.noSunrise")}</span>
          </>
        )}
      </div>
      <div className="calendar-nakshatra">{nakshatraName(facts.nakshatra)}</div>
      {day.events.length > 0 && (
        <ul className="calendar-events">
          {shown.map((row, n) => {
            const clock = formatJdClock(row.event.jd_ut, row.offset_secs, null);
            const where = eventWhere(t, row.event);
            return (
              <li key={`${n}:${row.event.jd_ut}`} data-event={row.event.kind}>
                <span className="calendar-event-time">{clock}</span>{" "}
                <span aria-hidden="true">{eventShort(t, row.event)}</span>
                <span className="visually-hidden">{where === "" ? eventLong(t, row.event) : `${eventLong(t, row.event)} · ${where}`}</span>
              </li>
            );
          })}
          {more > 0 && <li className="calendar-more">{t("calendar.more", { n: more })}</li>}
        </ul>
      )}
    </td>
  );
}
