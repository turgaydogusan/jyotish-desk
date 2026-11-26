import { fireEvent, screen, waitFor } from "@testing-library/react";
import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { ChartPanel } from "./ChartPanel";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), computeJaimini: vi.fn() } }));

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

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <ChartPanel panel={active.panels[0]} />;
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
  // Kapalı kutu yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır.
  expect(ring).toBeEnabled();
  expect(ring).toHaveAttribute("aria-disabled", "true");
  expect(ring).toHaveAccessibleDescription("Transit halkası yalnızca D1'de çizilir.");
  expect(screen.getByText("Transit halkası yalnızca D1'de çizilir.")).not.toHaveClass("visually-hidden");
  expect(api.transitsAt).not.toHaveBeenCalled();
});

test("the screen reader transit list says which transits are retrograde, never the nodes", async () => {
  const transit = fixture().transit;
  // Fikstürde Satürn, Rahu ve Ketu geri gider; düğümler retro diye okunmaz.
  expect(transit.snapshot.grahas.filter((g) => g.retrograde).map((g) => g.graha)).toEqual(["Saturn", "Rahu", "Ketu"]);
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: transit });
  renderPanel({ varga: "D1", transit_ring: true });
  const list = (await screen.findByText(/^Transit:/)).textContent ?? "";
  const items = list.replace(/^Transit: /, "").split(", ");
  expect(items).toHaveLength(9);
  expect(items.filter((item) => item.endsWith(" retro"))).toEqual([expect.stringMatching(/^Satürn .*° retro$/)]);
});

test("the arudha marks toggle is saved with the layout and asks for nothing while off", async () => {
  const { persistence, container } = renderPanel({ varga: "D1", style: "north" });
  const marks = screen.getByRole("checkbox", { name: "Arudha padalar" });
  expect(marks).not.toBeChecked();
  expect(marks).not.toHaveAttribute("aria-disabled");
  expect(marks).not.toHaveAccessibleDescription();
  expect(api.computeJaimini).not.toHaveBeenCalled();
  expect(container.querySelector(".chart-mark")).toBeNull();
  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
  fireEvent.click(marks);
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D1", style: "north", arudha_marks: true });
});

test("with the marks on, D1 writes the twelve padas in their signs and lists them for screen readers", async () => {
  vi.mocked(api.computeJaimini).mockClear();
  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
  const { container } = renderPanel({ varga: "D1", style: "south", arudha_marks: true });
  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(true);
  // Delhi fikstürü: AL Balık (10. ev), A7 Aslan (3. ev), UL Kova (9. ev); Yay'da üç pada.
  const houseOf = (text: string) => container.querySelector(`[data-mark="${text}"]`)!.getAttribute("data-house");
  expect([houseOf("AL"), houseOf("A7"), houseOf("UL")]).toEqual(["10", "3", "9"]);
  expect([houseOf("A4"), houseOf("A5"), houseOf("A11")]).toEqual(["7", "7", "7"]);
  const list = screen.getByText(/^Arudha padalar: /);
  expect(list).toHaveClass("visually-hidden");
  expect(list).toHaveTextContent("Arudha padalar: AL Bal, A2 Ter, A3 Koç, A4 Yay, A5 Yay, A6 Kov, A7 Asl, A8 Koç, A9 İki, A10 Boğ, A11 Yay, UL Kov");
  // Grahalar yerinde durur ve düğme olmayı sürdürür.
  expect(screen.getByRole("button", { name: "Gü, Yay 16°" })).toBeInTheDocument();
});

test("marks are written on D1 only and never without a birth time; the checkbox says why", async () => {
  vi.mocked(api.computeJaimini).mockClear();
  const d9 = renderPanel({ varga: "D9", arudha_marks: true });
  const marks = screen.getByRole("checkbox", { name: "Arudha padalar" });
  expect(marks).toBeEnabled();
  expect(marks).toHaveAttribute("aria-disabled", "true");
  expect(marks).toHaveAccessibleDescription("Arudha padalar yalnızca D1'de yazılır.");
  expect(screen.getByText("Arudha padalar yalnızca D1'de yazılır.")).not.toHaveClass("visually-hidden");
  expect(d9.container.querySelector(".chart-mark")).toBeNull();
  d9.unmount();
  const unknown = renderPanel({ varga: "D1", arudha_marks: true }, unknownTimeOpened());
  const off = screen.getByRole("checkbox", { name: "Arudha padalar" });
  expect(off).toBeEnabled();
  expect(off).toHaveAttribute("aria-disabled", "true");
  expect(off).toHaveAccessibleDescription("Doğum saati bilinmiyor: arudha padalar lagnaya bağlıdır ve yazılamaz.");
  expect(screen.getByText("Doğum saati bilinmiyor: arudha padalar lagnaya bağlıdır ve yazılamaz.")).not.toHaveClass("visually-hidden");
  expect(unknown.container.querySelector(".chart-mark")).toBeNull();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("a failed pada request leaves the chart drawn and offers a retry", async () => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
  const { container } = renderPanel({ varga: "D1", arudha_marks: true });
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(container.querySelector(".chart-mark")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
  expect(screen.queryByRole("alert")).toBeNull();
});

test("an unavailable checkbox keeps the focus, shows its reason and changes nothing when activated", async () => {
  vi.mocked(api.computeJaimini).mockClear();
  const { persistence } = renderPanel({ varga: "D9", style: "north", transit_ring: true, arudha_marks: false });
  const ring = screen.getByRole("checkbox", { name: "Transit halkası" });
  const marks = screen.getByRole("checkbox", { name: "Arudha padalar" });
  for (const box of [ring, marks]) {
    const before = (box as HTMLInputElement).checked;
    box.focus();
    expect(document.activeElement).toBe(box);
    fireEvent.click(box);
    expect((box as HTMLInputElement).checked).toBe(before);
    expect(document.activeElement).toBe(box);
  }
  expect(ring).toBeChecked();
  expect(marks).not.toBeChecked();
  // Varga değişince (kayıt tetiklenir) iki ayar da olduğu gibi durur.
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "south" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D9", style: "south", transit_ring: true, arudha_marks: false });
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("unchecking the box or leaving D1 removes the marks and their list", async () => {
  vi.mocked(api.computeJaimini).mockClear();
  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
  const { container } = renderWithStores(<LivePanel />, { layout: LAYOUT({ varga: "D1", style: "north", arudha_marks: true }), opened: fixture().opened });
  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
  fireEvent.change(screen.getByRole("combobox", { name: "Varga" }), { target: { value: "D9" } });
  await waitFor(() => expect(container.querySelector(".chart-mark")).toBeNull());
  expect(screen.queryByText(/^Arudha padalar: /)).toBeNull();
  // Ayar durur: D1'e dönünce işaretler yeniden yazılır.
  expect(screen.getByRole("checkbox", { name: "Arudha padalar" })).toBeChecked();
  fireEvent.change(screen.getByRole("combobox", { name: "Varga" }), { target: { value: "D1" } });
  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
  fireEvent.click(screen.getByRole("checkbox", { name: "Arudha padalar" }));
  await waitFor(() => expect(container.querySelector(".chart-mark")).toBeNull());
  expect(screen.queryByText(/^Arudha padalar: /)).toBeNull();
});
