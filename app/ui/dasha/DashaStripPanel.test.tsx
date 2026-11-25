import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { DashaStripPanel } from "./DashaStripPanel";
import { childrenOf } from "./tree";

const strip = (level?: string): Panel => ({ id: "ds", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings: level ? { level } : {} });

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit }));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("antar (default): the current maha's nine antardashas, the one at the transit moment marked", async () => {
  renderWithStores(<DashaStripPanel panel={strip()} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  const items = within(list).getAllByRole("button");
  expect(items).toHaveLength(9);
  const [maha, antar] = fixture().transit.dasha_path;
  const expected = childrenOf(fixture().opened.chart.dasha.periods, [maha]);
  expect(items.map((b) => b.getAttribute("data-lord"))).toEqual(expected.map((p) => p.lords[1]));
  const current = items.filter((b) => b.getAttribute("aria-current") === "true");
  expect(current).toHaveLength(1);
  expect(current[0]).toHaveAttribute("data-lord", antar);
  expect(current[0]).toHaveTextContent("transit anında");
});

test("maha level lists the nine mahadashas", async () => {
  renderWithStores(<DashaStripPanel panel={strip("maha")} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  expect(within(list).getAllByRole("button")).toHaveLength(9);
});

test("choosing a period moves the transit moment to its start", async () => {
  renderWithStores(<DashaStripPanel panel={strip("pratyantar")} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  const first = within(list).getAllByRole("button")[0];
  fireEvent.click(first);
  const [maha, antar] = fixture().transit.dasha_path;
  const start = childrenOf(fixture().opened.chart.dasha.periods, [maha, antar])[0].start_jd_ut;
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: start }));
});

test("a transit error is shown with a retry", async () => {
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(<DashaStripPanel panel={strip()} />);
  fireEvent.click(await screen.findByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("list", { name: /Dasa şeridi/ })).toBeInTheDocument();
});

test("no open chart: a message, no request", () => {
  renderWithStores(<DashaStripPanel panel={strip()} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.transitsAt).not.toHaveBeenCalled();
});
