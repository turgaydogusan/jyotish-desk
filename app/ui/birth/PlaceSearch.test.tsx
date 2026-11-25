import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { searchPlaces: vi.fn() } }));

import { api } from "../api";
import type { PlaceHit } from "../bindings";
import { fixture } from "../test/fixture";
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
afterEach(() => vi.useRealTimers());

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

test("Escape closes the list without bubbling; mouse selection works", async () => {
  const onSelect = vi.fn();
  const onOuterKey = vi.fn();
  render(
    <div onKeyDown={onOuterKey}>
      <Harness onSelect={onSelect} />
    </div>,
  );
  await type("new delhi");
  fireEvent.keyDown(box(), { key: "Escape" });
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(onOuterKey).not.toHaveBeenCalled();
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.mouseDown(screen.getAllByRole("option")[0]);
  expect(onSelect).toHaveBeenCalledTimes(1);
});

test("an unavailable atlas is explained and leaves the field usable", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "error", error: { code: "atlas_unavailable", params: {} } });
  render(<Harness onSelect={() => {}} />);
  await type("delhi");
  expect(screen.getByRole("status")).toHaveTextContent("Yer atlası açılamadı; koordinatları elle girin.");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(box()).toHaveValue("delhi");
});
