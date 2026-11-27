// Graha tablosunun isteğe bağlı Baladi sütunu (Plan 24). Ayrı dosyadır: `GrahaTablePanel.test.tsx`
// paralel planlarla paylaşılır.
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn(), computeAvasthas: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
  vi.mocked(api.computeAvasthas).mockReset();
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas }));
});

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const baladi = (g: string) => document.querySelector(`[data-baladi="${g}"]`)!;

test("the Baladi column is off by default and asks for nothing", () => {
  renderTable();
  expect(headers()).not.toContain("Baladi");
  expect(api.computeAvasthas).not.toHaveBeenCalled();
  expect(document.querySelector("[data-baladi]")).toBeNull();
});

test("the Baladi column shows each graha's stage from the avastha report; the nodes and the lagna have none", async () => {
  renderTable({ columns: ["sign"], baladi: true });
  expect(headers()).toEqual(["Graha", "Burç", "Baladi"]);
  await waitFor(() => expect(baladi("Sun")).toHaveTextContent("Yuva"));
  expect(api.computeAvasthas).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0]).toMatchObject({ jd_ut: 2451545, time_known: true, lagna_number: null });
  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map((g) => baladi(g).textContent)).toEqual([
    "Yuva",
    "Vriddha",
    "Bala",
    "Kumara",
    "Bala",
    "Vriddha",
    "Yuva",
    "",
    "",
  ]);
  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", ""]);
  // Karaka raporu istenmez: iki sütun birbirinden bağımsızdır.
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("Baladi sits after the karaka column when both are on", async () => {
  renderTable({ columns: ["sign"], karaka: true, baladi: true });
  expect(headers()).toEqual(["Graha", "Burç", "Karaka", "Baladi"]);
  await waitFor(() => expect(baladi("Saturn")).toHaveTextContent("Yuva"));
  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
  const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  expect([...saturn.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["Koç", "BK Bhratrikaraka", "Yuva"]);
});

test("unknown birth time: the Moon's stage is marked approximate, the other stages are not", async () => {
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas_time_unknown }));
  renderTable({ columns: ["sign"], baladi: true }, unknownTimeOpened());
  await waitFor(() => expect(baladi("Moon")).toHaveTextContent("≈yaklaşık Vriddha"));
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0].time_known).toBe(false);
  expect(baladi("Sun")).toHaveTextContent(/^Yuva$/);
  expect(baladi("Sun").querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByRole("rowheader", { name: "Lagna" })).toBeNull();
});

test("the Baladi checkbox writes settings.baladi and leaves the other settings alone", async () => {
  const { persistence } = renderTable({ columns: ["sign", "house"], karaka: true });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  fireEvent.click(summary);
  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Baladi" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: true, baladi: true });
});

test("a failed avastha request keeps the table and offers a retry", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderTable({ baladi: true });
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("table")).toBeInTheDocument();
  expect(baladi("Moon")).toHaveTextContent(/^$/);
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(baladi("Moon")).toHaveTextContent("Vriddha"));
  expect(screen.queryByRole("alert")).toBeNull();
});
