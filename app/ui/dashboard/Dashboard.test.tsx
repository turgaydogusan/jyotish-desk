import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("../api", () => ({ api: {
    dashaChildren: vi.fn(),
    transitsAt: vi.fn(async () => ({ status: "ok", data: (await import("../test/fixture")).fixture().transit })), listNotes: vi.fn(async () => ({ status: "ok", data: [] })),
  },
}));

import { api } from "../api";
import { fixture } from "../test/fixture";

import { hasOverlap } from "../layout/grid";
import type { LayoutState, Panel } from "../layout/model";
import { i18n } from "../i18n";
import { PANEL_REGISTRY } from "../panels/registry";
import { renderWithStores } from "../test/render";
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

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

const MOVE = { buttons: 1, pointerId: 1 };

function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(panels), opened = fixture().opened) {
  const utils = renderWithStores(<Dashboard width={width} />, { layout: initial, opened });
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
  return { ...utils, frame, box, all };
}

test("two dasha trees in one layout are regions with different names: the system is part of the name", () => {
  const tree = (id: string, x: number): Panel => ({ id, kind: "dasha_tree", x, y: 0, w: 4, h: 8, settings: { system: "vimshottari" } });
  const { frame } = setup(WIDTH, [tree("a", 0), { ...tree("b", 4), settings: {} }, { id: "c", kind: "chart", x: 8, y: 0, w: 4, h: 8, settings: { varga: "D9" } }]);
  expect(screen.getAllByRole("region", { name: "Dasa ağacı Vimshottari" })).toHaveLength(2);
  expect(frame("c")).toHaveAccessibleName("Harita D9");
  // Görünen başlık değişmez; ek yalnızca erişilebilir addadır.
  expect(within(frame("a")).getByText("Dasa ağacı")).toBeInTheDocument();
  expect(PANEL_REGISTRY.dasha_tree.nameDetail!({ ...tree("x", 0), settings: { system: "chara" } }, i18n.t)).toBe("Chara dasha");
});

test("renders every panel of the active layout on a 12-column grid", async () => {
  const { container, frame } = setup();
  expect(container.querySelector(".dashboard")).toHaveAttribute("data-columns", "12");
  expect(frame("a").style.gridColumn).toBe("1 / span 4");
  expect(frame("b").style.gridRow).toBe("1 / span 4");
  expect(screen.getByRole("region", { name: "Graha tablosu" })).toBeInTheDocument();
  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
  expect(await within(screen.getByRole("region", { name: "Notlar" })).findByRole("button", { name: "Yeni not" })).toBeInTheDocument();
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
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 10, clientY: 10, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 10 + 108 * 8 + 30, clientY: 10 + 40 * 2 - 10 });
  expect(box("a")).toEqual([8, 2, 4, 4]); // önizleme
  fireEvent.pointerUp(window, { pointerId: 1 });
  expect(box("a")).toEqual([8, 2, 4, 4]);
  await waitFor(() => expect(persistence.saved).toHaveLength(1));
  const saved = persistence.saved[0].viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels.find((p) => p.id === "a")).toMatchObject({ x: 8, y: 2 });
});

test("pointer cancel discards the drag", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 5, clientY: 0 });
  fireEvent.pointerCancel(window, { pointerId: 1 });
  expect(box("a")).toEqual([0, 0, 4, 4]);
});

test("the bottom-right handle resizes and respects the minimum size", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 2, clientY: 40 * 3 });
  fireEvent.pointerUp(window, { pointerId: 1 });
  expect(box("b")).toEqual([4, 0, 6, 7]);
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: -108 * 10, clientY: -40 * 10 });
  fireEvent.pointerUp(window, { pointerId: 1 });
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
  let resolve: (v: Awaited<ReturnType<typeof api.dashaChildren>>) => void = () => {};
  vi.mocked(api.dashaChildren).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  setup(WIDTH, [{ id: "d", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 8, settings: {} }], undefined, opened);
  expect(screen.queryByRole("status")).toBeNull();
  act(() => vi.advanceTimersByTime(150));
  expect(screen.getByRole("status")).toHaveTextContent("Yükleniyor");
  await act(async () => resolve({ status: "ok", data: fixture().dasha_children.periods }));
  expect(screen.queryByRole("status")).toBeNull();
});

// ---- Sürükleme sağlamlığı ----

function listenerBalance() {
  const names = ["pointermove", "pointerup", "pointercancel", "blur"];
  const add = vi.spyOn(window, "addEventListener");
  const remove = vi.spyOn(window, "removeEventListener");
  const count = (spy: typeof add) => spy.mock.calls.filter(([n]) => names.includes(n as string)).length;
  return { live: () => count(add) - count(remove) };
}

test("a move with no buttons pressed (released outside the window) cancels the drag", async () => {
  const { box, persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 3, clientY: 0 });
  expect(box("a")).toEqual([3, 0, 4, 4]);
  fireEvent.pointerMove(window, { buttons: 0, pointerId: 1, clientX: 108 * 5, clientY: 0 });
  expect(box("a")).toEqual([0, 0, 4, 4]);
  fireEvent.pointerUp(window, { pointerId: 1 });
  expect(box("a")).toEqual([0, 0, 4, 4]);
  await new Promise((r) => setTimeout(r, 20));
  expect(persistence.saved).toHaveLength(0);
});

