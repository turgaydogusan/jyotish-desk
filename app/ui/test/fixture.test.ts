import { fixture, unknownTimeOpened } from "./fixture";

test("the fixture is a real Delhi chart with three dasha levels and atlas hits", () => {
  const f = fixture();
  expect(f.opened.chart.grahas).toHaveLength(9);
  expect(f.opened.chart.vargas).toHaveLength(16);
  expect(f.opened.chart.dasha.periods).toHaveLength(9 + 81 + 729);
  expect(f.opened.chart_record.resolved.jd_ut).toBe(2451545.0);
  expect(f.dasha_children.path).toHaveLength(3);
  expect(f.dasha_children.periods.every((p) => p.level === 4)).toBe(true);
  expect(f.places[0]).toMatchObject({ name: "New Delhi", timezone: "Asia/Kolkata" });
  expect(f.gap_error).toEqual({ code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:00", suggestion: "2014-03-31T04:30:00" } });
});

test("fixture copies are independent", () => {
  const a = fixture();
  a.opened.client.name = "x";
  expect(fixture().opened.client.name).toBe("Örnek Danışan");
  expect(unknownTimeOpened().chart_record.birth.time_certainty).toBe("Unknown");
});
