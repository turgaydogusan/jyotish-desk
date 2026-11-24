import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [{ body: "Sun", lon: 256.5, retrograde: false }],
    })),
  },
}));

import { api } from "../api";
import { HighlightProvider } from "../highlight/store";
import { hasOverlap } from "../layout/grid";
import type { LayoutState, Panel } from "../layout/model";
import { memoryPersistence } from "../layout/persistence";
import { LayoutProvider } from "../layout/store";
import { SkyProvider } from "../panels/sky";
import { Dashboard } from "./Dashboard";
import { GRID_GAP_PX } from "./metrics";

// 12 sütun, sütun adımı 108 px (100 + 8 aralık), satır adımı 40 px.
const WIDTH = 1200 + 11 * GRID_GAP_PX;

function state(panels: Panel[]): LayoutState {
  return {
    schema_version: 1,
    active_layout_id: "user:t",
    layouts: [{ id: "user:t", name: "T", builtin: false, panels }],
  };
}

const PANELS: Panel[] = [
  { id: "a", kind: "graha_table", x: 0, y: 0, w: 4, h: 4, settings: {} },
  { id: "b", kind: "notes", x: 4, y: 0, w: 4, h: 4, settings: {} },
  { id: "c", kind: "yoga_list", x: 0, y: 4, w: 4, h: 4, settings: {} },
];

function setup(width = WIDTH, panels = PANELS) {
  const persistence = memoryPersistence(state(panels));
  const utils = render(
    <LayoutProvider persistence={persistence} debounceMs={0}>
      <HighlightProvider>
        <SkyProvider now={() => 0}>
          <Dashboard width={width} />
        </SkyProvider>
      </HighlightProvider>
    </LayoutProvider>,
  );
  const frame = (id: string) => utils.container.querySelector<HTMLElement>(`[data-panel-id="${id}"]`)!;
  const box = (id: string) => {
    const f = frame(id);
    return [f.dataset.x, f.dataset.y, f.dataset.w, f.dataset.h].map(Number);
  };
  const all = () =>
    [...utils.container.querySelectorAll<HTMLElement>("[data-panel-id]")].map((f) => ({
      x: Number(f.dataset.x),
      y: Number(f.dataset.y),
      w: Number(f.dataset.w),
      h: Number(f.dataset.h),
    }));
  return { ...utils, persistence, frame, box, all };
}

test("renders every panel of the active layout on a 12-column grid", async () => {
  const { container, frame } = setup();
  expect(container.querySelector(".dashboard")).toHaveAttribute("data-columns", "12");
  expect(frame("a").style.gridColumn).toBe("1 / span 4");
  expect(frame("b").style.gridRow).toBe("1 / span 4");
  expect(screen.getByRole("region", { name: "Graha tablosu" })).toBeInTheDocument();
  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
  expect(screen.getByRole("region", { name: "Notlar" })).toHaveTextContent("Bu panel sonraki bir sürümde gelecek.");
  expect(screen.getByRole("region", { name: "Bilinmeyen panel" })).toHaveTextContent("“yoga_list” paneli");
});

test("Alt+Arrow moves the focused panel; Alt+Shift+Arrow resizes; neighbours are pushed down", async () => {
  const { frame, box, all } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft", altKey: true });
  expect(box("b")).toEqual([3, 0, 4, 4]);
  expect(box("a")).toEqual([0, 4, 4, 4]);
  expect(box("c")).toEqual([0, 8, 4, 4]);
  fireEvent.keyDown(frame("b"), { key: "ArrowDown", altKey: true, shiftKey: true });
  expect(box("b")).toEqual([3, 0, 4, 5]);
  expect(hasOverlap(all())).toBe(false);
  // Alt olmadan ok tuşları ve panel içindeki öğeler düzeni değiştirmez.
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft" });
  fireEvent.keyDown(screen.getByRole("button", { name: "Güneş" }), { key: "ArrowRight", altKey: true });
  expect(box("b")).toEqual([3, 0, 4, 5]);
});

test("dragging the header moves the panel by whole cells and commits on pointer up", async () => {
  const { box, persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 10, clientY: 10, button: 0 });
  fireEvent.pointerMove(window, { clientX: 10 + 108 * 8 + 30, clientY: 10 + 40 * 2 - 10 });
  expect(box("a")).toEqual([8, 2, 4, 4]); // önizleme
  fireEvent.pointerUp(window);
  expect(box("a")).toEqual([8, 2, 4, 4]);
  await waitFor(() => expect(persistence.saved).toHaveLength(1));
  const saved = persistence.saved[0].layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels.find((p) => p.id === "a")).toMatchObject({ x: 8, y: 2 });
});

test("pointer cancel discards the drag", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: 108 * 5, clientY: 0 });
  fireEvent.pointerCancel(window);
  expect(box("a")).toEqual([0, 0, 4, 4]);
});

test("the bottom-right handle resizes and respects the minimum size", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: 108 * 2, clientY: 40 * 3 });
  fireEvent.pointerUp(window);
  expect(box("b")).toEqual([4, 0, 6, 7]);
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: -108 * 10, clientY: -40 * 10 });
  fireEvent.pointerUp(window);
  expect(box("b")).toEqual([4, 0, 3, 4]);
});

test("narrow windows reflow to 8 and 4 columns read-only", async () => {
  const { container, box, frame, all } = setup(700);
  await screen.findByRole("button", { name: "Güneş" });
  expect(container.querySelector(".dashboard")).toHaveAttribute("data-columns", "8");
  expect(container.querySelector('[data-testid^="panel-resize-"]')).toBeNull();
  expect(hasOverlap(all())).toBe(false);
  const before = box("b");
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft", altKey: true });
  expect(box("b")).toEqual(before);
  const narrow = setup(320);
  expect(narrow.container.querySelectorAll('.dashboard[data-columns="4"]')).toHaveLength(1);
});

test("the header shows an inline loading indicator between 100 ms and 1 s", async () => {
  vi.useFakeTimers();
  let resolve: (v: Awaited<ReturnType<typeof api.planetLongitudes>>) => void = () => {};
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  setup(WIDTH, [PANELS[0]]);
  expect(screen.queryByRole("status")).toBeNull();
  act(() => vi.advanceTimersByTime(150));
  expect(screen.getByRole("status")).toHaveTextContent("Yükleniyor");
  await act(async () => resolve({ status: "ok", data: [] }));
  expect(screen.queryByRole("status")).toBeNull();
  vi.useRealTimers();
});
