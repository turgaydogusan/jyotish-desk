import { act, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { OpenedClient, TransitView } from "../bindings";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransit, useTransitControl, type Transit } from "./store";

const ok = (data: TransitView) => ({ status: "ok" as const, data });
let control: Transit;
let session: ReturnType<typeof useSession>;

function Consumer() {
  const t = useTransit();
  return <output data-testid="utc">{t.view?.utc ?? ""}</output>;
}
function Control() {
  control = useTransitControl();
  session = useSession();
  return null;
}

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue(ok(fixture().transit)));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("nothing is requested while no panel needs transits", async () => {
  renderWithStores(<Control />);
  await act(() => Promise.resolve());
  expect(api.transitsAt).not.toHaveBeenCalled();
});

test("a consumer asks once for now, with the open chart's settings, houses and dasha seed", async () => {
  const o = fixture().opened;
  renderWithStores(<Consumer />);
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent(fixture().transit.utc));
  expect(api.transitsAt).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.transitsAt).mock.calls[0]).toEqual(["Now", o.calc_settings, o.chart.bhavas, o.chart.dasha.seed]);
});

test("a later moment replaces the view; a late reply for the old moment is dropped", async () => {
  let release: (v: ReturnType<typeof ok>) => void = () => {};
  vi.mocked(api.transitsAt).mockReturnValueOnce(new Promise((r) => (release = r)));
  const later = { ...fixture().transit, utc: "2030-01-01T00:00:00Z" };
  vi.mocked(api.transitsAt).mockResolvedValueOnce(ok(later));
  renderWithStores(
    <>
      <Consumer />
      <Control />
    </>,
  );
  act(() => control.setAt({ Local: { date: "2030-01-01", time: "03:00", tz: browserTimeZone() } }));
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z"));
  await act(async () => release(ok(fixture().transit)));
  expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z");
});

test("opening another chart goes back to now", async () => {
  renderWithStores(
    <>
      <Consumer />
      <Control />
    </>,
  );
  act(() => control.setAt({ Jd: 2_460_000.5 }));
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: 2_460_000.5 }));
  const other: OpenedClient = fixture().opened;
  other.chart_record = { ...other.chart_record, id: 77 };
  act(() => session.open(other));
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(control.at).toBe("Now");
});

test("choosing now again while already at now asks Rust again, so a stale now can be refreshed", async () => {
  renderWithStores(
    <>
      <Consumer />
      <Control />
    </>,
  );
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent(fixture().transit.utc));
  expect(api.transitsAt).toHaveBeenCalledTimes(1);
  vi.mocked(api.transitsAt).mockResolvedValueOnce(ok({ ...fixture().transit, utc: "2030-01-01T00:00:00Z" }));
  act(() => control.setAt("Now"));
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z"));
  expect(api.transitsAt).toHaveBeenCalledTimes(2);
  expect(vi.mocked(api.transitsAt).mock.calls[1][0]).toBe("Now");
  expect(control.at).toBe("Now");
});