test("window blur cancels the drag", async () => {
  const { box, persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 3, clientY: 0 });
  fireEvent.blur(window);
  expect(box("a")).toEqual([0, 0, 4, 4]);
  fireEvent.pointerUp(window, { pointerId: 1 });
  await new Promise((r) => setTimeout(r, 20));
  expect(persistence.saved).toHaveLength(0);
});

test("a second pointerdown does not leak the first drag's listeners", async () => {
  const balance = listenerBalance();
  const { box, persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  const header = getByTestId("panel-header-a");
  fireEvent.pointerDown(header, { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerDown(header, { clientX: 0, clientY: 0, button: 0, pointerId: 2 });
  // İlk işaretçinin olayları yok sayılır.
  fireEvent.pointerMove(window, { buttons: 1, pointerId: 1, clientX: 108 * 6, clientY: 0 });
  expect(box("a")).toEqual([0, 0, 4, 4]);
  fireEvent.pointerMove(window, { buttons: 1, pointerId: 2, clientX: 108 * 2, clientY: 0 });
  expect(box("a")).toEqual([2, 0, 4, 4]);
  fireEvent.pointerUp(window, { pointerId: 2 });
  expect(balance.live()).toBe(0);
  await waitFor(() => expect(persistence.saved).toHaveLength(1));
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 9, clientY: 0 });
  fireEvent.pointerUp(window, { pointerId: 1 });
  await new Promise((r) => setTimeout(r, 20));
  expect(persistence.saved).toHaveLength(1);
  expect(box("a")).toEqual([2, 0, 4, 4]);
});

test("unmounting mid-drag removes the window listeners", async () => {
  const balance = listenerBalance();
  const { unmount, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  expect(balance.live()).toBeGreaterThan(0);
  unmount();
  expect(balance.live()).toBe(0);
});

test("switching the active layout mid-drag cancels the drag and commits nothing", async () => {
  const two: LayoutState = {
    schema_version: 1,
    active_layout_id: "user:t",
    layouts: [
      { id: "user:t", name: "T", builtin: false, panels: PANELS },
      { id: "user:u", name: "U", builtin: false, panels: [PANELS[0]] },
    ],
  };
  const { persistence, getByTestId } = setup(WIDTH, PANELS, two);
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 3, clientY: 0 });
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  await waitFor(() => expect(persistence.saved).toHaveLength(1)); // yalnızca seçim kaydedildi
  fireEvent.pointerUp(window, { pointerId: 1 });
  await new Promise((r) => setTimeout(r, 20));
  // Yalnızca seçim değişti: hiçbir kayıtta düzenlerin içeriği ilk kayıttan farklı olmamalı.
  expect(persistence.saved.length).toBeGreaterThan(0);
  const first = JSON.stringify(persistence.saved[0].viewState.panel_layout!.layouts);
  for (const s of persistence.saved) expect(JSON.stringify(s.viewState.panel_layout!.layouts)).toBe(first);
});

test("a drag that ends where it started commits nothing", async () => {
  const { persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 108 * 3, clientY: 0 });
  fireEvent.pointerMove(window, { ...MOVE, clientX: 10, clientY: 5 });
  fireEvent.pointerUp(window, { pointerId: 1 });
  await new Promise((r) => setTimeout(r, 20));
  expect(persistence.saved).toHaveLength(0);
});

// ---- Erişilebilirlik ----

test("panel exposes full shortcut list and a visually hidden hint via aria-describedby", async () => {
  const { frame, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  const f = frame("a");
  const keys = f.getAttribute("aria-keyshortcuts")!;
  for (const k of ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]) {
    expect(keys).toContain(`Alt+${k}`);
    expect(keys).toContain(`Alt+Shift+${k}`);
  }
  expect(f).not.toHaveAttribute("title");
  const hint = document.getElementById(f.getAttribute("aria-describedby")!)!;
  expect(hint).toHaveTextContent("Taşımak için Alt+ok, boyutlandırmak için Alt+Shift+ok");
  expect(hint).toHaveClass("visually-hidden");
  expect(getByTestId("panel-header-a")).toHaveAttribute("title", "Taşımak için Alt+ok, boyutlandırmak için Alt+Shift+ok");
});

test("keyboard move and resize announce the new position in a polite live region", async () => {
  const { container, frame } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  const live = container.querySelector('[aria-live="polite"]')!;
  expect(live).toBeInTheDocument();
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft", altKey: true });
  expect(live).toHaveTextContent("Sütun 4, satır 1, 4×4");
  fireEvent.keyDown(frame("b"), { key: "ArrowDown", altKey: true, shiftKey: true });
  expect(live).toHaveTextContent("Sütun 4, satır 1, 4×5");
});

test("pressing a header focuses its panel", async () => {
  const { frame, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-b"), { clientX: 0, clientY: 0, button: 0, pointerId: 1 });
  expect(document.activeElement).toBe(frame("b"));
  fireEvent.pointerUp(window, { pointerId: 1 });
});
