import { formatDegree, formatSignedDegree, formatSpeed } from "./format";

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
