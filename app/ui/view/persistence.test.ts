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

test("with an open chart the whole ViewState goes to save_view_state, without one only the layout", async () => {
  vi.mocked(api.saveViewState).mockResolvedValue(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
  const p = storePersistence();
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: null, viewState: vs });
  expect(api.saveViewState).toHaveBeenCalledWith(4, vs);
  expect(api.savePanelLayout).toHaveBeenCalledWith(vs.panel_layout);
});

test("save errors reach onError", async () => {
  const error = { code: "store_sqlite", params: { detail: "disk full" } };
  vi.mocked(api.savePanelLayout).mockResolvedValue({ status: "error", error });
  const onError = vi.fn();
  storePersistence(onError).save({ chartId: null, viewState: vs });
  await vi.waitFor(() => expect(onError).toHaveBeenCalledWith(error));
});

test("loadLayout prefers the store and falls back to defaults on error", async () => {
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(vs.panel_layout));
  expect(await loadLayout()).toEqual(vs.panel_layout);
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  expect(await loadLayout()).toBeNull();
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect(await loadLayout()).toBeNull();
  expect(api.savePanelLayout).not.toHaveBeenCalled();
});

test("a Plan 04 layout in localStorage moves to the store once", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce(ok(null));
  const loaded = (await loadLayout()) as { active_layout_id: string };
  expect(loaded.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.transit);
  expect(vi.mocked(api.savePanelLayout).mock.calls[0][0]).toMatchObject({ active_layout_id: BUILTIN_LAYOUT_IDS.transit });
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).toBeNull();
});

test("a corrupt legacy entry is ignored and a failed move keeps it for the next start", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, "{not json");
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect(await loadLayout()).toBeNull();
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: {} } });
  expect(await loadLayout()).not.toBeNull();
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).not.toBeNull();
});
