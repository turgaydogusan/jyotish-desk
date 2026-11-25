import { act, fireEvent, render, screen } from "@testing-library/react";
import { useLayoutEffect } from "react";
import type { Graha } from "../bindings";
import { BUILTIN_LAYOUT_IDS } from "../layout/model";
import { initialViewModel } from "./model";
import { memoryPersistence, type ViewPersistence, type ViewSnapshot } from "./persistence";
import { SAVE_DEBOUNCE_MS, useLayoutStore, useViewStore, ViewStateProvider } from "./store";

function Probe() {
  const store = useLayoutStore();
  const view = useViewStore();
  return (
    <div>
      <span data-testid="active">{store.active.id}</span>
      <span data-testid="count">{store.state.layouts.length}</span>
      <span data-testid="chart">{String(view.model.chartId)}</span>
      <span data-testid="path">{view.model.dashaPath.join(">")}</span>
      <button onClick={() => store.saveAs("Benim")}>saveAs</button>
      <button onClick={() => store.select(BUILTIN_LAYOUT_IDS.research)}>research</button>
      <button onClick={() => view.setDashaPath(["Rahu", "Jupiter"] as Graha[])}>path</button>
      <button onClick={() => view.openChart(7, ["Venus"])}>open7</button>
      <button onClick={() => view.openChart(8, "garbage")}>open8</button>
      <button
        onClick={() => {
          store.select(BUILTIN_LAYOUT_IDS.research);
          view.openChart(8, []);
        }}
      >
        researchThenOpen8
      </button>
    </div>
  );
}

function setup(persistence: ViewPersistence = memoryPersistence(), layout: unknown = null) {
  return render(
    <ViewStateProvider initial={initialViewModel(layout)} persistence={persistence}>
      <Probe />
      <input aria-label="field" />
    </ViewStateProvider>,
  );
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("loads the stored layout through the tolerant parser", () => {
  setup(memoryPersistence(), { active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  expect(screen.getByTestId("count")).toHaveTextContent("3");
  expect(screen.getByTestId("chart")).toHaveTextContent("null");
});

test("does not save on mount; saves the whole ViewState once 500 ms after the last change", () => {
  const p = memoryPersistence();
  setup(p);
  act(() => vi.advanceTimersByTime(2000));
  expect(p.saved).toHaveLength(0);
  fireEvent.click(screen.getByText("research"));
  act(() => vi.advanceTimersByTime(300));
  fireEvent.click(screen.getByText("saveAs"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  expect(p.saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(p.saved).toHaveLength(1);
  const [s] = p.saved;
  expect(s.chartId).toBeNull();
  expect(s.viewState.schema_version).toBe(1);
  expect(s.viewState.panel_layout!.layouts).toHaveLength(4);
  expect(s.viewState.panel_layout!.active_layout_id).toMatch(/^user:/);
  expect(s.viewState.dasha_expanded_path).toEqual([]);
});

test("the dasha path is part of the saved ViewState of the open chart", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("open7"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(0); // açılan haritanın kendi durumu yeniden yazılmaz
  expect(screen.getByTestId("path")).toHaveTextContent("Venus");
  fireEvent.click(screen.getByText("path"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toEqual<ViewSnapshot[]>([
    { chartId: 7, viewState: expect.objectContaining({ dasha_expanded_path: ["Rahu", "Jupiter"] }) },
  ]);
});

test("opening another chart flushes the pending save to the previous chart first", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("open7"));
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("open8"));
  expect(p.saved.map((s) => [s.chartId, s.viewState.dasha_expanded_path])).toEqual([[7, ["Rahu", "Jupiter"]]]);
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(screen.getByTestId("path")).toHaveTextContent(""); // bozuk yol boş yol olur
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a change dispatched in the same tick as openChart is kept and saved to the previous chart", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("researchThenOpen8"));
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(p.saved.map((s) => [s.chartId, s.viewState.panel_layout!.active_layout_id])).toEqual([
    [null, BUILTIN_LAYOUT_IDS.research],
  ]);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a committed change whose effects have not run yet is still flushed to the previous chart", () => {
  // Layout effect'ler passive effect'lerden önce çalışır: değişiklik commit edilmiş ama
  // deponun kendi effect'i henüz çalışmamışken `openChart` çağrılır.
  function OpenOnResearch() {
    const { active } = useLayoutStore();
    const { model, openChart } = useViewStore();
    useLayoutEffect(() => {
      if (active.id === BUILTIN_LAYOUT_IDS.research && model.chartId === 7) openChart(8, []);
    }, [active.id, model.chartId, openChart]);
    return null;
  }
  const p = memoryPersistence();
  render(
    <ViewStateProvider initial={initialViewModel(null)} persistence={p}>
      <Probe />
      <OpenOnResearch />
    </ViewStateProvider>,
  );
  fireEvent.click(screen.getByText("open7"));
  fireEvent.click(screen.getByText("research"));
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  expect(p.saved.map((s) => [s.chartId, s.viewState.panel_layout!.active_layout_id])).toEqual([
    [7, BUILTIN_LAYOUT_IDS.research],
  ]);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a pending save is flushed on pagehide", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("research"));
  fireEvent(window, new Event("pagehide"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
});

test("a pending save is flushed when the document becomes hidden", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("research"));
  const spy = vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
  fireEvent(document, new Event("visibilitychange"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
  spy.mockRestore();
});

test("unmount flushes exactly once; later timers, pagehide and keydown do nothing", () => {
  const p = memoryPersistence();
  const { unmount } = setup(p);
  fireEvent.click(screen.getByText("research"));
  unmount();
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  fireEvent(window, new Event("pagehide"));
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(p.saved).toHaveLength(1);
});

test("a fresh persistence object each parent render does not defeat the debounce", () => {
  const saved: ViewSnapshot[] = [];
  const mk = (): ViewPersistence => ({ save: (s) => void saved.push(s) });
  function Parent({ tick }: { tick: number }) {
    return (
      <ViewStateProvider initial={initialViewModel(null)} persistence={mk()}>
        <span>{tick}</span>
        <Probe />
      </ViewStateProvider>
    );
  }
  const { rerender } = render(<Parent tick={0} />);
  fireEvent.click(screen.getByText("research"));
  rerender(<Parent tick={1} />);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  rerender(<Parent tick={2} />);
  expect(saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(saved).toHaveLength(1);
});

test("Cmd+1…9 selects layouts by order", () => {
  setup();
  fireEvent.keyDown(window, { key: "3", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true, shiftKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1" });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
});

test("Cmd+digit uses the physical key (AZERTY), works from a text field and ignores out-of-range, Ctrl, Alt, 0, repeat and IME", () => {
  setup();
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
    expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  }
  // ⌘ kısayolları yazı alanında da çalışır (Plan 10 incelemesi MF1).
  fireEvent.keyDown(screen.getByLabelText("field"), { key: "3", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
});

test("Cmd+9 with 3 layouts keeps the selection and does not preventDefault", () => {
  setup();
  fireEvent.click(screen.getByText("research"));
  const missing = new KeyboardEvent("keydown", { key: "9", code: "Digit9", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(missing));
  expect(missing.defaultPrevented).toBe(false);
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  const present = new KeyboardEvent("keydown", { key: "1", code: "Digit1", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(present));
  expect(present.defaultPrevented).toBe(true);
});

test("hooks outside the provider throw", () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  expect(() => render(<Probe />)).toThrow(/ViewStateProvider/);
});
