import { formatDegree } from "./format";

test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
  expect(formatDegree(0)).toBe("0°00′");
  expect(formatDegree(16.5)).toBe("16°30′");
  expect(formatDegree(10.25)).toBe("10°15′");
  expect(formatDegree(29.9999)).toBe("29°59′");
  expect(formatDegree(3.0166666666666666)).toBe("3°01′");
});
