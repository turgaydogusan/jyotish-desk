// Graha tablosunun isteğe bağlı "Nakshatra lordu" sütunu (Plan 26). Ayrı dosyadır:
// `GrahaTablePanel.test.tsx` eşzamanlı planlarla paylaşılır.
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
  vi.mocked(api.computeAvasthas).mockReset();
});

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const lord = (row: string) => document.querySelector(`[data-nakshatra-lord="${row}"]`)!;

test("the nakshatra lord column is off by default", () => {
  renderTable();
  expect(headers()).not.toContain("Nakshatra lordu");
  expect(document.querySelector("[data-nakshatra-lord]")).toBeNull();
});

test("the column names the lord of each graha's nakshatra and of the lagna's, without a command", () => {
  renderTable({ columns: ["nakshatra"], nakshatraLord: true });
  expect(headers()).toEqual(["Graha", "Nakshatra", "Nakshatra lordu"]);
  // Delhi fikstürünün nakshatraları, `Graha::ALL` sırasında: 20, 15, 23, 19, 1, 17, 2, 8, 22;
  // lagna 6. Lord, dokuzlu sıranın ((n − 1) mod 9). adıdır (Ketu, Venüs, Güneş, Ay, Mars, Rahu,
  // Jüpiter, Satürn, Merkür): 20 → 1 Venüs; 15 → 5 Rahu; 23 → 4 Mars; 19 → 0 Ketu; 1 → 0 Ketu;
  // 17 → 7 Satürn; 2 → 1 Venüs; 8 → 7 Satürn; 22 → 3 Ay; lagna 6 → 5 Rahu.
  expect(fixture().opened.chart.grahas.map((g) => g.nakshatra)).toEqual([20, 15, 23, 19, 1, 17, 2, 8, 22]);
  expect(fixture().opened.chart.lagna.nakshatra).toBe(6);
  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map((g) => lord(g).textContent)).toEqual([
    "Venüs",
    "Rahu",
    "Mars",
    "Ketu",
    "Ketu",
    "Satürn",
    "Venüs",
    "Satürn",
    "Ay",
  ]);
  expect(lord("lagna")).toHaveTextContent(/^Rahu$/);
  expect(api.computeJaimini).not.toHaveBeenCalled();
  expect(api.computeAvasthas).not.toHaveBeenCalled();
});

test("the column sits after the karaka and Baladi columns", () => {
  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas }));
  renderTable({ columns: ["sign"], karaka: true, baladi: true, nakshatraLord: true });
  expect(headers()).toEqual(["Graha", "Burç", "Karaka", "Baladi", "Nakshatra lordu"]);
});

test("unknown birth time: the Moon's lord is marked approximate and the lagna row is hidden", () => {
  renderTable({ columns: ["nakshatra"], nakshatraLord: true }, unknownTimeOpened());
  expect(lord("Moon")).toHaveTextContent("≈yaklaşık Rahu");
  expect(lord("Sun")).toHaveTextContent(/^Venüs$/);
  expect(document.querySelector('[data-nakshatra-lord="lagna"]')).toBeNull();
});

test("the checkbox writes settings.nakshatraLord and leaves the other settings alone", async () => {
  const { persistence } = renderTable({ columns: ["sign", "house"], karaka: false });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  fireEvent.click(summary);
  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Nakshatra lordu" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: false, nakshatraLord: true });
});
