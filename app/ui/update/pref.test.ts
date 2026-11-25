import { vi } from "vitest";
import { loadAutoCheck, saveAutoCheck, UPDATE_STORAGE_KEY } from "./pref";

test("the update check is off until the user turns it on", () => {
  expect(loadAutoCheck()).toBe(false);
  expect(saveAutoCheck(true)).toBe(true);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("auto");
  expect(loadAutoCheck()).toBe(true);
  expect(saveAutoCheck(false)).toBe(true);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("off");
  expect(loadAutoCheck()).toBe(false);
});

test("an unknown stored value and a storage that throws both mean off", () => {
  for (const v of ["true", "1", "AUTO", ""]) {
    localStorage.setItem(UPDATE_STORAGE_KEY, v);
    expect(loadAutoCheck(), v).toBe(false);
  }
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadAutoCheck(broken)).toBe(false);
  expect(saveAutoCheck(true, broken)).toBe(false);
});

test("a localStorage getter that throws fails closed", () => {
  const spy = vi.spyOn(window, "localStorage", "get").mockImplementation(() => {
    throw new Error("denied");
  });
  try {
    expect(loadAutoCheck()).toBe(false);
    expect(saveAutoCheck(true)).toBe(false);
  } finally {
    spy.mockRestore();
  }
});
