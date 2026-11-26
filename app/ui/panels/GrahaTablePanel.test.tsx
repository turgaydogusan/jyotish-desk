import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
});

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const row = (name: string) => screen.getByRole("button", { name }).closest("tr")!;

test("default columns and a lagna row, values straight from the chart", () => {
  renderTable();
  expect(headers()).toEqual(["Graha", "Burç", "Derece", "Nakshatra", "Pada", "Retro", "Yanık", "Durum", "Ev"]);
  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", "16°20′", "Ardra", "3", "", "", "", "1"]);
  const moon = row("Ay");
  expect([...moon.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["Ter", "19°28′", "Swati", "4", "", "", "Dost", "5"]);
  expect(row("Satürn")).toHaveTextContent("Rretro");
  expect(row("Satürn")).toHaveTextContent("Düşük");
  expect(row("Rahu").querySelectorAll("td")[6]).toHaveTextContent("—");
  expect(row("Rahu").querySelectorAll("td")[4]).toHaveTextContent(/^$/);
  expect(row("Ketu").querySelectorAll("td")[4]).toHaveTextContent(/^$/);
});

test("optional columns show longitude, signed latitude and speed", () => {
  renderTable({ columns: ["lon", "lat", "speed"] });
  expect(headers()).toEqual(["Graha", "Boylam", "Enlem", "Hız (°/gün)"]);
  expect([...row("Ay").querySelectorAll("td")].map((c) => c.textContent)).toEqual(["199°28′", "+5°10′", "+12.021"]);
});

test("the column chooser writes settings.columns in canonical order and keeps one column", async () => {
  const { persistence } = renderTable({ columns: ["house"] });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  const chooser = summary.closest("details")!;
  fireEvent.click(summary);
  const house = within(chooser).getByRole("checkbox", { name: "Ev" });
  expect(house).toBeChecked();
  expect(house).toBeDisabled();
  fireEvent.click(within(chooser).getByRole("checkbox", { name: "Boylam" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings.columns).toEqual(["lon", "house"]);
});

test("unknown birth time hides the lagna row and house column and marks the Moon approximate", () => {
  renderTable({}, unknownTimeOpened());
  expect(screen.queryByRole("rowheader", { name: "Lagna" })).toBeNull();
  expect(headers()).not.toContain("Ev");
  const moon = row("Ay");
  expect(moon.querySelectorAll("td")[2]).toHaveTextContent("≈yaklaşık Swati");
  expect(moon.querySelector('[title="yaklaşık"]')).not.toBeNull();
  // `title` yalnızca görünmez işarettedir: "yaklaşık" ekran okuyucuda bir kez okunur.
  expect(moon.querySelector('[title="yaklaşık"]')).toHaveAttribute("aria-hidden", "true");
  expect(row("Güneş").querySelector('[title="yaklaşık"]')).toBeNull();
});

test("the lagna row shows its longitude, leaves latitude and speed empty, and a missing longitude is a dash", () => {
  const lagna = () => screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  const first = renderTable({ columns: ["lon", "lat", "speed"] });
  expect([...lagna().querySelectorAll("td")].map((c) => c.textContent)).toEqual(["76°20′", "", ""]);
  first.unmount();
  const o = fixture().opened;
  (o.chart.lagna as { lon: number | null }).lon = null;
  renderTable({ columns: ["lon"] }, o);
  expect([...lagna().querySelectorAll("td")].map((c) => c.textContent)).toEqual(["—"]);
});

test("the karaka column is off by default and asks for nothing", () => {
  renderTable();
  expect(headers()).not.toContain("Karaka");
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("the karaka column shows each graha's role from the Jaimini report; Ketu and the lagna have none", async () => {
  renderTable({ columns: ["sign"], karaka: true });
  expect(headers()).toEqual(["Graha", "Burç", "Karaka"]);
  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  const role = (g: string) => document.querySelector(`[data-karaka="${g}"]`)!.textContent;
  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map(role)).toEqual([
    "MK Matrikaraka",
    "AK Atmakaraka",
    "GK Gnatikaraka",
    "PiK Pitrikaraka",
    "DK Darakaraka",
    "PK Putrakaraka",
    "BK Bhratrikaraka",
    "AmK Amatyakaraka",
    "",
  ]);
  expect(within(row("Ay")).getByTitle("Atmakaraka")).toHaveTextContent("AK");
  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", ""]);
});

test("unknown birth time: the Moon's karaka role is marked approximate, the other roles are not", async () => {
  renderTable({ columns: ["sign"], karaka: true }, unknownTimeOpened());
  const moon = () => document.querySelector('[data-karaka="Moon"]')!;
  await waitFor(() => expect(moon()).toHaveTextContent("≈yaklaşık AK Atmakaraka"));
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(moon().querySelector('[title="yaklaşık"]')).toHaveAttribute("aria-hidden", "true");
  expect(document.querySelector('[data-karaka="Sun"]')).toHaveTextContent(/^MK Matrikaraka$/);
  expect(document.querySelector('[data-karaka="Ketu"]')!.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("the karaka checkbox writes settings.karaka and leaves settings.columns alone", async () => {
  const { persistence } = renderTable({ columns: ["sign", "house"] });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  fireEvent.click(summary);
  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Karaka" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: true });
});

test("a failed karaka request keeps the table and offers a retry", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderTable({ karaka: true });
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(row("Ay")).toBeInTheDocument();
  expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent(/^$/);
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
});
