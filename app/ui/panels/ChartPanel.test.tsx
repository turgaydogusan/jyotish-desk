import { fireEvent, screen, waitFor } from "@testing-library/react";
import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ChartPanel } from "./ChartPanel";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

const LAYOUT = (settings: Record<string, unknown>) => ({
  schema_version: 1,
  active_layout_id: "user:t",
  layouts: [{ id: "user:t", name: "T", builtin: false, panels: [{ id: "c", kind: "chart", x: 0, y: 0, w: 6, h: 12, settings } satisfies Panel] }],
});

function renderPanel(settings: Record<string, unknown>, opened = fixture().opened) {
  const layout = LAYOUT(settings);
  const panel = layout.layouts[0].panels[0];
  return renderWithStores(<ChartPanel panel={panel} />, { layout, opened });
}

test("varga and style selectors write the panel settings and keep unknown keys", async () => {
  const { persistence } = renderPanel({ varga: "D1", style: "north", transit_ring: false });
  const varga = screen.getByRole("combobox", { name: "Varga" });
  expect(varga).toHaveDisplayValue("D1 · Rasi");
  expect(varga.querySelectorAll("option")).toHaveLength(16);
  fireEvent.change(varga, { target: { value: "D9" } });
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "circle" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D9", style: "circle", transit_ring: false });
  expect(persistence.saved.at(-1)!.chartId).toBe(fixture().opened.chart_record.id);
});

test("unknown birth time hides the lagna, names houses by sign and explains why", () => {
  const { container } = renderPanel({ varga: "D1", style: "north" }, unknownTimeOpened());
  expect(screen.getByRole("note")).toHaveTextContent("Doğum saati bilinmiyor: lagna ve evler gösterilmiyor, 1. ev Koç.");
  expect(container.querySelector(".chart-cell--lagna")).toBeNull();
  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("data-sign", "0");
  expect(screen.queryByRole("button", { name: /\. ev,/ })).toBeNull();
  expect(screen.getByRole("note")).not.toHaveTextContent("Bölüm haritasındaki");
});

test("unknown birth time in a divisional chart adds that placements are approximate", () => {
  renderPanel({ varga: "D9", style: "north" }, unknownTimeOpened());
  expect(screen.getByRole("note")).toHaveTextContent("Bölüm haritasındaki yerleşimler doğum saatine bağlıdır ve yaklaşıktır.");
});

test("D9 marks vargottama and never combust", () => {
  const opened = fixture().opened;
  const jupiter = opened.chart.grahas.find((g) => g.graha === "Jupiter")!;
  expect(jupiter.vargottama).toBe(true);
  opened.chart.grahas.find((g) => g.graha === "Mercury")!.combust = true;
  renderPanel({ varga: "D9", style: "south" }, opened);
  expect(screen.getByRole("button", { name: /^Jü, .*vargottama$/ })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /yanık/ })).toBeNull();
});

// Görsel regresyon (temadan bağımsız işaretleme): üç biçim, gerçek Delhi D1. Renkler
// token'larla CSS'te; ekran görüntüsü karşılaştırması Playwright `e2e:visual`'da.
test.each(["north", "south", "circle"])("Delhi D1 %s markup matches the snapshot", async (style) => {
  const { container } = renderPanel({ varga: "D1", style });
  await expect(container.querySelector("svg")!.outerHTML).toMatchFileSnapshot(`../chart/__snapshots__/delhi-d1-${style}.svg`);
});

test("the transit ring toggle is saved with the layout and reads no transits while off", async () => {
  const { persistence } = renderPanel({ varga: "D1", style: "north", transit_ring: false });
  const ring = screen.getByRole("checkbox", { name: "Transit halkası" });
  expect(ring).not.toBeChecked();
  expect(api.transitsAt).not.toHaveBeenCalled();
  fireEvent.click(ring);
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D1", style: "north", transit_ring: true });
});

test("with the ring on, D1 draws the nine transits and lists them for screen readers; other vargas do not", async () => {
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
  const d1 = renderPanel({ varga: "D1", transit_ring: true });
  await waitFor(() => expect(d1.container.querySelectorAll(".chart-transit")).toHaveLength(9));
  expect(screen.getByText(/^Transit:/)).toHaveClass("visually-hidden");
  d1.unmount();
  vi.mocked(api.transitsAt).mockClear();
  const d9 = renderPanel({ varga: "D9", transit_ring: true });
  expect(d9.container.querySelector(".chart-transit")).toBeNull();
  const ring = screen.getByRole("checkbox", { name: "Transit halkası" });
  expect(ring).toBeDisabled();
  expect(ring).toHaveAccessibleDescription("Transit halkası yalnızca D1'de çizilir.");
  expect(api.transitsAt).not.toHaveBeenCalled();
});
