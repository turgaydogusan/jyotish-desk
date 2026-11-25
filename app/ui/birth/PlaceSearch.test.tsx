import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { onTestFinished, vi } from "vitest";

vi.mock("../api", () => ({ api: { searchPlaces: vi.fn() } }));

import { api } from "../api";
import type { PlaceHit } from "../bindings";
import { fixture } from "../test/fixture";
import birthCss from "./birth.css?raw";
import { PlaceSearch, SEARCH_DEBOUNCE_MS, SEARCH_LIMIT } from "./PlaceSearch";

function Harness({ onSelect }: { onSelect(h: PlaceHit): void }) {
  const [value, setValue] = useState("");
  return <PlaceSearch value={value} onChange={setValue} onSelect={(h) => (setValue(h.name), onSelect(h))} />;
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.searchPlaces).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

const box = () => screen.getByRole("combobox", { name: "Doğum yeri" });
async function type(text: string) {
  fireEvent.change(box(), { target: { value: text } });
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
}

test("debounced search opens a listbox; arrows move the active option; Enter selects", async () => {
  const onSelect = vi.fn();
  render(<Harness onSelect={onSelect} />);
  fireEvent.change(box(), { target: { value: "ne" } });
  fireEvent.change(box(), { target: { value: "new delhi" } });
  expect(api.searchPlaces).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
  expect(api.searchPlaces).toHaveBeenCalledTimes(1);
  expect(api.searchPlaces).toHaveBeenCalledWith("new delhi", SEARCH_LIMIT);
  expect(box()).toHaveAttribute("aria-expanded", "true");
  const options = screen.getAllByRole("option");
  expect(options.length).toBe(fixture().places.length);
  expect(options[0]).toHaveTextContent("New Delhi");
  expect(options[0]).toHaveTextContent("Asia/Kolkata");
  expect(box()).toHaveAttribute("aria-activedescendant", options[0].id);
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(box()).toHaveAttribute("aria-activedescendant", options[options.length - 1].id);
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onSelect).toHaveBeenCalledWith(fixture().places[0]);
  expect(box()).toHaveValue("New Delhi");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  // Seçim yeni bir arama başlatmaz.
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS * 2));
  expect(api.searchPlaces).toHaveBeenCalledTimes(1);
});

test("a stale response is ignored", async () => {
  let first!: (v: Awaited<ReturnType<typeof api.searchPlaces>>) => void;
  vi.mocked(api.searchPlaces).mockReturnValueOnce(new Promise((r) => (first = r)));
  render(<Harness onSelect={() => {}} />);
  await type("lon");
  await type("new delhi");
  await act(async () => first({ status: "ok", data: [{ ...fixture().places[0], name: "London", geoname_id: 1 }] }));
  expect(screen.queryByText("London")).toBeNull();
  expect(screen.getAllByRole("option")[0]).toHaveTextContent("New Delhi");
});

// Fikstürde tek yer var; gezinme testleri için üç sonuç.
const threeHits = (): PlaceHit[] => ["Delhi", "Delhi Cantonment", "New Delhi"].map((name, i) => ({ ...fixture().places[0], name, geoname_id: i + 1 }));

test("the active option is scrolled into view as it changes", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: threeHits() });
  const scroll = vi.spyOn(Element.prototype, "scrollIntoView");
  render(<Harness onSelect={() => {}} />);
  await type("delhi");
  const options = screen.getAllByRole("option");
  expect(options.length).toBe(3);
  expect(scroll.mock.contexts.at(-1)).toBe(options[0]);
  expect(scroll).toHaveBeenLastCalledWith({ block: "nearest" });
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  expect(scroll.mock.contexts.at(-1)).toBe(options[1]);
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(scroll.mock.contexts.at(-1)).toBe(options[options.length - 1]);
});

test("the active option is marked by an outline, not by colour alone", () => {
  const rule = /\.place-results li\.is-active \{([^}]*)\}/.exec(birthCss)?.[1] ?? "";
  expect(rule).toMatch(/outline:\s*2px solid var\(--ink\)/);
});

// Pencere (Dialog) Esc'i belge düzeyinde dinler: liste Esc'i ancak görünürken tüketir.
function documentKeys() {
  const onKey = vi.fn();
  document.addEventListener("keydown", onKey);
  onTestFinished(() => document.removeEventListener("keydown", onKey));
  return onKey;
}

test("Escape closes a visible list without reaching the document, then passes through; mouse selection works", async () => {
  const onSelect = vi.fn();
  const onDocumentKey = documentKeys();
  render(<Harness onSelect={onSelect} />);
  await type("new delhi");
  expect(fireEvent.keyDown(box(), { key: "Escape" })).toBe(false);
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(onDocumentKey).not.toHaveBeenCalled();
  // Liste kapalı: ikinci Esc pencereye ulaşır.
  expect(fireEvent.keyDown(box(), { key: "Escape" })).toBe(true);
  expect(onDocumentKey).toHaveBeenCalledTimes(1);
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.mouseDown(screen.getAllByRole("option")[0]);
  expect(onSelect).toHaveBeenCalledTimes(1);
});

test("with no results the first Escape and Enter are not consumed", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: [] });
  const onSelect = vi.fn();
  const onDocumentKey = documentKeys();
  render(<Harness onSelect={onSelect} />);
  await type("zzzz");
  expect(screen.getByRole("status")).toHaveTextContent("0 sonuç");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(fireEvent.keyDown(box(), { key: "Enter" })).toBe(true);
  expect(fireEvent.keyDown(box(), { key: "Escape" })).toBe(true);
  expect(onDocumentKey.mock.calls.map(([e]) => (e as KeyboardEvent).key)).toEqual(["Enter", "Escape"]);
  expect(onSelect).not.toHaveBeenCalled();
});

test("keys pressed during IME composition neither move nor select nor close", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: threeHits() });
  const onSelect = vi.fn();
  render(<Harness onSelect={onSelect} />);
  await type("delhi");
  const first = screen.getAllByRole("option")[0].id;
  for (const composing of [{ isComposing: true }, { keyCode: 229 }]) {
    fireEvent.keyDown(box(), { key: "ArrowDown", ...composing });
    fireEvent.keyDown(box(), { key: "Enter", ...composing });
    fireEvent.keyDown(box(), { key: "Escape", ...composing });
  }
  expect(onSelect).not.toHaveBeenCalled();
  expect(box()).toHaveAttribute("aria-expanded", "true");
  expect(box()).toHaveAttribute("aria-activedescendant", first);
});

test("an unavailable atlas is explained and leaves the field usable", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "error", error: { code: "atlas_unavailable", params: { reason: "missing" } } });
  render(<Harness onSelect={() => {}} />);
  await type("delhi");
  expect(screen.getByRole("status")).toHaveTextContent("Yer araması kullanılamıyor (atlas dosyası yok). Enlem, boylam ve saat dilimini elle girebilirsiniz.");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(box()).toHaveValue("delhi");
  // Alan boşalınca eski aramanın hatası kalkar.
  await type("");
  expect(screen.getByRole("status")).toBeEmptyDOMElement();
});
