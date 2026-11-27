import { expect, test } from "vitest";
import type { CalendarDayView, SkyEvent } from "../bindings";
import { i18n, LANGUAGES } from "../i18n";
import { calendarFixture } from "../test/calendarFixture";
import { CALENDAR_KINDS, CELL_EVENTS, DEFAULT_CALENDAR_KINDS, eventLong, eventShort, eventWhere, MAX_EVENTS, MAX_YEAR, MIN_YEAR, parseFilter, WEEK_START, weekdayOrder, weekRows } from "./model";

const t = i18n.getFixedT("tr");
const days = (row: (CalendarDayView | null)[]) => row.map((d) => d?.day ?? null);

test("the kind lists and the year range are Rust's", () => {
  const { ids } = calendarFixture();
  expect([...CALENDAR_KINDS]).toEqual(ids.kinds);
  expect([...DEFAULT_CALENDAR_KINDS]).toEqual(ids.default_kinds);
  expect([MIN_YEAR, MAX_YEAR]).toEqual([ids.min_year, ids.max_year]);
  expect(MAX_EVENTS).toBe(ids.max_events);
  expect(CELL_EVENTS).toBe(3);
});

test("the week starts on Monday in Turkish and French and on Sunday in English", () => {
  expect(Object.keys(WEEK_START).sort()).toEqual([...LANGUAGES].sort());
  expect(WEEK_START).toEqual({ tr: "Monday", en: "Sunday", fr: "Monday" });
  expect(weekdayOrder("Monday")).toEqual(["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]);
  expect(weekdayOrder("Sunday")).toEqual(["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
});

test("days sit in the column of their weekday", () => {
  const month = calendarFixture().month.days;
  // 1 Kasım 2026 Pazar'dır. Pazartesi başlayan haftada ilk satırın yalnızca son hücresi doludur.
  const monday = weekRows(month, "Monday");
  expect(monday).toHaveLength(6);
  expect(days(monday[0])).toEqual([null, null, null, null, null, null, 1]);
  expect(days(monday[1])).toEqual([2, 3, 4, 5, 6, 7, 8]);
  expect(days(monday[5])).toEqual([30, null, null, null, null, null, null]);
  // Pazar başlayan haftada ay beş satırdır.
  const sunday = weekRows(month, "Sunday");
  expect(sunday).toHaveLength(5);
  expect(days(sunday[0])).toEqual([1, 2, 3, 4, 5, 6, 7]);
  expect(days(sunday[4])).toEqual([29, 30, null, null, null, null, null]);
  // Her gün tam bir kez yerleşir.
  expect(monday.flat().filter((d) => d !== null)).toHaveLength(30);
  expect(weekRows([], "Monday")).toEqual([]);
});

test("a day the time zone skipped leaves its cell empty and the rest stay in their columns", () => {
  // Takvimden atlanan gün (Rust o günü listeye koymaz): 11'i olmayan bir ay.
  const month = calendarFixture().month.days.filter((d) => d.day !== 11);
  const rows = weekRows(month, "Monday");
  expect(days(rows[2])).toEqual([9, 10, null, 12, 13, 14, 15]);
  expect(rows).toHaveLength(6);
});

test("the filter is read tolerantly from the panel settings", () => {
  expect(parseFilter({})).toEqual({ kinds: [...DEFAULT_CALENDAR_KINDS], moon: false });
  // Tanınmayan türler atılır; sıra Rust'ın sırasıdır.
  expect(parseFilter({ kinds: ["TransitToNatal", "Conjunction", "SignIngress", 7], moon: true })).toEqual({ kinds: ["SignIngress", "TransitToNatal"], moon: true });
  // Boş küme geçerli bir seçimdir (yalnızca olgular); bozuk değer varsayılandır.
  expect(parseFilter({ kinds: [] }).kinds).toEqual([]);
  expect(parseFilter({ kinds: "all", moon: "yes" })).toEqual({ kinds: [...DEFAULT_CALENDAR_KINDS], moon: false });
});

test("an event has a short cell text, a full name and a place", () => {
  // Fikstürdeki gerçek bir olaydan kurulur: `SkyEvent` yeni bir alan kazanırsa test derlenmeyi sürdürür.
  const base = calendarFixture().month.days[8].events[0].event;
  const event = (kind: SkyEvent["kind"], grahas: SkyEvent["grahas"], sign: SkyEvent["sign"] = null, nakshatra: number | null = null): SkyEvent => ({ ...base, kind, grahas, sign, nakshatra, house: null });
  const rows: [SkyEvent, string, string, string][] = [
    [event("SignIngress", ["Sun"], "Scorpio"), "Gü → Akr", "Güneş: burç girişi", "Akr"],
    [event("NakshatraIngress", ["Mercury"], null, 16), "Me → Vishakha", "Merkür: nakshatra girişi", "Vishakha"],
    [event("StationRetrograde", ["Jupiter"], "Leo"), "Jü R başlar", "Jüpiter: R başlıyor", "Asl"],
    [event("StationDirect", ["Venus"], "Virgo"), "Ve R biter", "Venüs: R bitiyor", "Baş"],
    [event("NewMoon", ["Sun", "Moon"], "Libra", 16), "Yeni ay", "Yeni ay", "Ter · Vishakha"],
    [event("FullMoon", ["Sun", "Moon"], "Taurus", 3), "Dolunay", "Dolunay", "Boğ · Krittika"],
    [event("TransitToNatal", ["Mars", "Moon"], "Libra"), "Ma → doğum Ay", "Mars, doğum Ay boylamında", "Ter"],
    [event("TransitToNatal", ["Sun", "Sun"], "Sagittarius"), "Gü dönüşü", "Güneş dönüşü", "Yay"],
  ];
  for (const [e, short, long, where] of rows) {
    expect([eventShort(t, e), eventLong(t, e), eventWhere(t, e)], e.kind).toEqual([short, long, where]);
  }
  // Burcu ya da nakshatrası gelmeyen giriş çizgiyle yazılır.
  expect(eventShort(t, event("SignIngress", ["Sun"]))).toBe("Gü → —");
  expect(eventWhere(t, event("SignIngress", ["Sun"]))).toBe("");
});
