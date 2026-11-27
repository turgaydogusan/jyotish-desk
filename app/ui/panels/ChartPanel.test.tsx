import { fireEvent, screen, waitFor } from "@testing-library/react";
import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, placidusOpened, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { ChartPanel, partnerPlanets } from "./ChartPanel";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), computeJaimini: vi.fn(), computeCompat: vi.fn(), chartInput: vi.fn(), chartBrief: vi.fn() } }));

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

/** Açık harita, uyum fikstürünün eşine bağlı; eşin okunması ve rapor fikstürden gelir. */
function withPartner(roles: "OpenIsGroom" | "OpenIsBride" = "OpenIsGroom") {
  const opened = fixture().opened;
  opened.view_state.compat_partner_uuid = fixture().compat.partner_uuid;
  opened.view_state.compat_roles = roles;
  vi.mocked(api.chartBrief).mockResolvedValue({ status: "ok", data: fixture().compat.brief });
  vi.mocked(api.chartInput).mockResolvedValue({ status: "ok", data: fixture().compat.input });
  vi.mocked(api.computeCompat).mockImplementation(async (groom) => ({
    status: "ok",
    data: groom.jd_ut === fixture().opened.chart.jd_ut ? fixture().compat.report : fixture().compat.report_swapped,
  }));
  return opened;
}

function clearCompat() {
  for (const f of [api.computeCompat, api.chartInput, api.chartBrief, api.transitsAt]) vi.mocked(f).mockReset();
}

