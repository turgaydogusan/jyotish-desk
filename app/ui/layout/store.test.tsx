import { act, fireEvent, render, screen } from "@testing-library/react";
import { BUILTIN_LAYOUT_IDS, serializeLayoutState, type LayoutState } from "./model";
import { LAYOUT_STORAGE_KEY, localStoragePersistence, memoryPersistence, type LayoutPersistence } from "./persistence";
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
  fireEvent.keyDown(window, { key: "3", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true, shiftKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1" });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
});

test("Cmd+digit uses the physical key (AZERTY) and ignores out-of-range, Ctrl, Alt, 0, repeat and IME", () => {
  render(<LayoutProvider persistence={memoryPersistence()}><Probe /></LayoutProvider>);
  const before = BUILTIN_LAYOUT_IDS.research;
  fireEvent.click(screen.getByText("research"));
  // AZERTY: the "&" key is Digit1, so `key` is not a digit.
  fireEvent.keyDown(window, { key: "&", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.click(screen.getByText("research"));
  const ignored = [
    { key: "1", code: "Digit1", ctrlKey: true },
    { key: "1", code: "Digit1", altKey: true },
    { key: "0", code: "Digit0", metaKey: true },
    { key: "3", code: "Digit3", metaKey: true, repeat: true },
    { key: "3", code: "Digit3", metaKey: true, isComposing: true },
  ];
  for (const init of ignored) {
    fireEvent.keyDown(window, init);
    expect(screen.getByTestId("active")).toHaveTextContent(before);
  }
});

test("Cmd+9 with 3 layouts keeps the selection and does not preventDefault", () => {
  render(<LayoutProvider persistence={memoryPersistence()}><Probe /></LayoutProvider>);
  fireEvent.click(screen.getByText("research"));
  const missing = new KeyboardEvent("keydown", { key: "9", code: "Digit9", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(missing));
  expect(missing.defaultPrevented).toBe(false);
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  const present = new KeyboardEvent("keydown", { key: "1", code: "Digit1", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(present));
  expect(present.defaultPrevented).toBe(true);
});

test("a fresh persistence object each parent render does not defeat the debounce", () => {
  const saved: LayoutState[] = [];
  const mk = (): LayoutPersistence => ({ load: () => null, save: (s) => void saved.push(s) });
  function Parent({ tick }: { tick: number }) {
    return <LayoutProvider persistence={mk()}><span>{tick}</span><Probe /></LayoutProvider>;
  }
  const { rerender } = render(<Parent tick={0} />);
  fireEvent.click(screen.getByText("research"));
  rerender(<Parent tick={1} />);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  expect(saved).toHaveLength(0);
  rerender(<Parent tick={2} />);
  expect(saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(saved).toHaveLength(1);
});

test("unmount flushes exactly once; later timers, pagehide and keydown do nothing", () => {
  const p = memoryPersistence();
  const { unmount } = render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  fireEvent.click(screen.getByText("research"));
  unmount();
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  fireEvent(window, new Event("pagehide"));
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(p.saved).toHaveLength(1);
});

test("a pending save is flushed when the document becomes hidden", () => {
  const p = memoryPersistence();
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  fireEvent.click(screen.getByText("research"));
  const spy = vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
  fireEvent(document, new Event("visibilitychange"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
  spy.mockRestore();
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
