// Transit takvimi paneli (yol haritası §4.11, Plan 29): seçilen yerin takvim ayı. Her hücre bir
// takvim günüdür: gün doğumundaki tithi ve Ay nakshatrası ile o güne düşen gök olayları. Ay,
// günler, olgular ve olaylar `month_calendar`'dan gelir; burada hesap yoktur (C4): "bugün",
// önceki ve sonraki ay da Rust'tan okunur. Yer panelin ayarıdır ve düzenle birlikte saklanır
// (varsayılan: bulunulan yer); gösterilen ay saklanmaz, panel her açılışta bugünün ayını gösterir.
// Izgaranın altında seçili günün ayrıntısı durur: seçim yoksa bugünün (bu aydaysa). Olay
// süzgeci (türler ve Ay'ın olayları) de panelin ayarıdır.
// Takvim olgudur: "uygun gün" hükmü, puan ve açıklama metni yoktur.
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarRequest, CalendarView, CommandError, OpenedClient, YearMonth } from "../bindings";
import type { Panel } from "../layout/model";
import { chartInput } from "../modules/input";
import { muhurtaPlace } from "../modules/muhurta";
import { PlaceControls, PlaceMissing, usePanelPlace } from "../panels/MuhurtaPlace";
import "../panels/panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import "./calendar.css";
import { CalendarGrid, NO_SUNRISE_MARK, type CalendarGridProps } from "./CalendarGrid";
import { DayDetail } from "./DayDetail";
import { EventFilter } from "./EventFilter";
import { MAX_EVENTS, MAX_YEAR, MIN_YEAR, parseFilter } from "./model";
import { useCalendar } from "./useCalendar";

const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;

export function CalendarPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  // Başka harita: panel bugünün ayına döner.
  return <Calendar key={opened.chart_record.id} panel={panel} opened={opened} />;
}

