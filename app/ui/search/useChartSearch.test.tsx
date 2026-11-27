import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { chartSearch: vi.fn(), chartSearchCancel: vi.fn() } }));

import { api, type Result } from "../api";
import type { ChartSearchPage } from "../bindings";
import { fixture } from "../test/fixture";
import { useChartSearch, type ChartSearch } from "./useChartSearch";

const FX = fixture();
const QUERY = FX.search.query;
const HIT = FX.search.page.hits[0];

/** Elle çözülen yanıt: sayfaların geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<ChartSearchPage>) => void;
  const promise = new Promise<Result<ChartSearchPage>>((r) => (resolve = r));
  return { promise, resolve };
}
/** `n`. sayfa: 500'er harita, sayfa başına bir eşleşme; `last` son sayfadır. */
function page(n: number, last: boolean, total = 1200): Result<ChartSearchPage> {
  const scanned = Math.min(n * 500, total);
  return {
    status: "ok",
    data: {
      search_id: 0,
      hits: [{ ...HIT, chart_id: n }],
      scanned,
      total,
      skipped_time_unknown: n,
      failed: 0,
      done: last,
      next: last ? null : { after_chart_id: scanned, scanned, skipped_time_unknown: n, failed: 0, total, now_jd_ut: 2461372 },
      method: "chart_search.v1",
    },
  };
}

let search: ChartSearch;
function Probe() {
  search = useChartSearch();
  return <span data-testid="state">{`${search.status} ${search.hits.map((h) => h.chart_id).join(",")} ${search.scanned}/${search.total ?? "?"} ${search.skippedTimeUnknown}`}</span>;
}
const state = () => screen.getByTestId("state").textContent;
const calls = () => vi.mocked(api.chartSearch).mock.calls;

beforeEach(() => {
  vi.mocked(api.chartSearch).mockReset();
  vi.mocked(api.chartSearchCancel).mockReset();
  vi.mocked(api.chartSearchCancel).mockResolvedValue({ status: "ok", data: null });
});

test("pages are asked one after another with the same id and the cursor of the page before", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.chartSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  expect(state()).toBe("idle  0/? 0");
  act(() => search.start(QUERY));
  // İlk yanıttan önce toplam bilinmez.
  expect(state()).toBe("running  0/? 0");
  expect(calls()).toHaveLength(1);
  expect(calls()[0][2]).toBeNull();
  const id = calls()[0][1];
  await act(async () => replies[0].resolve(page(1, false)));
  expect(state()).toBe("running 1 500/1200 1");
  expect(calls()).toHaveLength(2);
  expect(calls()[1]).toEqual([QUERY, id, { after_chart_id: 500, scanned: 500, skipped_time_unknown: 1, failed: 0, total: 1200, now_jd_ut: 2461372 }]);
  await act(async () => replies[1].resolve(page(2, false)));
  expect(state()).toBe("running 1,2 1000/1200 2");
  await act(async () => replies[2].resolve(page(3, true)));
  expect(state()).toBe("done 1,2,3 1200/1200 3");
  expect(calls()).toHaveLength(3);
  // Biten arama durdurulmaz.
  expect(api.chartSearchCancel).not.toHaveBeenCalled();
  expect(search.query).toBe(QUERY);
});

test("a new search gets a new id, cancels the old one and drops its late page", async () => {
  const [old, fresh] = [deferred(), deferred()];
  vi.mocked(api.chartSearch).mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise);
  render(<Probe />);
  act(() => search.start(QUERY));
  const oldId = calls()[0][1];
  const other = { ...QUERY, scope: { ...QUERY.scope, exclude_prashna: false } };
  act(() => search.start(other));
  const newId = calls()[1][1];
  expect(newId).not.toBe(oldId);
  expect(api.chartSearchCancel).toHaveBeenCalledWith(oldId);
  // Eski aramanın geç gelen sayfası gösterilmez ve sonraki sayfası istenmez.
  await act(async () => old.resolve(page(7, false)));
  expect(state()).toBe("running  0/? 0");
  expect(calls()).toHaveLength(2);
  await act(async () => fresh.resolve(page(1, true, 300)));
  expect(state()).toBe("done 1 300/300 1");
  expect(search.query).toBe(other);
});

test("stopping tells Rust, keeps what arrived and ignores the page in flight", async () => {
  const replies = [deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.chartSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  act(() => search.start(QUERY));
  const id = calls()[0][1];
  await act(async () => replies[0].resolve(page(1, false)));
  act(() => search.stop());
  expect(state()).toBe("stopped 1 500/1200 1");
  expect(api.chartSearchCancel).toHaveBeenCalledWith(id);
  // Yoldaki sayfa (ya da Rust'ın `search_stale` yanıtı) durumu değiştirmez.
  await act(async () => replies[1].resolve({ status: "error", error: { code: "search_stale", params: {} } }));
  expect(state()).toBe("stopped 1 500/1200 1");
  expect(calls()).toHaveLength(2);
  // Süren arama yokken "Durdur" hiçbir şey yapmaz.
  act(() => search.stop());
  expect(api.chartSearchCancel).toHaveBeenCalledTimes(1);
});

test("an error keeps the matches of the same search and a new search starts empty", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.chartSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  act(() => search.start(QUERY));
  await act(async () => replies[0].resolve(page(1, false)));
  await act(async () => replies[1].resolve({ status: "error", error: { code: "engine_stopped", params: {} } }));
  expect(state()).toBe("error 1 500/1200 1");
  expect(search.error?.code).toBe("engine_stopped");
  // Hatadan sonraki arama boş başlar: hatadan önceki eşleşmeler yeni aramada gösterilmez.
  act(() => search.start(QUERY));
  expect(state()).toBe("running  0/? 0");
  expect(search.error).toBeNull();
  await act(async () => replies[2].resolve(page(4, true, 10)));
  expect(state()).toBe("done 4 10/10 4");
});

test("closing the surface cancels a running search and reset clears the list", async () => {
  const replies = [deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.chartSearch).mockReturnValueOnce(r.promise));
  const view = render(<Probe />);
  act(() => search.start(QUERY));
  await act(async () => replies[0].resolve(page(1, true, 10)));
  act(() => search.reset());
  expect(state()).toBe("idle  0/? 0");
  act(() => search.start(QUERY));
  const id = calls()[1][1];
  view.unmount();
  expect(api.chartSearchCancel).toHaveBeenLastCalledWith(id);
});
