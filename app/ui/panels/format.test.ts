import { formatDegree, formatJdClock, formatJdDate, formatJdMinute, formatJdSpan, formatSignedDegree, formatSpeed } from "./format";

test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
  expect(formatDegree(0)).toBe("0°00′");
  expect(formatDegree(16.5)).toBe("16°30′");
  expect(formatDegree(10.25)).toBe("10°15′");
  expect(formatDegree(29.9999)).toBe("29°59′");
  expect(formatDegree(3.0166666666666666)).toBe("3°01′");
});

test("signed degrees and speeds carry an explicit sign and a true minus", () => {
  expect(formatSignedDegree(5.170740538571619)).toBe("+5°10′");
  expect(formatSignedDegree(-0.5)).toBe("−0°30′");
  expect(formatSpeed(12.021263924093429)).toBe("+12.021");
  expect(formatSpeed(-0.05)).toBe("−0.050");
});

test("a julian day is written as local date and minute at a fixed offset, floored", () => {
  // JD 2451545.0 = 2000-01-01 12:00 UT.
  expect(formatJdMinute(2451545, 0)).toBe("2000-01-01 12:00");
  expect(formatJdMinute(2451545, 19800)).toBe("2000-01-01 17:30");
  expect(formatJdMinute(2451545, -18000)).toBe("2000-01-01 07:00");
  // 59,9 saniye aşağı yuvarlanır; gün sınırı ofsetle geçilir.
  expect(formatJdMinute(2451545 + 59.9 / 86400, 0)).toBe("2000-01-01 12:00");
  expect(formatJdMinute(2451545 + 60 / 86400 + 1e-9, 0)).toBe("2000-01-01 12:01");
  // 0,40001 gün = 9 sa 36 dk 0,9 sn → 21:36 UT → +05:30 ertesi gün 03:06.
  expect(formatJdMinute(2451545.40001, 19800)).toBe("2000-01-02 03:06");
  expect(formatJdMinute(null, 0)).toBe("—");
  expect(formatJdMinute(Number.NaN, 0)).toBe("—");
  expect(formatJdMinute(1e300, 0)).toBe("—");
});

test("a clock time carries a day offset when it falls on another local day than the base", () => {
  // JD 2451545.0 = 2000-01-01 12:00 UT; +05:30'da 17:30.
  expect(formatJdClock(2451545, 19800, 2451545)).toBe("17:30");
  expect(formatJdClock(2451545, 0, null)).toBe("12:00");
  // 9 saat sonra: 21:00 UT, +05:30'da ertesi gün 02:30.
  expect(formatJdClock(2451545.375, 19800, 2451545)).toBe("02:30 (+1)");
  expect(formatJdClock(2451545.375, 0, 2451545)).toBe("21:00");
  // Tabandan önceki gün: 2000-01-01 01:00:30 UT, −05:00'da önceki gün 20:00.
  expect(formatJdClock(2451544.5 + 3630 / 86400, -18000, 2451545)).toBe("20:00 (−1)");
  expect(formatJdClock(2451547.5, 0, 2451545)).toBe("00:00 (+3)");
  expect(formatJdClock(null, 0, 2451545)).toBe("—");
  expect(formatJdClock(Number.NaN, 0, 2451545)).toBe("—");
});

test("a julian day is written as a local date at a fixed offset", () => {
  expect(formatJdDate(2451545, 0)).toBe("2000-01-01");
  expect(formatJdDate(2451545.375, 19800)).toBe("2000-01-02");
  expect(formatJdDate(null, 0)).toBe("—");
});

test("a span is written in days, hours and minutes, rounding the minute down", () => {
  const units = { day: "g", hour: "sa", minute: "dk" };
  expect(formatJdSpan(10, 10 + 45 / 1440, units)).toBe("45 dk");
  expect(formatJdSpan(10, 10 + 125 / 1440, units)).toBe("2 sa 05 dk");
  expect(formatJdSpan(10, 10 + 125.9 / 1440, units)).toBe("2 sa 05 dk");
  expect(formatJdSpan(10, 13 + 4.5 / 24, units)).toBe("3 g 4 sa");
  expect(formatJdSpan(10, 10, units)).toBe("0 dk");
  expect(formatJdSpan(10, 9, units)).toBe("—");
  expect(formatJdSpan(null, 9, units)).toBe("—");
});
