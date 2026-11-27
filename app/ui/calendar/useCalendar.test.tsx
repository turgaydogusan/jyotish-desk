import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { monthCalendar: vi.fn() } }));

import { api, type Result } from "../api";
import type { CalendarRequest, CalendarView } from "../bindings";
import { calendarFixture } from "../test/calendarFixture";
import { useCalendar } from "./useCalendar";

const FX = calendarFixture();

/** Elle çözülen söz: yanıtın geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<CalendarView>) => void;
  const promise = new Promise<Result<CalendarView>>((r) => (resolve = r));
  return { promise, resolve };
}

function Probe({ request, epoch = 0, scope = "1:p" }: { request: CalendarRequest | null; epoch?: number; scope?: string }) {
  const c = useCalendar(request, epoch, scope);
  return (
    <span data-testid="state">
      {c.loading ? "loading" : c.error ? `error:${c.error.code}` : "settled"}|{c.shown ? `${c.shown.month.year}-${c.shown.month.month}` : "none"}
    </span>
  );
}

const state = () => screen.getByTestId("state").textContent;
const asked = (month: CalendarRequest["month"]): CalendarRequest => ({ ...FX.request, month });

beforeEach(() => vi.mocked(api.monthCalendar).mockReset());

test("without a request nothing is asked; a request is asked once", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValue({ status: "ok", data: FX.month });
  const { rerender } = render(<Probe request={null} />);
  expect(state()).toBe("settled|none");
  expect(api.monthCalendar).not.toHaveBeenCalled();
  rerender(<Probe request={FX.request} />);
  expect(state()).toBe("loading|none");
  await act(async () => {});
  expect(state()).toBe("settled|2026-11");
  expect(vi.mocked(api.monthCalendar).mock.calls).toEqual([[FX.request]]);
  // Aynı istekle yeniden çizim yeni istek göndermez.
  rerender(<Probe request={{ ...FX.request }} />);
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
});

test("the last month stays shown while the next one loads, and a late answer to an older request is dropped", async () => {
  const [first, december, january] = [deferred(), deferred(), deferred()];
  vi.mocked(api.monthCalendar).mockReturnValueOnce(first.promise).mockReturnValueOnce(december.promise).mockReturnValueOnce(january.promise);
  const { rerender } = render(<Probe request={asked(null)} />);
  await act(async () => first.resolve({ status: "ok", data: FX.month }));
  expect(state()).toBe("settled|2026-11");
  // İki hızlı adım: Aralık istenir, yanıtı gelmeden Ocak istenir.
  rerender(<Probe request={asked({ year: 2026, month: 12 })} />);
  expect(state()).toBe("loading|2026-11");
  rerender(<Probe request={asked({ year: 2027, month: 1 })} />);
  expect(state()).toBe("loading|2026-11");
  // Aralık'ın geç gelen yanıtı gösterilmez.
  await act(async () => december.resolve({ status: "ok", data: FX.month_next }));
  expect(state()).toBe("loading|2026-11");
  await act(async () => january.resolve({ status: "ok", data: FX.month_polar }));
  expect(state()).toBe("settled|2027-1");
  expect(api.monthCalendar).toHaveBeenCalledTimes(3);
});

test("a new epoch reads the same request again; an error forgets the shown month; another scope starts empty", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValue({ status: "ok", data: FX.month });
  const { rerender } = render(<Probe request={FX.request} />);
  await act(async () => {});
  rerender(<Probe request={FX.request} epoch={1} />);
  expect(state()).toBe("loading|2026-11");
  await act(async () => {});
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);

  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  rerender(<Probe request={asked({ year: 2026, month: 12 })} epoch={1} />);
  await act(async () => {});
  expect(state()).toBe("error:calc|none");

  // Başka harita ya da panel: önceki ay gösterilmez.
  rerender(<Probe request={FX.request} epoch={2} scope="2:p" />);
  expect(state()).toBe("loading|none");
  await act(async () => {});
  expect(state()).toBe("settled|2026-11");
});
