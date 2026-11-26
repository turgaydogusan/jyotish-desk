import { expect, test } from "vitest";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { chartInput, inputKey } from "./input";

test("module input is the computed chart's moment, place and settings", () => {
  const opened = fixture().opened;
  const input = chartInput(opened);
  expect(input).toEqual({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  // Hesapta kullanılan ayarlar: haritanın `meta.settings` kopyası, `advanced` bloğuyla.
  expect(input.settings.schema_version).toBe(2);
  expect(input.settings.advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
});

test("an unknown birth time is passed on so that Rust hides lagna-dependent output", () => {
  expect(chartInput(unknownTimeOpened()).time_known).toBe(false);
});

test("the input key changes with the chart, the birth time certainty and the settings", () => {
  const base = fixture().opened;
  const key = inputKey(base);
  expect(inputKey(fixture().opened)).toBe(key);
  const other = fixture().opened;
  other.chart_record.id += 1;
  expect(inputKey(other)).not.toBe(key);
  expect(inputKey(unknownTimeOpened())).not.toBe(key);
  const raman = fixture().opened;
  raman.chart.meta.settings = { ...raman.chart.meta.settings, ayanamsa: "Raman" };
  expect(inputKey(raman)).not.toBe(key);
  const moved = fixture().opened;
  moved.chart.jd_ut = 2451545.5;
  expect(inputKey(moved)).not.toBe(key);
});
