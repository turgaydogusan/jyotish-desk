// Geçiş takviminin arayüz modeli (yol haritası §4.11, Plan 29): süzgecin panel ayarından
// okunması, günlerin hafta satırlarına dizilmesi ve olayın hücredeki kısa yazısı. Burada hesap
// yoktur (C4): günler, hafta günleri, olgular, olaylar, önceki ve sonraki ay Rust'tan gelir;
// buradaki her şey seçim listesi, yerleşim ve gösterimdir.
import type { TFunction } from "i18next";
import type { CalendarDayView, CalendarFilter, SkyEvent, SkyEventKind, Vara } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { Language } from "../i18n";
import { VARAS } from "../modules/muhurta";
import { nakshatraName } from "../panels/labels";

/** Rust `CALENDAR_KINDS`: takvimde gösterilebilen olay türleri, süzgeçteki sırayla. */
export const CALENDAR_KINDS = ["SignIngress", "NakshatraIngress", "StationRetrograde", "StationDirect", "NewMoon", "FullMoon", "TransitToNatal"] as const satisfies readonly SkyEventKind[];
/** Rust `CALENDAR_DEFAULT_KINDS`: yeni panelin türleri. */
export const DEFAULT_CALENDAR_KINDS = ["SignIngress", "NakshatraIngress", "StationRetrograde", "StationDirect", "NewMoon", "FullMoon"] as const satisfies readonly SkyEventKind[];
/** Rust `CALENDAR_YEARS`. */
export const MIN_YEAR = 1;
export const MAX_YEAR = 2999;
/** Rust `CALENDAR_MAX_EVENTS`: bir ayın taşıdığı en çok olay. */
export const MAX_EVENTS = 300;
/** Bir hücrede yazılan en çok olay; fazlası sayılır ve gün ayrıntısında listelenir. */
export const CELL_EVENTS = 3;

/** Haftanın ilk günü, arayüz diline göre: Türkçe ve Fransızcada Pazartesi, İngilizcede Pazar. */
export const WEEK_START: Record<Language, Vara> = { tr: "Monday", en: "Sunday", fr: "Monday" };

/** Sütun sırası: haftanın ilk gününden başlayan yedi gün. */
export function weekdayOrder(start: Vara): Vara[] {
  const first = VARAS.indexOf(start);
  return VARAS.map((_, i) => VARAS[(first + i) % 7]);
}

/**
 * Günlerin hafta satırları: her gün kendi hafta gününün sütununda durur, boş hücreler `null`'dır.
 * Sütunu Rust'ın verdiği hafta günü belirler; takvimden gün atlayan bir saat diliminde de
 * (ardışık iki günün hafta günü bir gün atlar) her gün doğru sütundadır.
 */
export function weekRows(days: readonly CalendarDayView[], start: Vara): (CalendarDayView | null)[][] {
  const order = weekdayOrder(start);
  const rows: (CalendarDayView | null)[][] = [];
  // İlk gün her zaman yeni bir satır açar.
  let last = 7;
  for (const day of days) {
    const col = order.indexOf(day.weekday);
    if (col <= last) rows.push([null, null, null, null, null, null, null]);
    rows[rows.length - 1][col] = day;
    last = col;
  }
  return rows;
}

/** Panel ayarındaki süzgecin toleranslı okuması: tanınmayan türler atılır; anahtar yoksa varsayılan. */
export function parseFilter(settings: Record<string, unknown>): CalendarFilter {
  const raw = settings.kinds;
  const kinds: SkyEventKind[] = Array.isArray(raw) ? CALENDAR_KINDS.filter((k) => raw.includes(k)) : [...DEFAULT_CALENDAR_KINDS];
  return { kinds, moon: settings.moon === true };
}

/** Olayın hücredeki kısa yazısı: graha ve burç kısaltmaları, nakshatra adı. */
export function eventShort(t: TFunction, e: SkyEvent): string {
  const [a, b] = e.grahas;
  const graha = t(`graha.${a}`);
  switch (e.kind) {
    case "SignIngress":
      return t("calendar.short.SignIngress", { graha, sign: e.sign === null ? "—" : t(`sign.${signIndex(e.sign)}`) });
    case "NakshatraIngress":
      return t("calendar.short.NakshatraIngress", { graha, nakshatra: e.nakshatra === null ? "—" : nakshatraName(e.nakshatra) });
    case "TransitToNatal":
      return a === b ? t("calendar.short.return", { graha }) : t("calendar.short.TransitToNatal", { a: graha, b: t(`graha.${b}`) });
    case "NewMoon":
    case "FullMoon":
      return t(`calendar.short.${e.kind}`);
    default:
      return t(`calendar.short.${e.kind}`, { graha });
  }
}

/** Olayın tam adı (olay araması panelinin metinleriyle): ekran okuyucu ve gün ayrıntısı için. */
export function eventLong(t: TFunction, e: SkyEvent): string {
  const [a, b] = e.grahas;
  const name = (g: string) => t(`grahaName.${g}`);
  switch (e.kind) {
    case "Conjunction":
      return t("eventSearch.event.Conjunction", { a: name(a), b: name(b) });
    case "TransitToNatal":
      return a === b ? t("eventSearch.event.return", { graha: name(a) }) : t("eventSearch.event.TransitToNatal", { a: name(a), b: name(b) });
    case "HouseFromNatalMoon":
      return t("eventSearch.event.HouseFromNatalMoon", { graha: name(a), house: e.house ?? "—" });
    case "NewMoon":
    case "FullMoon":
      return t(`eventSearch.event.${e.kind}`);
    default:
      return t(`eventSearch.event.${e.kind}`, { graha: name(a) });
  }
}

/** Olayın burcu ve nakshatrası: "Akrep · Anuradha"; ikisi de yoksa boş. */
export function eventWhere(t: TFunction, e: SkyEvent): string {
  return [e.sign === null ? null : t(`sign.${signIndex(e.sign)}`), e.nakshatra === null ? null : nakshatraName(e.nakshatra)].filter((x) => x !== null).join(" · ");
}
