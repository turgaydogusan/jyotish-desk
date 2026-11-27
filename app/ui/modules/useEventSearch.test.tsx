import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { eventSearch: vi.fn(), eventSearchCancel: vi.fn() } }));

import { api, type Result } from "../api";
import type { EventSearchView } from "../bindings";
import { fixture } from "../test/fixture";
import { useEventSearch, type EventSearch } from "./useEventSearch";

const FX = fixture().search;
const SPEC = FX.event_spec;

function deferred() {
  let resolve!: (r: Result<EventSearchView>) => void;
  const promise = new Promise<Result<EventSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
const ok = (view: EventSearchView): Result<EventSearchView> => ({ status: "ok", data: view });

let search: EventSearch;
function Probe({ scope = "1" }: { scope?: string }) {
  search = useEventSearch(scope);
  return <span data-testid="state">{`${search.status} ${search.view?.events.length ?? "-"} ${search.error?.code ?? "-"}`}</span>;
}
const state = () => screen.getByTestId("state").textContent;
const calls = () => vi.mocked(api.eventSearch).mock.calls;

beforeEach(() => {
  vi.mocked(api.eventSearch).mockReset();
  vi.mocked(api.eventSearchCancel).mockReset();
  vi.mocked(api.eventSearchCancel).mockResolvedValue({ status: "ok", data: null });
});

test("a search runs once with a new id and shows what came back", async () => {
  const reply = deferred();
  vi.mocked(api.eventSearch).mockReturnValueOnce(reply.promise);
  render(<Probe />);
  expect(state()).toBe("idle - -");
  act(() => search.start(SPEC));
  expect(state()).toBe("running - -");
  expect(calls()).toHaveLength(1);
  expect(calls()[0][0]).toBe(SPEC);
  await act(async () => reply.resolve(ok(FX.events)));
  expect(state()).toBe(`done ${FX.events.events.length} -`);
  expect(api.eventSearchCancel).not.toHaveBeenCalled();
});

test("a new search cancels the old one and drops its late answer", async () => {
  const [old, fresh] = [deferred(), deferred()];
  vi.mocked(api.eventSearch).mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise);
  render(<Probe />);
  act(() => search.start(SPEC));
  act(() => search.start({ ...SPEC, grahas: ["Saturn"] }));
  expect(calls()[1][1]).not.toBe(calls()[0][1]);
  expect(api.eventSearchCancel).toHaveBeenCalledWith(calls()[0][1]);
  await act(async () => old.resolve(ok(FX.events)));
  expect(state()).toBe("running - -");
  await act(async () => fresh.resolve(ok({ ...FX.events, events: [] })));
  expect(state()).toBe("done 0 -");
});

test("stopping tells Rust and the stale answer of a stopped search is not an error", async () => {
  const reply = deferred();
  vi.mocked(api.eventSearch).mockReturnValueOnce(reply.promise);
  render(<Probe />);
  act(() => search.start(SPEC));
  act(() => search.stop());
  expect(state()).toBe("stopped - -");
  expect(api.eventSearchCancel).toHaveBeenCalledWith(calls()[0][1]);
  await act(async () => reply.resolve({ status: "error", error: { code: "search_stale", params: {} } }));
  expect(state()).toBe("stopped - -");
  // Başka bir yüzeyin aramasıyla geçersiz olan arama da durmuş sayılır.
  vi.mocked(api.eventSearch).mockResolvedValueOnce({ status: "error", error: { code: "search_stale", params: {} } });
  await act(async () => search.start(SPEC));
  expect(state()).toBe("stopped - -");
});

test("an error shows no earlier events, and another chart cancels and clears", async () => {
  vi.mocked(api.eventSearch).mockResolvedValueOnce(ok(FX.events));
  const view = render(<Probe />);
  await act(async () => search.start(SPEC));
  expect(state()).toBe(`done ${FX.events.events.length} -`);
  // Hata: önceki aramanın olayları gösterilmez.
  vi.mocked(api.eventSearch).mockResolvedValueOnce({ status: "error", error: { code: "invalid_search_input", params: { field: "range" } } });
  await act(async () => search.start(SPEC));
  expect(state()).toBe("error - invalid_search_input");
  // Hatadan sonraki arama boş başlar.
  const pending = deferred();
  vi.mocked(api.eventSearch).mockReturnValueOnce(pending.promise);
  act(() => search.start(SPEC));
  expect(state()).toBe("running - -");
  const id = calls()[2][1];
  view.rerender(<Probe scope="2" />);
  expect(state()).toBe("idle - -");
  expect(api.eventSearchCancel).toHaveBeenLastCalledWith(id);
  await act(async () => pending.resolve(ok(FX.events)));
  expect(state()).toBe("idle - -");
});
