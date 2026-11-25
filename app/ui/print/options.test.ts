import { vi } from "vitest";
import { defaultPrintOptions, loadPrintOptions, parsePrintOptions, PRINT_STORAGE_KEY, savePrintOptions, togglePage } from "./options";

beforeEach(() => localStorage.clear());

test("defaults: all four pages in spec order, north style", () => {
  expect(defaultPrintOptions()).toEqual({ pages: ["cover", "charts", "dasha", "grahas"], style: "north" });
});

test("parsing keeps known pages in spec order without repeats and falls back field by field", () => {
  expect(parsePrintOptions({ pages: ["grahas", "cover", "grahas", "appendix", 3], style: "south" })).toEqual({ pages: ["cover", "grahas"], style: "south" });
  expect(parsePrintOptions({ pages: [], style: "hexagon" })).toEqual(defaultPrintOptions());
  expect(parsePrintOptions({ pages: "cover", style: "circle" })).toEqual({ ...defaultPrintOptions(), style: "circle" });
  for (const raw of [null, undefined, 7, "x", []]) expect(parsePrintOptions(raw)).toEqual(defaultPrintOptions());
});

test("toggling removes and re-adds a page in spec order; the last page stays", () => {
  expect(togglePage(["cover", "charts", "dasha", "grahas"], "charts")).toEqual(["cover", "dasha", "grahas"]);
  expect(togglePage(["cover", "grahas"], "dasha")).toEqual(["cover", "dasha", "grahas"]);
  expect(togglePage(["dasha"], "dasha")).toEqual(["dasha"]);
});

test("options round-trip through storage; unreadable storage gives the defaults", () => {
  savePrintOptions({ pages: ["charts"], style: "circle" });
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!)).toEqual({ pages: ["charts"], style: "circle" });
  expect(loadPrintOptions()).toEqual({ pages: ["charts"], style: "circle" });
  localStorage.setItem(PRINT_STORAGE_KEY, "{not json");
  expect(loadPrintOptions()).toEqual(defaultPrintOptions());
  localStorage.removeItem(PRINT_STORAGE_KEY);
  expect(loadPrintOptions()).toEqual(defaultPrintOptions());
});

test("a storage that throws neither breaks loading nor saving", () => {
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadPrintOptions(broken)).toEqual(defaultPrintOptions());
  expect(() => savePrintOptions(defaultPrintOptions(), broken)).not.toThrow();
});

test("a localStorage getter that throws gives the defaults and does not break saving", () => {
  const spy = vi.spyOn(window, "localStorage", "get").mockImplementation(() => {
    throw new Error("denied");
  });
  try {
    expect(loadPrintOptions()).toEqual(defaultPrintOptions());
    expect(() => savePrintOptions(defaultPrintOptions())).not.toThrow();
  } finally {
    spy.mockRestore();
  }
});