test("the partner ring draws the nine grahas of the partner chart in D1 and lists them for screen readers", async () => {
  clearCompat();
  const { container } = renderPanel({ varga: "D1", partner_ring: true }, withPartner());
  await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
  // Eşin grahaları raporun ikinci yarısıdır (açık harita damat rolünde): Güneş Balık'ta.
  const partner = fixture().compat.report.overlay.slice(9);
  expect(partner[0]).toMatchObject({ owner: "Bride", graha: "Sun", sign: "Pisces" });
  expect(partnerPlanets(fixture().compat.report, "Groom").map((p) => [p.id, p.sign])).toEqual(partner.map((r) => [r.graha, ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"].indexOf(r.sign)]));
  // Çizim transit halkasınınkidir: parantezli kısaltma, etkileşimsiz; Güneş Balık evindedir.
  const sun = container.querySelector('.chart-transit[data-transit="Sun"]')!;
  expect(sun).toHaveTextContent("(Gü)");
  expect(sun).toHaveAttribute("aria-hidden", "true");
  expect(container.querySelector('[data-sign="11"]')).toHaveAttribute("data-house", sun.getAttribute("data-house")!);
  const list = screen.getByText(/^Eş \(Eş Danışan\): /);
  expect(list).toHaveClass("visually-hidden");
  expect(list.textContent!.replace(/^Eş \(Eş Danışan\): /, "").split(", ")).toHaveLength(9);
  expect(list).toHaveTextContent("Güneş Bal 2°");
  // Transitler okunmaz; rapor bir kez istenir.
  expect(api.transitsAt).not.toHaveBeenCalled();
  expect(api.computeCompat).toHaveBeenCalledTimes(1);
});

test("with the roles swapped the partner is the first half of the report: the same nine grahas are drawn", async () => {
  clearCompat();
  const { container } = renderPanel({ varga: "D1", partner_ring: true }, withPartner("OpenIsBride"));
  await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
  expect(vi.mocked(api.computeCompat).mock.calls[0][0]).toEqual(fixture().compat.input);
  expect(partnerPlanets(fixture().compat.report_swapped, "Bride")).toEqual(partnerPlanets(fixture().compat.report, "Groom"));
  expect(screen.getByText(/^Eş \(Eş Danışan\): /)).toHaveTextContent("Güneş Bal 2°");
});

test("the two rings are never on together: turning one on turns the other off", async () => {
  clearCompat();
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
  const layout = LAYOUT({ varga: "D1", style: "north", transit_ring: true });
  const { persistence, container } = renderWithStores(<LivePanel />, { layout, opened: withPartner() });
  const saved = () => persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
  const transit = screen.getByRole("checkbox", { name: "Transit halkası" });
  const partner = screen.getByRole("checkbox", { name: "Eş halkası" });
  expect([transit, partner].map((b) => (b as HTMLInputElement).checked)).toEqual([true, false]);
  expect(api.computeCompat).not.toHaveBeenCalled();
  fireEvent.click(partner);
  await waitFor(() => expect(saved()).toEqual({ varga: "D1", style: "north", transit_ring: false, partner_ring: true }));
  expect([transit, partner].map((b) => (b as HTMLInputElement).checked)).toEqual([false, true]);
  await waitFor(() => expect(screen.getByText(/^Eş \(/)).toBeInTheDocument());
  expect(screen.queryByText(/^Transit:/)).toBeNull();
  fireEvent.click(transit);
  await waitFor(() => expect(saved()).toEqual({ varga: "D1", style: "north", transit_ring: true, partner_ring: false }));
  await waitFor(() => expect(screen.getByText(/^Transit:/)).toBeInTheDocument());
  expect(screen.queryByText(/^Eş \(/)).toBeNull();
  expect(container.querySelectorAll(".chart-transit")).toHaveLength(9);
});

test("a record with both rings on draws the transit ring only", async () => {
  clearCompat();
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
  renderPanel({ varga: "D1", transit_ring: true, partner_ring: true }, withPartner());
  await screen.findByText(/^Transit:/);
  expect(screen.getByRole("checkbox", { name: "Eş halkası" })).not.toBeChecked();
  expect(api.computeCompat).not.toHaveBeenCalled();
});

test("the partner ring box is offered only while the open chart has a partner; outside D1 it says why", async () => {
  clearCompat();
  // Eş seçilmemiş: düzen ayarı açık olsa da kutu yoktur, hiçbir şey istenmez ve çizilmez.
  const none = renderPanel({ varga: "D1", partner_ring: true });
  expect(screen.queryByRole("checkbox", { name: "Eş halkası" })).toBeNull();
  expect(none.container.querySelector(".chart-transit")).toBeNull();
  expect(none.container.querySelector(".chart-toolbar-hint")).toBeNull();
  none.unmount();
  // Eş var, ama varga D9: kutu odaklanır, nedeni görünür bir satırdır ve tıklamak ayarı değiştirmez.
  const d9 = renderPanel({ varga: "D9", partner_ring: true }, withPartner());
  const box = screen.getByRole("checkbox", { name: "Eş halkası" });
  expect(box).toBeEnabled();
  expect(box).toHaveAttribute("aria-disabled", "true");
  expect(box).toHaveAccessibleDescription("Eş halkası yalnızca D1'de çizilir.");
  expect(screen.getByText("Eş halkası yalnızca D1'de çizilir.")).not.toHaveClass("visually-hidden");
  fireEvent.click(box);
  expect(box).toBeChecked();
  expect(d9.container.querySelector(".chart-transit")).toBeNull();
  for (const f of [api.computeCompat, api.chartInput, api.chartBrief]) expect(f).not.toHaveBeenCalled();
});

test("with the partner ring off nothing is asked even when a partner is linked", () => {
  clearCompat();
  renderPanel({ varga: "D1" }, withPartner());
  expect(screen.getByRole("checkbox", { name: "Eş halkası" })).not.toBeChecked();
  for (const f of [api.computeCompat, api.chartInput, api.chartBrief]) expect(f).not.toHaveBeenCalled();
});

test("a failed partner report leaves the chart drawn and offers a retry", async () => {
  clearCompat();
  const opened = withPartner();
  vi.mocked(api.computeCompat).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  const { container } = renderPanel({ varga: "D1", partner_ring: true }, opened);
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(container.querySelector("svg")).not.toBeNull();
  expect(container.querySelector(".chart-transit")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
});

// Chalit görünümü (yol haritası Plan 25). Fikstür: Delhi, Placidus; Merkür, Jüpiter ve Venüs
// burç evinden bir önceki bhavadadır.
const unknownPlacidus = () => {
  const o = placidusOpened();
  o.chart_record.birth.time = null;
  o.chart_record.birth.time_certainty = "Unknown";
  o.chart_record.resolved.warnings = ["TimeUnknownNoon"];
  return o;
};

test("whole sign: the chalit box is not offered; a stored setting shows it with the reason and draws the rasi chart", () => {
  const off = renderPanel({ varga: "D1", style: "north" });
  expect(screen.queryByRole("checkbox", { name: "Bhava chalit" })).toBeNull();
  expect(off.container.querySelector(".chart-toolbar-hint")).toBeNull();
  off.unmount();
  const { container } = renderPanel({ varga: "D1", style: "north", chalit: true });
  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
  expect(box).toBeChecked();
  expect(box).toBeEnabled();
  expect(box).toHaveAttribute("aria-disabled", "true");
  expect(box).toHaveAccessibleDescription("Ev sistemi tam burç: chalit görünümü rasi ile aynıdır.");
  expect(screen.getByText("Ev sistemi tam burç: chalit görünümü rasi ile aynıdır.")).not.toHaveClass("visually-hidden");
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(container.querySelector("[data-shifted]")).toBeNull();
  expect(screen.queryByText(/^Hücreler bhavadır/)).toBeNull();
});

test("the chalit toggle is saved with the layout and redraws the grahas by bhava without a request", async () => {
  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) vi.mocked(f).mockClear();
  const opened = placidusOpened();
  const { container, persistence } = renderWithStores(<LivePanel />, { layout: LAYOUT({ varga: "D1", style: "north" }), opened });
  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
  expect(box).not.toBeChecked();
  expect(box).not.toHaveAttribute("aria-disabled");
  // Rasi görünümü: Merkür burcunun evinde (7).
  expect(container.querySelector('[data-graha="Mercury"]')).toHaveAttribute("data-house", "7");
  expect(container.querySelector("[data-shifted]")).toBeNull();
  fireEvent.click(box);
  await waitFor(() => expect(screen.getByRole("group", { name: "Bhava chalit haritası" })).toBeInTheDocument());
  const houses = Object.fromEntries([...container.querySelectorAll("[data-graha]")].map((g) => [g.getAttribute("data-graha"), Number(g.getAttribute("data-house"))]));
  // Her graha haritanın verdiği bhavadadır (`Chart.grahas[].house`).
  expect(houses).toEqual(Object.fromEntries(opened.chart.grahas.map((g) => [g.graha, g.house])));
  expect(houses).toMatchObject({ Sun: 7, Mercury: 6, Jupiter: 10, Venus: 5 });
  // Çizim sırası hücre sırasıdır (5., 6. ve 10. bhava).
  expect([...container.querySelectorAll("[data-shifted]")].map((g) => g.getAttribute("data-graha"))).toEqual(["Venus", "Mercury", "Jupiter"]);
  // İşaret öteki işaretlerden sonra yazılır (Merkür bu haritada yanıktır: "Mec*").
  expect(container.querySelector('[data-graha="Mercury"] text')).toHaveTextContent(/^Mec\*$/);
  expect(container.querySelector('[data-graha="Venus"] text')).toHaveTextContent(/^Ve\*$/);
  // Ad grahanın kendi burcunu söyler; işaret sözcükle okunur.
  expect(screen.getByRole("button", { name: /^Me, Yay \d+°, yanık, bhavası farklı$/ })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /^Ve, Akr \d+°, bhavası farklı$/ })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /^Gü, Yay \d+°$/ })).toBeInTheDocument();
  // Hücreler yerinde: 1. ev lagna burcudur (İkizler).
  expect(screen.getByRole("button", { name: "1. ev, İki" })).toBeInTheDocument();
  expect(screen.getByText("Hücreler bhavadır (Placidus); burç numarası o numaralı rasi evinin burcudur. * Graha, burcunun evinden başka bir bhavadadır.")).toBeInTheDocument();
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ varga: "D1", style: "north", chalit: true }));
  fireEvent.click(screen.getByRole("checkbox", { name: "Bhava chalit" }));
  await waitFor(() => expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument());
  expect(container.querySelector("[data-shifted]")).toBeNull();
  expect(screen.queryByText(/^Hücreler bhavadır/)).toBeNull();
  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) expect(f).not.toHaveBeenCalled();
});

