import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVimshopaka: vi.fn() } }));

import { api, type Result } from "../api";
import type { VimshopakaReport } from "../bindings";
import { setLanguage } from "../i18n";
import { fr } from "../i18n/fr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { formatVimshopaka, VIMSHOPAKA_SCHEMES, VimshopakaPanel, vimshopakaDetail } from "./VimshopakaPanel";

const ok = (report: VimshopakaReport = fixture().natal_extras.vimshopaka): Result<VimshopakaReport> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "vb", kind: "vimshopaka", x: 0, y: 0, w: 7, h: 12, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <VimshopakaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const CAPTION = "Vimshopaka bala: yedi grahanın dört varga kümesindeki değeri, yirmi üzerinden";
const summary = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeVimshopaka).mockReset();
  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok());
});

test("summary: seven grahas with their value in the four varga sets, three decimals, unrounded", async () => {
  renderPanel();
  const table = await summary();
  const opened = fixture().opened;
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(1);
  expect(api.computeVimshopaka).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Shadvarga", "Saptavarga", "Dashavarga", "Shodashavarga"]);
  // Delhi fikstürü.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "14.600", "13.950", "11.725", "12.475"],
    ["Ay", "12.050", "11.650", "10.600", "10.275"],
    ["Mars", "17.100", "16.975", "16.425", "16.300"],
    ["Merkür", "10.550", "10.225", "10.800", "11.700"],
    ["Jüpiter", "17.200", "17.200", "14.150", "13.700"],
    ["Venüs", "14.900", "15.200", "13.300", "13.075"],
    ["Satürn", "7.100", "6.850", "10.550", "10.350"],
  ]);
  // Özet görünümünde varga tablosu yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.getByText(/^Basamaklar: 20 kendi burcu · 18 çok dost · 15 dost · 10 nötr · 7 düşman · 5 çok düşman\./)).toBeInTheDocument();
  expect(screen.getByText(/^Yücelme ve moolatrikona ayrı basamak değildir;/)).toBeInTheDocument();
  expect(screen.getByText("Doğrulanmamış tablo: küme ağırlıkları ve basamak değerleri basılı kaynakla henüz karşılaştırılmadı.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: vimshopaka.bphs_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati/)).toBeNull();
});

test("vargas view: the sign and the rung value in each of the sixteen vargas; the setting is stored with the panel", async () => {
  const { persistence } = renderPanel();
  await summary();
  fireEvent.change(screen.getByRole("combobox", { name: "Ayrıntı" }), { target: { value: "vargas" } });
  const table = await screen.findByRole("table", { name: "On altı vargada burç ve basamak değeri" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "D1", "D2", "D3", "D4", "D7", "D9", "D10", "D12", "D16", "D20", "D24", "D27", "D30", "D40", "D45", "D60"]);
  // Güneş Yay 16°'dedir (tek, çift doğalı burç). D2: tek burcun ikinci yarısı Ay'ın horasıdır
  // (Yengeç). D3: ikinci drekkana, beşinci burç (Koç). D4: üçüncü parça, yedinci burç
  // (İkizler). D7: dördüncü parça, Yay'dan dördüncü burç (Balık). D9: çift doğalı burçta
  // beşinci burçtan (Koç) sayılır, beşinci navamsa: Aslan, kendi burcu (20).
  const sun = within(table).getAllByRole("row")[1];
  expect(cells(sun).slice(0, 7)).toEqual(["Güneş", "Yay 10", "Yen 18", "Koç 18", "İki 7", "Bal 10", "Asl 20"]);
  // Yeniden istek yoktur: görünüm aynı raporu çizer.
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(1);
  await waitFor(() =>
    expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ detail: "vargas" }),
  );
  expect(vimshopakaDetail({ detail: "vargas" })).toBe("vargas");
  expect(vimshopakaDetail({ detail: "items" })).toBe("summary");
  expect(vimshopakaDetail({})).toBe("summary");
});

test("the scheme columns are Rust's four sets and values are written without rounding", () => {
  expect(VIMSHOPAKA_SCHEMES.map(([, name]) => name)).toEqual(fixture().natal_extras.schemes);
  expect(VIMSHOPAKA_SCHEMES.map(([key]) => key)).toEqual(fixture().natal_extras.schemes.map((s) => s.toLowerCase()));
  expect([20, 5, 11.725, 12.475, 14.275, null, Number.NaN].map(formatVimshopaka)).toEqual(["20.000", "5.000", "11.725", "12.475", "14.275", "—", "—"]);
  // Her değer 0,025'in katıdır: üç ondalık yuvarlama gerektirmez.
  for (const g of fixture().natal_extras.vimshopaka.grahas) {
    for (const [key] of VIMSHOPAKA_SCHEMES) expect(Math.abs((g[key] as number) * 40 - Math.round((g[key] as number) * 40))).toBeLessThan(1e-9);
  }
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeVimshopaka).not.toHaveBeenCalled();
});

test("unknown and approximate birth times are said in words; the table is still shown", async () => {
  const report = fixture().natal_extras.vimshopaka;
  report.approximate = true;
  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok(report));
  const first = renderPanel({}, unknownTimeOpened());
  await summary();
  expect(vi.mocked(api.computeVimshopaka).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText(/^Doğum saati bilinmiyor: vargalar öğlen 12:00 konumlarındandır\./)).toBeInTheDocument();
  expect(screen.queryByText(/^Doğum saati yaklaşık/)).toBeNull();
  first.unmount();

  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok());
  const approx = fixture().opened;
  approx.chart_record.birth.time_certainty = "Approx";
  renderPanel({}, approx);
  await summary();
  expect(screen.getByText(/^Doğum saati yaklaşık: D16 ve üstü vargalar/)).toBeInTheDocument();
  expect(screen.queryByText(/^Doğum saati bilinmiyor/)).toBeNull();
});

test("shared highlight: pinning a graha marks its row in both tables", async () => {
  renderPanel({ detail: "vargas" });
  const table = await summary();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.graha);
  expect(marked()).toEqual([]);
  fireEvent.click(within(table).getByRole("button", { name: "Mars" }));
  expect(marked()).toEqual(["Mars", "Mars"]);
  expect(within(table).getByRole("button", { name: "Mars" })).toHaveAttribute("aria-pressed", "true");
});

test("an error keeps the toolbar and is retried; focus stays inside the panel", async () => {
  vi.mocked(api.computeVimshopaka).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await summary();
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("set names stay the same in every language; the rest is translated; French uses the typographic apostrophe", async () => {
  await setLanguage("en");
  const first = renderPanel({ detail: "vargas" });
  const table = await screen.findByRole("table", { name: "Vimshopaka bala: the value of the seven grahas in the four varga sets, out of twenty" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Shadvarga", "Saptavarga", "Dashavarga", "Shodashavarga"]);
  expect(screen.getByRole("table", { name: "Sign and rung value in the sixteen vargas" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Detail" })).toHaveDisplayValue("Vargas");
  first.unmount();
  await setLanguage("fr");
  renderPanel();
  const french = await screen.findByRole("table", { name: "Vimshopaka bala : valeur des sept grahas dans les quatre ensembles de vargas, sur vingt" });
  expect(bodyRows(french)[0]).toEqual(["Soleil", "14.600", "13.950", "11.725", "12.475"]);
  const texts = Object.values(fr.vimshopaka).flatMap((v) => (typeof v === "string" ? [v] : Object.values(v)));
  for (const text of texts) expect(text).not.toContain("'");
});