function Calendar({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const place = usePanelPlace(panel, opened);
  // `null`: yerin bugünkü ayı ("bugün" Rust'ta okunur).
  const [month, setMonth] = useState<YearMonth | null>(null);
  const [epoch, setEpoch] = useState(0);
  // Seçili günün tarihi; ızgaradan Page Up ya da Page Down ile ay değişince odağın gideceği gün.
  const [selected, setSelected] = useState<string | null>(null);
  const [pending, setPending] = useState<{ month: YearMonth; day: number } | null>(null);
  const filter = parseFilter(panel.settings);
  // Doğum verisi yalnızca doğum boylamına gelişler seçiliyse gönderilir.
  const natal = filter.kinds.includes("TransitToNatal");
  const request: CalendarRequest | null =
    place.place === null ? null : { month, place: muhurtaPlace(place.place), filter, settings: opened.chart.meta.settings, natal: natal ? chartInput(opened) : null };
  const module = useCalendar(request, epoch, `${opened.chart_record.id}:${panel.id}`);
  const phase = usePanelLoadPhase(module.loading);
  const view = module.shown;
  const goTo = (next: YearMonth | null) => {
    setMonth(next);
    // "Bu ay" her seçildiğinde yeniden okunur: Rust'ın saati ilerlemiş olabilir.
    if (next === null) setEpoch((n) => n + 1);
  };
  // Hata yalnızca alan adını taşır; ad arayüz dilinde yazılır.
  const shownError = module.error === null ? null : fieldError(module.error, (f) => (i18n.exists(`calendar.field.${f}`) ? t(`calendar.field.${f}`) : f));
  return (
    <div className="module-panel calendar" aria-busy={module.loading && view !== null ? true : undefined}>
      {/* Yer ve ay seçimi yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PlaceControls panel={panel} opened={opened} at={place} />
      </div>
      <MonthNav view={view} requested={month} onMonth={goTo} />
      <EventFilter panel={panel} filter={filter} natalApprox={natal && timeUnknown(opened)} />
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={module.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <Month
          view={view}
          selected={selected}
          onSelect={setSelected}
          onPage={(step, day) => {
            const to = step < 0 ? view.prev : view.next;
            if (to === null) return;
            setPending({ month: to, day });
            goTo(to);
          }}
          focusDay={pending !== null && pending.month.year === view.month.year && pending.month.month === view.month.month ? pending.day : null}
          onFocused={() => setPending(null)}
        />
      )}
    </div>
  );
}

/** `invalid_calendar_input`'un alan adını verilen adla değiştirir; öteki hatalar olduğu gibi geçer. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_calendar_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

/**
 * Ay seçimi: önceki ve sonraki ay (Rust'ın verdiği aylar), ay ve yıl alanları, "Bu ay". Alanlar
 * istenen ayı yazar (istek hatayla bitse de başka bir ay seçilebilsin); ay istenmediyse ("bu
 * ay") yanıttaki ayı. İlk yanıt gelene dek kullanılamazlar. Kullanılamayan düğme yerel
 * `disabled` değildir: odaklanır ve basılınca bir şey yapmaz.
 */
function MonthNav({ view, requested, onMonth }: { view: CalendarView | null; requested: YearMonth | null; onMonth(month: YearMonth | null): void }) {
  const { t } = useTranslation();
  const id = useId();
  const shown = requested ?? view?.month ?? null;
  const step = (to: YearMonth | null | undefined, label: string, mark: string) => (
    <button type="button" aria-label={label} title={label} aria-disabled={to ? undefined : true} onClick={() => to && onMonth(to)}>
      <span aria-hidden="true">{mark}</span>
    </button>
  );
  return (
    <div className="module-toolbar calendar-nav" role="group" aria-label={t("calendar.nav")}>
      {step(view?.prev, t("calendar.prev"), "‹")}
      <label htmlFor={`${id}-month`} className="visually-hidden">
        {t("calendar.monthLabel")}
      </label>
      <select id={`${id}-month`} value={shown?.month ?? ""} disabled={shown === null} onChange={(e) => shown && onMonth({ year: shown.year, month: Number(e.target.value) })}>
        {shown === null && <option value="">—</option>}
        {MONTHS.map((m) => (
          <option key={m} value={m}>
            {t(`calendar.month.${m}`)}
          </option>
        ))}
      </select>
      <YearField key={shown?.year ?? "none"} id={`${id}-year`} shown={shown} onMonth={onMonth} />
      {step(view?.next, t("calendar.next"), "›")}
      <button type="button" onClick={() => onMonth(null)}>
        {t("calendar.thisMonth")}
      </button>
    </div>
  );
}

/** Yıl alanı: Enter ya da alandan çıkışla uygulanır; aralık dışı ya da tam sayı olmayan değer uygulanmaz. */
function YearField({ id, shown, onMonth }: { id: string; shown: YearMonth | null; onMonth(month: YearMonth): void }) {
  const { t } = useTranslation();
  const [text, setText] = useState(shown === null ? "" : String(shown.year));
  const year = /^\d{1,4}$/.test(text.trim()) ? Number(text.trim()) : null;
  const valid = year !== null && year >= MIN_YEAR && year <= MAX_YEAR;
  const apply = () => {
    if (shown === null) return;
    if (valid && year !== shown.year) onMonth({ year, month: shown.month });
    // Uygulanmayan değer gösterilen yıla döner.
    else if (!valid) setText(String(shown.year));
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    apply();
  };
  return (
    <form className="calendar-year" onSubmit={submit}>
      <label htmlFor={id} className="visually-hidden">
        {t("calendar.yearLabel")}
      </label>
      <input id={id} type="text" inputMode="numeric" size={5} maxLength={4} value={text} disabled={shown === null} aria-invalid={shown !== null && !valid ? true : undefined} onChange={(e) => setText(e.target.value)} onBlur={apply} />
    </form>
  );
}

function Month({ view, selected, ...grid }: CalendarGridProps) {
  const { t } = useTranslation();
  const noSunrise = view.days.some((d) => d.facts.sunrise_jd_ut === null);
  // Ayrıntı: seçili gün bu aydaysa o, değilse bugün (bu aydaysa).
  const shown = view.days.find((d) => d.date === selected) ?? view.days.find((d) => d.day === view.today) ?? null;
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {view.truncated && (
        <p className="panel-notice" role="note">
          {t("calendar.truncated", { max: MAX_EVENTS })}
        </p>
      )}
      <CalendarGrid view={view} selected={shown?.date ?? null} {...grid} />
      {noSunrise && (
        <p className="panel-notice" role="note">
          {t("calendar.noSunriseNote", { mark: NO_SUNRISE_MARK })}
        </p>
      )}
      <DayDetail view={view} day={shown} />
      <p className="panel-message">{t("calendar.method", { method: view.method })}</p>
    </>
  );
}
