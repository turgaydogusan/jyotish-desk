import { vi } from "vitest";

vi.mock("../api", () => ({
  api: { saveViewState: vi.fn(), savePanelLayout: vi.fn(), loadPanelLayout: vi.fn() },
}));

import { api } from "../api";
import { BUILTIN_LAYOUT_IDS } from "../layout/model";
import { LEGACY_LAYOUT_KEY, loadLayout, storePersistence } from "./persistence";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const vs = { schema_version: 1, panel_layout: { active_layout_id: "a", layouts: [] }, dasha_expanded_path: [] };

afterEach(() => vi.mocked(api.savePanelLayout).mockReset());

afterEach(() => vi.mocked(api.saveViewState).mockReset());

/** Elle çözülen yanıt: kaydın ne zaman bittiğini test belirler. */
function deferred() {
  let resolve!: (v: { status: "ok"; data: null }) => void;
  const promise = new Promise<{ status: "ok"; data: null }>((r) => (resolve = r));
  return { promise, resolve: () => resolve(ok(null)) };
}

test("with an open chart the whole ViewState goes to save_view_state, without one only the layout", async () => {
  vi.mocked(api.saveViewState).mockResolvedValue(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
  const p = storePersistence();
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: null, viewState: vs });
  await p.idle();
  expect(api.saveViewState).toHaveBeenCalledWith(4, vs);
  expect(api.savePanelLayout).toHaveBeenCalledWith(vs.panel_layout);
});

test("saves are sent one at a time in order and a waiting save to the same target keeps only the latest", async () => {
  const first = deferred();
  vi.mocked(api.saveViewState).mockReturnValueOnce(first.promise).mockResolvedValue(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
  const p = storePersistence();
  const at = (path: string[]) => ({ ...vs, dasha_expanded_path: path as never });
  p.save({ chartId: 1, viewState: at(["Sun"]) });
  p.save({ chartId: 2, viewState: at(["Moon"]) });
  p.save({ chartId: 2, viewState: at(["Mars"]) });
  p.save({ chartId: null, viewState: vs });
  // İlk kayıt bitmeden ikincisi gönderilmez.
  expect(api.saveViewState).toHaveBeenCalledTimes(1);
  expect(api.savePanelLayout).not.toHaveBeenCalled();
  first.resolve();
  await p.idle();
  expect(vi.mocked(api.saveViewState).mock.calls).toEqual([
    [1, at(["Sun"])],
    [2, at(["Mars"])],
  ]);
  expect(api.savePanelLayout).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.saveViewState).mock.invocationCallOrder[1]).toBeLessThan(vi.mocked(api.savePanelLayout).mock.invocationCallOrder[0]);
});

test("save errors reach onError and a later success reaches onSaved", async () => {
  const error = { code: "store_sqlite", params: { detail: "disk full" } };
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error }).mockResolvedValueOnce(ok(null));
  const onError = vi.fn();
  const onSaved = vi.fn();
  const p = storePersistence({ onError, onSaved });
  p.save({ chartId: null, viewState: vs });
  await p.idle();
  expect(onError).toHaveBeenCalledWith(error);
  expect(onSaved).not.toHaveBeenCalled();
  p.save({ chartId: null, viewState: vs });
  await p.idle();
  expect(onSaved).toHaveBeenCalledOnce();
});

test("held persistence sends nothing (the stored layout could not be read)", async () => {
  const p = storePersistence({}, { hold: true });
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: null, viewState: vs });
  await p.idle();
  expect(api.saveViewState).not.toHaveBeenCalled();
  expect(api.savePanelLayout).not.toHaveBeenCalled();
});

test("loadLayout prefers the store and reports a read error without a layout", async () => {
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(vs.panel_layout));
  expect(await loadLayout()).toEqual({ layout: vs.panel_layout, error: null });
  const error = { code: "store_unavailable", params: {} };
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce({ status: "error", error });
  expect(await loadLayout()).toEqual({ layout: null, error });
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect(await loadLayout()).toEqual({ layout: null, error: null });
  expect(api.savePanelLayout).not.toHaveBeenCalled();
});

test("a Plan 04 layout in localStorage moves to the store once", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce(ok(null));
  const loaded = (await loadLayout()).layout as { active_layout_id: string };
  expect(loaded.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.transit);
  expect(vi.mocked(api.savePanelLayout).mock.calls[0][0]).toMatchObject({ active_layout_id: BUILTIN_LAYOUT_IDS.transit });
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).toBeNull();
});

test("a corrupt legacy entry is ignored and a failed move keeps it for the next start", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, "{not json");
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect((await loadLayout()).layout).toBeNull();
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: {} } });
  expect((await loadLayout()).layout).not.toBeNull();
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).not.toBeNull();
});

test("a save that throws reaches onError as task_failed and the queue keeps going", async () => {
  vi.mocked(api.saveViewState).mockRejectedValueOnce(new Error("ipc")).mockResolvedValue(ok(null));
  const onError = vi.fn();
  const onSaved = vi.fn();
  const p = storePersistence({ onError, onSaved });
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: 5, viewState: vs });
  await p.idle();
  expect(onError).toHaveBeenCalledWith({ code: "task_failed", params: {} });
  expect(onSaved).toHaveBeenCalledOnce();
  p.save({ chartId: 4, viewState: vs });
  await p.idle();
  expect(api.saveViewState).toHaveBeenCalledTimes(3);
});
