import { unixMsToJdUt } from "./time";

test("unixMsToJdUt maps the Unix epoch to JD 2440587.5", () => {
  expect(unixMsToJdUt(0)).toBe(2440587.5);
  expect(unixMsToJdUt(86_400_000)).toBe(2440588.5);
});
