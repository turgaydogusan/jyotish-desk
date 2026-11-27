// Geçiş takviminin fikstürü (Plan 29): `delhi.json`'un `calendar` alanı. Ayrı dosyadır:
// `fixture.ts` paralel planlarla paylaşılır; bu dosya aynı fikstürü okur ve yalnızca takvimin
// alanını tipler. Değerler `ui_fixture.rs`'te gerçek motorla üretilir (arayüz hesap yapmaz).
import type { CalendarRequest, CalendarView, SkyEventKind } from "../bindings";
import { fixture } from "./fixture";

export interface CalendarFixture {
  /** Bulunulan yerin (Delhi) bugünkü ayı, varsayılan süzgeçle; açık haritanın girdisi gitmez. */
  request: CalendarRequest;
  /** Aynı ay: bütün türler, Ay ve açık haritanın girdisi. */
  request_full: CalendarRequest;
  /** `month_calendar(request)`: Kasım 2026; bugün ayın 25'idir. */
  month: CalendarView;
  /** `month_calendar(request_full)`: doğum boylamına gelişler ve Ay'ın girişleriyle. */
  month_full: CalendarView;
  /** Sonraki ay (Aralık 2026), varsayılan süzgeçle. */
  month_next: CalendarView;
  /** Tromsø, Ocak 2027: Güneş ayın 19'una dek doğmaz. */
  month_polar: CalendarView;
  /** Rust `CALENDAR_KINDS`, `CALENDAR_DEFAULT_KINDS`, `CALENDAR_MAX_EVENTS` ve `CALENDAR_YEARS`. */
  ids: { kinds: SkyEventKind[]; default_kinds: SkyEventKind[]; max_events: number; min_year: number; max_year: number };
}

/** Her çağrıda derin kopya (`fixture()` gibi). */
export function calendarFixture(): CalendarFixture {
  return (fixture() as unknown as { calendar: CalendarFixture }).calendar;
}