test.each(["north", "south", "circle"])("chalit %s: every graha sits in the cell of its bhava", (style) => {
  const opened = placidusOpened();
  const { container } = renderPanel({ varga: "D1", style, chalit: true }, opened);
  expect(container.querySelector("svg")!.dataset.style).toBe(style);
  for (const g of opened.chart.grahas) expect(container.querySelector(`[data-graha="${g.graha}"]`), g.graha).toHaveAttribute("data-house", String(g.house));
  expect(container.querySelectorAll("[data-shifted]")).toHaveLength(3);
});

test("chalit is drawn on D1 only and never without a birth time; the box says why", () => {
  const d9 = renderPanel({ varga: "D9", chalit: true }, placidusOpened());
  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
  expect(box).toBeEnabled();
  expect(box).toHaveAttribute("aria-disabled", "true");
  expect(box).toHaveAccessibleDescription("Bhava chalit yalnızca D1'de çizilir.");
  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
  expect(d9.container.querySelector("[data-shifted]")).toBeNull();
  // Kapalı kutu ayarı değiştirmez.
  fireEvent.click(box);
  expect(box).toBeChecked();
  d9.unmount();
  const unknown = renderPanel({ varga: "D1", chalit: true }, unknownPlacidus());
  const off = screen.getByRole("checkbox", { name: "Bhava chalit" });
  expect(off).toHaveAttribute("aria-disabled", "true");
  expect(off).toHaveAccessibleDescription("Doğum saati bilinmiyor: bhavalar lagnaya bağlıdır ve chalit görünümü çizilemez.");
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(unknown.container.querySelector("[data-shifted]")).toBeNull();
});

