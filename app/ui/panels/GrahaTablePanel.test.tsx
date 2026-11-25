import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

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
  expect(row("Satürn")).toHaveTextContent("℞retro");
  expect(row("Satürn")).toHaveTextContent("Düşük");
  expect(row("Rahu").querySelectorAll("td")[6]).toHaveTextContent("—");
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
  expect(row("Güneş").querySelector('[title="yaklaşık"]')).toBeNull();
});
