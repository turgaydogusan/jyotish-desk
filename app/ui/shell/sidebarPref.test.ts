import { vi } from "vitest";
import { loadSidebarOpen, saveSidebarOpen, SIDEBAR_STORAGE_KEY } from "./sidebarPref";

test("the sidebar is open unless it was stored closed", () => {
  expect(loadSidebarOpen()).toBe(true);
  saveSidebarOpen(false);
  expect(localStorage.getItem(SIDEBAR_STORAGE_KEY)).toBe("closed");
  expect(loadSidebarOpen()).toBe(false);
  saveSidebarOpen(true);
  expect(loadSidebarOpen()).toBe(true);
  localStorage.setItem(SIDEBAR_STORAGE_KEY, "sideways");
  expect(loadSidebarOpen()).toBe(true);
});

test("a storage that throws leaves the sidebar open and does not break saving", () => {
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadSidebarOpen(broken)).toBe(true);
  expect(() => saveSidebarOpen(false, broken)).not.toThrow();
});

test("a localStorage getter that throws leaves the sidebar open and does not break saving", () => {
  const spy = vi.spyOn(window, "localStorage", "get").mockImplementation(() => {
    throw new Error("denied");
  });
  try {
    expect(loadSidebarOpen()).toBe(true);
    expect(() => saveSidebarOpen(false)).not.toThrow();
  } finally {
    spy.mockRestore();
  }
});