test("the chalit view draws no ring and no marks, asks for none and says so on the checked boxes", async () => {
  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) vi.mocked(f).mockClear();
  const { container } = renderWithStores(<LivePanel />, { layout: LAYOUT({ varga: "D1", transit_ring: true, arudha_marks: true, chalit: true }), opened: placidusOpened() });
  expect(screen.getByRole("group", { name: "Bhava chalit haritası" })).toBeInTheDocument();
  expect(container.querySelector(".chart-transit")).toBeNull();
  expect(container.querySelector(".chart-mark")).toBeNull();
  const note = "Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez.";
  expect(screen.getByText(note)).not.toHaveClass("visually-hidden");
  for (const name of ["Transit halkası", "Arudha padalar"]) {
    const box = screen.getByRole("checkbox", { name });
    expect(box).toBeChecked();
    expect(box).not.toHaveAttribute("aria-disabled");
    expect(box).toHaveAccessibleDescription(note);
  }
  expect(api.transitsAt).not.toHaveBeenCalled();
  expect(api.computeJaimini).not.toHaveBeenCalled();
  // Chalit kapanınca katmanlar geri gelir: istekler o zaman yapılır.
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
  fireEvent.click(screen.getByRole("checkbox", { name: "Bhava chalit" }));
  await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
  expect(screen.queryByText(note)).toBeNull();
});

test("an unchecked layer carries no chalit note", () => {
  renderPanel({ varga: "D1", chalit: true }, placidusOpened());
  expect(screen.queryByText("Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez.")).toBeNull();
  expect(screen.getByRole("checkbox", { name: "Transit halkası" })).not.toHaveAccessibleDescription();
});

test("Placidus inside the polar circle: the chalit view says the bhavas are Porphyry", () => {
  const opened = placidusOpened();
  opened.chart.meta.warnings = ["PlacidusPolarFallback"];
  renderPanel({ varga: "D1", chalit: true }, opened);
  expect(screen.getByText("Placidus bu enlemde hesaplanamadı; bhavalar Porphyry noktalarından sayılır.")).toHaveAttribute("role", "note");
  // Rasi görünümünde not yoktur (uyarı rozeti harita başlığındadır).
  const rasi = renderPanel({ varga: "D1" }, opened);
  expect(rasi.container.querySelector('[role="note"]')).toBeNull();
});
