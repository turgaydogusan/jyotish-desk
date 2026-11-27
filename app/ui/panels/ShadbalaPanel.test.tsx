import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeShadbala: vi.fn() } }));

import { api, type Result } from "../api";
import type { ShadbalaReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { balaUnit, formatBala, formatFixed, formatRatio } from "./bala";
import { BELOW_MARK, ShadbalaPanel, shadbalaDetail } from "./ShadbalaPanel";

const ok = (report: ShadbalaReport | null = fixture().shadbala): Result<ShadbalaReport | null> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "sb", kind: "shadbala", x: 0, y: 0, w: 8, h: 12, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <ShadbalaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const CAPTION = "Shadbala: yedi grahanın altı gücü, toplamı, gerekli en az değeri, oranı ve sırası";
const summary = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeShadbala).mockReset();
  vi.mocked(api.computeShadbala).mockImplementation(async () => ok());
});

test("summary: seven grahas with the six strengths, total, required value, ratio and rank; time lords below", async () => {
  renderPanel();
  const table = await summary();
  const opened = fixture().opened;
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
  expect(api.computeShadbala).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Sthana", "Dig", "Zaman gücü", "Cheshta", "Naisargika", "Drik", "Toplam", "Gerekli", "Oran", "Sıra"]);
  // Delhi fikstürü, virupa, bir ondalık; oran iki ondalık.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "216.2", "34.3", "137.9", "1.2", "60.0", "55.8", "505.3", "390.0", "1.30", "2"],
    ["Ay", "141.5", "44.7", "113.3", "19.0", "51.4", "43.5", "413.5", "360.0", "1.15", "4"],
    ["Mars", "248.0", "50.2", "81.2", "20.4", "17.1", "−4.3", "412.6", "300.0", "1.38", "5"],
    ["Merkür", "182.3", "2.8", "175.3", "8.0", "25.7", "60.0", "454.1", "420.0", "1.08", "3"],
    ["Jüpiter", "233.8", "35.0", "185.8", "36.4", "34.3", "7.8", "533.1", "390.0", "1.37", "1"],
    ["Venüs", "168.6", "38.6", "57.9", "26.3", "42.9", "36.7", "371.0", "330.0", "1.12", "6"],
    ["Satürn", "108.2", "19.9", "182.8", "41.6", "8.6", "−7.4", "353.7", "300.0", "1.18", "7"],
  ]);
  // Özet görünümünde alt kalem tabloları yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  const lords = screen.getByRole("list", { name: "Zaman lordları" });
  expect(within(lords).getAllByRole("listitem").map((li) => li.textContent)).toEqual([
    "Yıl lordu: Merkür",
    "Ay lordu: Jüpiter",
    "Gün lordu: Satürn",
    "Saat lordu: Güneş",
    "Tribhaga lordu: Satürn",
  ]);
  expect(screen.getByText("Değerler virupa cinsindendir (60 virupa = 1 rupa).")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: shadbala.bphs_v1")).toBeInTheDocument();
  // Fikstürde bütün oranlar 1'in üstündedir: işaret ve açıklaması yoktur.
  expect(screen.queryByText(BELOW_MARK)).toBeNull();
  expect(screen.queryByText(/gerekli en az değerin altında/)).toBeNull();
});

