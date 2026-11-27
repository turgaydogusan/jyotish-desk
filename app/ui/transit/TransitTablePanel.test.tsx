import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone } from "./store";
import { TransitTablePanel } from "./TransitTablePanel";

const panel: Panel = { id: "tt", kind: "transit_table", x: 0, y: 0, w: 6, h: 8, settings: {} };

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit }));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("nine rows: sign, degree, nakshatra, its lord, natal house and the retro mark (none on the nodes)", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />);
  const table = await screen.findByRole("table");
  const rows = within(table).getAllByRole("row").slice(1);
  expect(rows).toHaveLength(9);
  const tr = fixture().transit.snapshot.grahas;
  rows.forEach((row, i) => {
    const cells = within(row).getAllByRole("cell");
    expect(cells[4]).toHaveTextContent(String(tr[i].natal_house));
    const nodes = tr[i].graha === "Rahu" || tr[i].graha === "Ketu";
    expect(cells[5]).toHaveTextContent(tr[i].retrograde && !nodes ? "R" : "");
  });
});

test("the nakshatra lord column names the lord of each transit nakshatra", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />);
  const table = await screen.findByRole("table");
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Burç", "Derece", "Nakshatra", "Nakshatra lordu", "Natal ev", "Retro"]);
  // Fikstür (2026-11-25 10:00 UT) nakshatraları, `Graha::ALL` sırasında: 17, 4, 10, 16, 10, 14,
  // 26, 23, 10. Lord, dokuzlu sıranın ((n − 1) mod 9). adıdır: Ketu, Venüs, Güneş, Ay, Mars,
  // Rahu, Jüpiter, Satürn, Merkür. 17 → 7 Satürn; 4 → 3 Ay; 10 → 0 Ketu; 16 → 6 Jüpiter;
  // 14 → 4 Mars; 26 → 7 Satürn; 23 → 4 Mars.
  expect(fixture().transit.snapshot.grahas.map((g) => g.nakshatra)).toEqual([17, 4, 10, 16, 10, 14, 26, 23, 10]);
  const lords = [...table.querySelectorAll("[data-nakshatra-lord]")].map((c) => c.textContent);
  expect(lords).toEqual(["Satürn", "Ay", "Ketu", "Jüpiter", "Ketu", "Mars", "Satürn", "Mars", "Ketu"]);
});

test("unknown birth time hides the natal house column values", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />, { opened: unknownTimeOpened() });
  const table = await screen.findByRole("table");
  for (const row of within(table).getAllByRole("row").slice(1)) expect(within(row).getAllByRole("cell")[4]).toHaveTextContent("—");
});

test("apply sends the wall-clock time in the browser's time zone; now goes back to now", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />);
  await screen.findByRole("table");
  fireEvent.change(screen.getByLabelText("Tarih"), { target: { value: "2026-12-01" } });
  fireEvent.change(screen.getByLabelText("Saat"), { target: { value: "09:30" } });
  fireEvent.click(screen.getByRole("button", { name: "Uygula" }));
  await waitFor(() =>
    expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Local: { date: "2026-12-01", time: "09:30", tz: browserTimeZone() } }),
  );
  const now = screen.getByRole("button", { name: "Şimdi" });
  expect(now).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(now);
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(now).toHaveAttribute("aria-pressed", "true");
});

test("a date typed right after the table appears is not overwritten by a late sync", async () => {
  // Zamanlayıcının her dilimi dolmuş görünür: React edilgen etkileri bir sonraki işe erteler
  // (yük altındaki CI ile aynı sıra). Kutuların eşitlenmesi bir etkide olsaydı yazılan tarihi ezerdi.
  let now = 0;
  const clock = vi.spyOn(performance, "now").mockImplementation(() => (now += 10));
  try {
    renderWithStores(<TransitTablePanel panel={panel} />);
    await screen.findByRole("table");
    fireEvent.change(screen.getByLabelText("Tarih"), { target: { value: "2026-12-01" } });
    fireEvent.change(screen.getByLabelText("Saat"), { target: { value: "09:30" } });
    fireEvent.click(screen.getByRole("button", { name: "Uygula" }));
    await waitFor(() =>
      expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Local: { date: "2026-12-01", time: "09:30", tz: browserTimeZone() } }),
    );
  } finally {
    clock.mockRestore();
  }
});

test("an error is shown in the panel with a retry", async () => {
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:00", suggestion: "2014-03-31T04:30:00" } } });
  renderWithStores(<TransitTablePanel panel={panel} />);
  fireEvent.click(await screen.findByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("table")).toBeInTheDocument();
  expect(api.transitsAt).toHaveBeenCalledTimes(2);
});
