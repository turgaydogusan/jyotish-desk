import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { muhurtaSearch: vi.fn() } }));

import { api, type Result } from "../api";
import type { MuhurtaSearchView } from "../bindings";
import { fixture } from "../test/fixture";
import { useMuhurtaSearch, type MuhurtaSearch } from "./useMuhurtaSearch";

const FX = fixture();
const PAGES = FX.muhurta.pages;
const QUERY = FX.muhurta.query;
const SETTINGS = FX.opened.chart.meta.settings;

/** Elle çözülen yanıt: sayfaların geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<MuhurtaSearchView>) => void;
  const promise = new Promise<Result<MuhurtaSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
const ok = (page: MuhurtaSearchView): Result<MuhurtaSearchView> => ({ status: "ok", data: page });

let search: MuhurtaSearch;
function Probe({ scope = "1" }: { scope?: string }) {
  search = useMuhurtaSearch(scope);
  return <span data-testid="state">{`${search.status} ${search.windows.length} ${search.scanned}/${search.chunks}`}</span>;
}
const state = () => screen.getByTestId("state").textContent;

beforeEach(() => vi.mocked(api.muhurtaSearch).mockReset());

test("pages are asked one after another with the cursor of the page before, and windows add up", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  expect(state()).toBe("idle 0 0/0");
  act(() => search.start(QUERY, SETTINGS));
  expect(state()).toBe("running 0 0/0");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0]).toEqual([QUERY, SETTINGS, null]);
  await act(async () => replies[0].resolve(ok(PAGES[0])));
  expect(state()).toBe("running 15 1/3");
  // İkinci istek ilk yanıtın imleciyle gider (aralık imleçte taşınır).
  expect(vi.mocked(api.muhurtaSearch).mock.calls[1]).toEqual([QUERY, SETTINGS, PAGES[0].next]);
  await act(async () => replies[1].resolve(ok(PAGES[1])));
  expect(state()).toBe("running 27 2/3");
  await act(async () => replies[2].resolve(ok(PAGES[2])));
  expect(state()).toBe("done 30 3/3");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
  expect(search.range).toEqual({ from_jd_ut: PAGES[0].from_jd_ut, to_jd_ut: PAGES[0].to_jd_ut, from_offset_secs: 19800, to_offset_secs: 19800 });
  expect(search.method).toBe("muhurta.v1");
  expect(search.windows).toEqual(PAGES.flatMap((p) => p.windows));
});

test("stopping keeps the windows so far and asks for no further page", async () => {
  const replies = [deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  act(() => search.start(QUERY, SETTINGS));
  await act(async () => replies[0].resolve(ok(PAGES[0])));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  act(() => search.stop());
  expect(state()).toBe("stopped 15 1/3");
  // Yoldaki sayfanın yanıtı düşer; üçüncü sayfa hiç istenmez.
  await act(async () => replies[1].resolve(ok(PAGES[1])));
  expect(state()).toBe("stopped 15 1/3");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  // Durmuş aramada `stop` bir şey değiştirmez.
  act(() => search.stop());
  expect(state()).toBe("stopped 15 1/3");
});

test("a new search replaces the old one and the old one's late page is dropped", async () => {
  const [old, fresh] = [deferred(), deferred()];
  vi.mocked(api.muhurtaSearch).mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise);
  render(<Probe />);
  act(() => search.start(QUERY, SETTINGS));
  act(() => search.start({ ...QUERY, to_date: "2026-12-01" }, SETTINGS));
  expect(state()).toBe("running 0 0/0");
  await act(async () => old.resolve(ok(PAGES[0])));
  expect(state()).toBe("running 0 0/0");
  await act(async () => fresh.resolve(ok({ ...PAGES[2], scanned: 1, chunks: 1 })));
  expect(state()).toBe("done 3 1/1");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
});

test("an error ends the search and keeps the windows that arrived", async () => {
  vi.mocked(api.muhurtaSearch)
    .mockResolvedValueOnce(ok(PAGES[0]))
    .mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  render(<Probe />);
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("error 15 1/3");
  expect(search.error?.code).toBe("calc");
  // Yeni arama hatayı ve eski pencereleri siler.
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("done 3 1/1");
  expect(search.error).toBeNull();
});

test("warnings of every page are gathered once", async () => {
  vi.mocked(api.muhurtaSearch)
    .mockResolvedValueOnce(ok({ ...PAGES[0], warnings: ["SunriseUnavailable"] }))
    .mockResolvedValueOnce(ok({ ...PAGES[1], warnings: ["SunriseUnavailable", "LowPrecision"] }))
    .mockResolvedValueOnce(ok(PAGES[2]));
  render(<Probe />);
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("done 30 3/3");
  expect(search.warnings).toEqual(["SunriseUnavailable", "LowPrecision"]);
});

test("another chart drops the running search and its results; unmounting stops asking", async () => {
  const [first, second, third] = [deferred(), deferred(), deferred()];
  vi.mocked(api.muhurtaSearch).mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise).mockReturnValueOnce(third.promise);
  const { rerender, unmount } = render(<Probe scope="1" />);
  act(() => search.start(QUERY, SETTINGS));
  await act(async () => first.resolve(ok(PAGES[0])));
  expect(state()).toBe("running 15 1/3");
  rerender(<Probe scope="2" />);
  expect(state()).toBe("idle 0 0/0");
  await act(async () => second.resolve(ok(PAGES[1])));
  expect(state()).toBe("idle 0 0/0");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  act(() => search.start(QUERY, SETTINGS));
  unmount();
  await act(async () => third.resolve(ok(PAGES[0])));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
});