test("items view adds the items of sthana bala and of the temporal strength", async () => {
  renderPanel({ detail: "items" });
  await summary();
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toHaveDisplayValue("Kalemler");
  const sthana = screen.getByRole("table", { name: "Sthana bala kalemleri" });
  expect(cells(within(sthana).getAllByRole("row")[0])).toEqual(["Graha", "Uccha", "Saptavargaja", "Ojayugma", "Kendradi", "Drekkana", "Toplam"]);
  expect(bodyRows(sthana)).toEqual([
    ["Güneş", "22.2", "104.0", "30.0", "60.0", "0.0", "216.2"],
    ["Ay", "4.5", "92.0", "15.0", "30.0", "0.0", "141.5"],
    ["Mars", "58.0", "145.0", "15.0", "15.0", "15.0", "248.0"],
    ["Merkür", "32.3", "60.0", "30.0", "60.0", "0.0", "182.3"],
    ["Jüpiter", "28.8", "130.0", "30.0", "30.0", "15.0", "233.8"],
    ["Venüs", "13.6", "110.0", "30.0", "15.0", "0.0", "168.6"],
    ["Satürn", "1.2", "32.0", "30.0", "30.0", "15.0", "108.2"],
  ]);
  const temporal = screen.getByRole("table", { name: "Zaman gücü kalemleri" });
  expect(cells(within(temporal).getAllByRole("row")[0])).toEqual(["Graha", "Gündüz–gece", "Paksha", "Tribhaga", "Yıl lordu", "Ay lordu", "Gün lordu", "Saat lordu", "Ayana", "Toplam"]);
  expect(bodyRows(temporal)).toEqual([
    ["Güneş", "34.5", "41.0", "0.0", "0.0", "0.0", "0.0", "60.0", "2.4", "137.9"],
    ["Ay", "25.5", "38.0", "0.0", "0.0", "0.0", "0.0", "0.0", "49.8", "113.3"],
    ["Mars", "25.5", "41.0", "0.0", "0.0", "0.0", "0.0", "0.0", "14.8", "81.2"],
    ["Merkür", "60.0", "41.0", "0.0", "15.0", "0.0", "0.0", "0.0", "59.3", "175.3"],
    ["Jüpiter", "34.5", "19.0", "60.0", "0.0", "30.0", "0.0", "0.0", "42.2", "185.8"],
    ["Venüs", "34.5", "19.0", "0.0", "0.0", "0.0", "0.0", "0.0", "4.4", "57.9"],
    ["Satürn", "25.5", "41.0", "60.0", "0.0", "0.0", "45.0", "0.0", "11.3", "182.8"],
  ]);
});

test("detail and unit are saved with the layout, redraw the panel and ask for nothing again", async () => {
  const { persistence } = renderPanel();
  const table = await summary();
  fireEvent.change(screen.getByRole("combobox", { name: "Birim" }), { target: { value: "rupa" } });
  // Rupa: iki ondalık; toplam Rust'ın `rupas` alanıdır; oran ve sıra birimden bağımsızdır.
  await waitFor(() => expect(bodyRows(table)[0]).toEqual(["Güneş", "3.60", "0.57", "2.30", "0.02", "1.00", "0.93", "8.42", "6.50", "1.30", "2"]));
  expect(bodyRows(table)[2]).toEqual(["Mars", "4.13", "0.84", "1.35", "0.34", "0.29", "−0.07", "6.88", "5.00", "1.38", "5"]);
  expect(screen.getByText("Değerler rupa cinsindendir (1 rupa = 60 virupa).")).toBeInTheDocument();
  fireEvent.change(screen.getByRole("combobox", { name: "Ayrıntı" }), { target: { value: "items" } });
  await screen.findByRole("table", { name: "Sthana bala kalemleri" });
  await waitFor(() =>
    expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ unit: "rupa", detail: "items" }),
  );
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
});

test("setting readers and number formats: unknown values fall back; minus is typographic; no negative zero", () => {
  expect([shadbalaDetail({}), shadbalaDetail({ detail: "all" }), shadbalaDetail({ detail: "items" })]).toEqual(["summary", "summary", "items"]);
  expect([balaUnit({}), balaUnit({ unit: "shashtiamsa" }), balaUnit({ unit: "rupa" })]).toEqual(["virupa", "virupa", "rupa"]);
  expect([formatBala(505.34, "virupa"), formatBala(505.34, "rupa"), formatBala(-7.35, "virupa"), formatBala(-7.35, "rupa")]).toEqual(["505.3", "8.42", "−7.3", "−0.12"]);
  expect([formatBala(-0.04, "virupa"), formatBala(-0.2, "rupa"), formatBala(0, "virupa")]).toEqual(["0.0", "0.00", "0.0"]);
  expect([formatBala(null, "virupa"), formatFixed(null, 2), formatFixed(Number.NaN, 2), formatFixed(Number.POSITIVE_INFINITY, 1)]).toEqual(["—", "—", "—", "—"]);
});

