import { act, fireEvent, render, screen } from "@testing-library/react";
import { DashaTreePanel } from "../dasha/DashaTreePanel";
import { setLanguage } from "../i18n";
import { tr } from "../i18n/tr";
import { PANEL_KINDS, type Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ChartPanel } from "./ChartPanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import { panelDefinition, PANEL_REGISTRY } from "./registry";
import { UnavailablePanel } from "./UnavailablePanel";

const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
  id: "c",
  kind: "chart",
  x: 0,
  y: 0,
  w: 6,
  h: 12,
  settings,
});
const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };

test("registry covers the seven kinds; chart, graha_table and dasha_tree are implemented", () => {
  expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
  expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
  expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
  expect(PANEL_REGISTRY.dasha_tree.component).toBe(DashaTreePanel);
  for (const kind of ["dasha_strip", "transit_table", "notes", "birth_info"] as const) {
    expect(PANEL_REGISTRY[kind].component).toBe(UnavailablePanel);
  }
  expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
  const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
  for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
  expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D9" }))).toBe("D9");
  expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D5" }))).toBe("D1");
});

test("chart panel draws the open chart's D1 with its lagna", () => {
  const { container } = renderWithStores(<ChartPanel panel={chartPanel()} />);
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Gü, Yay 16°" })).toBeInTheDocument();
  // Lagna İkizler (İki): 1. ev.
  expect(screen.getByRole("button", { name: "1. ev, İki" })).toBeInTheDocument();
  expect(container.querySelector("svg")!.dataset.style).toBe("north");
});

test("chart panel draws any varga of the chart and honours the style setting", () => {
  const { container } = renderWithStores(<ChartPanel panel={chartPanel({ varga: "D9", style: "south" })} />);
  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
  expect(container.querySelector("svg")!.dataset.style).toBe("south");
  // D9 lagnası Kova (Kov).
  expect(screen.getByRole("button", { name: "1. ev, Kov" })).toBeInTheDocument();
});

test("an unknown style falls back to north; a varga missing from the chart says so", () => {
  const opened = fixture().opened;
  opened.chart.vargas = opened.chart.vargas.filter((v) => v.varga !== "D60");
  const { container } = renderWithStores(
    <>
      <ChartPanel panel={chartPanel({ style: "spiral" })} />
      <ChartPanel panel={chartPanel({ varga: "D60" })} />
    </>,
    { opened },
  );
  expect(container.querySelector("svg")!.dataset.style).toBe("north");
  expect(screen.getByText("D60 bu haritada yok.")).toBeInTheDocument();
});

test("without an open chart both panels say so", () => {
  renderWithStores(
    <>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </>,
    { opened: null },
  );
  expect(screen.getAllByText("Açık harita yok.")).toHaveLength(2);
  expect(screen.queryByRole("group")).toBeNull();
});

test("graha table lists body, sign, degree with minutes and the retro marker", () => {
  renderWithStores(<GrahaTablePanel panel={tablePanel} />);
  const sun = screen.getByRole("button", { name: "Güneş" }).closest("tr")!;
  expect(sun).toHaveTextContent("Yay");
  expect(sun).toHaveTextContent("16°30′");
  const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  expect(saturn).toHaveTextContent("Koç");
  expect(saturn).toHaveTextContent("℞");
  expect(screen.getAllByRole("row")).toHaveLength(11); // başlık + lagna + 9 graha
});

test("a null degree is shown as a dash", () => {
  const opened = fixture().opened;
  opened.chart.grahas[2].deg_in_sign = null;
  renderWithStores(<GrahaTablePanel panel={tablePanel} />, { opened });
  const mars = screen.getByRole("button", { name: "Mars" }).closest("tr")!;
  expect(mars.querySelectorAll("td")[1]).toHaveTextContent("—");
});

test("hovering a graha in the table highlights it in the chart, click pins, Esc clears", () => {
  const { container } = renderWithStores(
    <>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </>,
  );
  const row = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  const chartSaturn = () => container.querySelector('svg [data-graha="Saturn"]')!;
  fireEvent.mouseEnter(row);
  expect(chartSaturn()).toHaveClass("chart-planet--highlight");
  expect(row).toHaveClass("is-highlighted");
  fireEvent.mouseLeave(row);
  expect(chartSaturn()).not.toHaveClass("chart-planet--highlight");

  fireEvent.click(chartSaturn());
  expect(row).toHaveClass("is-highlighted");
  expect(screen.getByRole("button", { name: "Satürn" })).toHaveAttribute("aria-pressed", "true");
  fireEvent.keyDown(window, { key: "Escape" });
  expect(row).not.toHaveClass("is-highlighted");
});

test("unavailable and unknown panels explain themselves without an error", () => {
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_strip" }} />);
  expect(screen.getByText("Bu panel sonraki bir sürümde gelecek.")).toBeInTheDocument();
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "yoga_list" }} />);
  expect(screen.getByText("“yoga_list” paneli bu sürümde tanınmıyor. Düzen olduğu gibi korunuyor.")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("labels follow the selected language", async () => {
  await setLanguage("en");
  renderWithStores(
    <>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </>,
  );
  expect(screen.getByRole("button", { name: "Su, Sg 16°" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Sun" }).closest("tr")).toHaveTextContent("Sg");
});

test("chart aria-pressed follows the pin only: focus hovers, Enter pins, Enter again unpins", () => {
  renderWithStores(<ChartPanel panel={chartPanel()} />);
  const planet = screen.getByRole("button", { name: "Gü, Yay 16°" });
  act(() => planet.focus());
  expect(planet).toHaveAttribute("aria-pressed", "false");
  fireEvent.keyDown(planet, { key: "Enter" });
  expect(planet).toHaveAttribute("aria-pressed", "true");
  fireEvent.keyDown(planet, { key: "Enter" });
  expect(planet).toHaveAttribute("aria-pressed", "false");
});
