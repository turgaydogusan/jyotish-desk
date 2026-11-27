import { calendarFixture } from "./calendarFixture";
import { fixture } from "./fixture";

test("the calendar fixture is a real month for the fixture place with the facts of every day", () => {
  const c = calendarFixture();
  expect(c.request.place).toEqual(fixture().muhurta.place);
  expect(c.request.month).toBeNull();
  expect(c.request.natal).toBeNull();
  expect(c.month.month).toEqual({ year: 2026, month: 11 });
  expect([c.month.prev, c.month.next]).toEqual([
    { year: 2026, month: 10 },
    { year: 2026, month: 12 },
  ]);
  expect(c.month.today).toBe(25);
  expect(c.month.days.map((d) => d.day)).toEqual(Array.from({ length: 30 }, (_, i) => i + 1));
  expect(c.month.days[0]).toMatchObject({ date: "2026-11-01", weekday: "Sunday" });
  // Bugünün gün doğumu olguları gün panchangı fikstürüyle aynıdır (aynı an, aynı yer).
  const today = c.month.days[24];
  const day = fixture().muhurta.day.day;
  expect(today.facts.sunrise_jd_ut).toBeCloseTo(day.sunrise_jd_ut as number, 6);
  expect([today.facts.tithi, today.facts.nakshatra]).toEqual([day.tithis[0].value.no, day.nakshatras[0].value.no]);
  expect(today.weekday).toBe(day.vara);
});

test("the four months cover the default filter, the full filter, the next month and a polar month", () => {
  const c = calendarFixture();
  const kinds = (view: typeof c.month) => new Set(view.days.flatMap((d) => d.events.map((e) => e.event.kind)));
  expect([...kinds(c.month)].every((k) => c.ids.default_kinds.includes(k))).toBe(true);
  expect(kinds(c.month_full).has("TransitToNatal")).toBe(true);
  expect([c.month.natal, c.month_full.natal]).toEqual([false, true]);
  expect(c.request_full.filter).toEqual({ kinds: c.ids.kinds, moon: true });
  expect(c.request_full.natal).not.toBeNull();
  expect(c.month_next.month).toEqual(c.month.next);
  expect(c.month_next.today).toBeNull();
  const risen = c.month_polar.days.filter((d) => d.facts.sunrise_jd_ut !== null).map((d) => d.day);
  expect(risen).toEqual([19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
  expect(c.ids).toMatchObject({ max_events: 300, min_year: 1, max_year: 2999 });
  expect(c.ids.kinds).toHaveLength(7);
});

test("calendar fixture copies are independent", () => {
  const a = calendarFixture();
  a.month.days[0].events.push(a.month.days[8].events[0]);
  expect(calendarFixture().month.days[0].events).toHaveLength(0);
});
