import { act, fireEvent, render, screen } from "@testing-library/react";
import { BUILTIN_LAYOUT_IDS, serializeLayoutState, type LayoutState } from "./model";
import { LAYOUT_STORAGE_KEY, localStoragePersistence, memoryPersistence } from "./persistence";
import { LayoutProvider, SAVE_DEBOUNCE_MS, useLayoutStore } from "./store";

function Probe() {
  const store = useLayoutStore();
  return (
    <div>
      <span data-testid="active">{store.active.id}</span>
      <span data-testid="count">{store.state.layouts.length}</span>
      <button onClick={() => store.saveAs("Benim")}>saveAs</button>
      <button onClick={() => store.select(BUILTIN_LAYOUT_IDS.research)}>research</button>
    </div>
  );
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("loads persisted state through the tolerant parser", () => {
  const p = memoryPersistence({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] });
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  expect(screen.getByTestId("count")).toHaveTextContent("3");
});

test("does not save on mount; saves once 500 ms after the last change", () => {
  const p = memoryPersistence();
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  act(() => vi.advanceTimersByTime(2000));
  expect(p.saved).toHaveLength(0);

  fireEvent.click(screen.getByText("research"));
  act(() => vi.advanceTimersByTime(300));
  fireEvent.click(screen.getByText("saveAs"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  expect(p.saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(p.saved).toHaveLength(1);
  expect(p.saved[0].layouts).toHaveLength(4);
  expect(p.saved[0].active_layout_id).toMatch(/^user:/);
});

test("a pending save is flushed on pagehide", () => {
  const p = memoryPersistence();
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  fireEvent.click(screen.getByText("research"));
  fireEvent(window, new Event("pagehide"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
});

test("Cmd+1…9 selects layouts by order", () => {
  render(<LayoutProvider persistence={memoryPersistence()}><Probe /></LayoutProvider>);
  fireEvent.keyDown(window, { key: "3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "2", metaKey: true, shiftKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1" });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
});

test("useLayoutStore outside the provider throws", () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  expect(() => render(<Probe />)).toThrow(/LayoutProvider/);
});

test("localStorage persistence round-trips and survives corrupt data", () => {
  vi.spyOn(console, "warn").mockImplementation(() => {});
  localStorage.clear();
  const p = localStoragePersistence();
  expect(p.load()).toBeNull();
  const state: LayoutState = { schema_version: 1, active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: [] };
  p.save(state);
  expect(localStorage.getItem(LAYOUT_STORAGE_KEY)).toBe(serializeLayoutState(state));
  expect(p.load()).toEqual(state);
  localStorage.setItem(LAYOUT_STORAGE_KEY, "{not json");
  expect(p.load()).toBeNull();
});
