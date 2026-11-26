import { formatDegree, formatJdMinute, formatSignedDegree, formatSpeed } from "./format";

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