test("a ratio below one carries the text mark and a legend; a graha war is written as text", async () => {
  const report = fixture().shadbala;
  report.grahas[5].ratio = 0.94;
  report.grahas[2].war = { with: "Venus", winner: "Mars" };
  report.grahas[5].war = { with: "Mars", winner: "Mars" };
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await summary()).getAllByRole("row");
  const venus = rows[6];
  expect(venus.querySelectorAll("td")[8].textContent).toBe(`0.94 ${BELOW_MARK}gerekli değerin altında`);
  // İşaret ekran okuyucudan gizlidir; anlamı sözcükle okunur.
  expect(within(venus).getByText(BELOW_MARK)).toHaveAttribute("aria-hidden", "true");
  expect(within(venus).getByText("gerekli değerin altında")).toHaveClass("visually-hidden");
  expect(rows[1].querySelectorAll("td")[8].textContent).toBe("1.30");
  expect(screen.getByText(`${BELOW_MARK} toplam, klasik gerekli en az değerin altında.`)).toBeInTheDocument();
  // Savaş: kısa metin görünür, tam adlar ve kazanan ekran okuyucuya okunur.
  const mars = rows[3];
  expect(within(mars).getByText("savaş: Ve")).toHaveAttribute("aria-hidden", "true");
  expect(within(mars).getByText("Venüs ile graha savaşında; kazanan Mars")).toHaveClass("visually-hidden");
  expect(within(venus).getByText("savaş: Ma")).toBeInTheDocument();
  expect(within(rows[1]).queryByText(/savaş/)).toBeNull();
});

test("a ratio just below one never reads 1.00 next to the below mark", async () => {
  // Rounding 0.995–0.999… to two decimals would print "1.00 <1".
  expect([formatRatio(0.9951), formatRatio(0.9999), formatRatio(0.994), formatRatio(0.94), formatRatio(1), formatRatio(1.004), formatRatio(1.3)]).toEqual(["0.99", "0.99", "0.99", "0.94", "1.00", "1.00", "1.30"]);
  expect([formatRatio(null), formatRatio(Number.NaN), formatRatio(-0.2)]).toEqual(["—", "—", "−0.20"]);
  const report = fixture().shadbala;
  report.grahas[5].ratio = 0.9972;
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(report));
  renderPanel();
  const venus = within(await summary()).getAllByRole("row")[6];
  expect(venus.querySelectorAll("td")[8].textContent).toBe(`0.99 ${BELOW_MARK}gerekli değerin altında`);
});

test("every table sits in a keyboard-focusable scroll group named by its caption", async () => {
  // A narrow panel scrolls the tables sideways; the items tables hold no focusable control,
  // so the scroll container itself takes the keyboard focus (arrow keys scroll it).
  renderPanel({ detail: "items" });
  await summary();
  for (const name of [CAPTION, "Sthana bala kalemleri", "Zaman gücü kalemleri"]) {
    const group = screen.getByRole("group", { name });
    expect(group).toHaveClass("bala-scroll");
    expect(group).toHaveAttribute("tabindex", "0");
    expect(within(group).getByRole("table", { name })).toBeInTheDocument();
  }
  expect(screen.getAllByRole("group")).toHaveLength(3);
});

test("unknown birth time: Rust returns no report and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: Shadbala ve Bhava bala doğum saatine bağlı olduğu için hesaplanmıyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
});

test("no sunrise or sunset: with a known birth time the missing report is explained by the Sun", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel();
  expect(
    await screen.findByText("Doğum yerinde o gün gün doğumu ya da batımı yok: zaman gücü ölçülemediği için Shadbala ve Bhava bala hesaplanmıyor."),
  ).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(true);
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeShadbala).not.toHaveBeenCalled();
});

test("an error is shown in the panel with the toolbar and retry asks again", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await summary();
  expect(api.computeShadbala).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Shadbala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  expect(document.activeElement).toBe(retry);
  fireEvent.click(retry);
  await summary();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Shadbala" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<ShadbalaReport | null>) => void;
    vi.mocked(api.computeShadbala).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    // Yüklenirken "rapor yok" açıklaması gösterilmez.
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks its row in every table; a time lord pins the same graha", async () => {
  renderPanel({ detail: "items" });
  const table = await summary();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.graha);
  const jupiter = within(table).getByRole("button", { name: "Jüpiter" });
  fireEvent.click(jupiter);
  expect(jupiter).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["Jupiter", "Jupiter", "Jupiter"]);
  // Ay lordu Jüpiter: aynı hedef, o da basılı görünür.
  const lords = screen.getByRole("list", { name: "Zaman lordları" });
  expect(within(lords).getByRole("button", { name: "Jüpiter" })).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(within(lords).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["Saturn", "Saturn", "Saturn"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});
