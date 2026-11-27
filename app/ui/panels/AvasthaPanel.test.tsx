import { act, fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeAvasthas: vi.fn() } }));

import { api, type Result } from "../api";
import type { AvasthaReport } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES } from "./avasthaLabels";
import { AvasthaPanel } from "./AvasthaPanel";
import { PANEL_REGISTRY } from "./registry";

const ok = (report: AvasthaReport = fixture().avasthas): Result<AvasthaReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "av", kind: "avastha", x: 0, y: 0, w: 7, h: 16, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<AvasthaPanel panel={panel} />, { layout, opened });
}

const statesTable = () => screen.findByRole("table", { name: "Avasthalar: yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi durumu" });
const termsTable = () => screen.getByRole("table", { name: "Shayanadi terimleri" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
/** Lajjitadi hücresi: liste öğeleri ayrı ayrı (ayraç CSS'tedir). */
const states = (row: HTMLElement) => [...row.querySelectorAll('td[data-series="lajjitadi"] li')].map((li) => li.textContent);

beforeEach(() => {
  vi.mocked(api.computeAvasthas).mockReset();
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ok());
});

test("asks for the open chart's report and lists the four series of the seven grahas", async () => {
  renderPanel();
  const table = await statesTable();
  const opened = fixture().opened;
  expect(api.computeAvasthas).toHaveBeenCalledTimes(1);
  expect(api.computeAvasthas).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Baladi", "Jagradadi", "Lajjitadi", "Shayanadi"]);
  const rows = within(table).getAllByRole("row").slice(1);
  // Delhi fikstürü. Lajjitadi sütunu aşağıda liste öğeleriyle ayrıca denetlenir.
  expect(rows.map((r) => [cells(r)[0], cells(r)[1], cells(r)[2], cells(r)[4]])).toEqual([
    ["Güneş", "Yuva", "Swapna", "Kautuka"],
    ["Ay", "Vriddha", "Swapna", "Sabha"],
    ["Mars", "Bala", "Swapna", "Shayana"],
    ["Merkür", "Kumara", "Swapna", "Sabha"],
    ["Jüpiter", "Bala", "Swapna", "Nidra"],
    ["Venüs", "Vriddha", "Swapna", "Shayana"],
    ["Satürn", "Yuva", "Sushupti", "Gamana"],
  ]);
  expect(rows.map(states)).toEqual([["Mudita (Jüpiter)"], [], [], ["Mudita (Güneş)"], ["Kshudhita (Satürn)", "Mudita (Ay)"], [], ["Kshudhita (Ay)", "Mudita (Jüpiter)"]]);
  // Hiçbir durumda olmayan graha: görünen çizgi, okunan sözcük.
  const moon = rows[1].querySelector('td[data-series="lajjitadi"]')!;
  expect(moon).toHaveTextContent("—hiçbiri");
  expect(moon.querySelector('[aria-hidden="true"]')).toHaveTextContent("—");
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.queryByText(/gün doğumu yok/)).toBeNull();
});

test("the Shayanadi terms come from Rust: three terms, the total and the remainder for each graha", async () => {
  renderPanel();
  await statesTable();
  const rows = within(termsTable()).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Graha", "Nakshatra (N)", "Graha sayısı (G)", "Navamsa dilimi (A)", "Toplam", "Artan"]);
  // Güneş: 20 × 1 × 5 + 15 + 25 + 3 = 143 = 11 × 12 + 11. Jüpiter: 1 × 5 × 1 + 43 = 48, artan 0.
  expect(rows.slice(1).map(cells)).toEqual([
    ["Güneş", "20", "1", "5", "143", "11"],
    ["Ay", "15", "2", "6", "223", "7"],
    ["Mars", "23", "3", "2", "181", "1"],
    ["Merkür", "19", "4", "3", "271", "7"],
    ["Jüpiter", "1", "5", "1", "48", "0"],
    ["Venüs", "17", "6", "3", "349", "1"],
    ["Satürn", "2", "7", "5", "113", "5"],
  ]);
  expect(screen.getByText("Ortak terimler: doğum nakshatrası 15 (Swati), ghati 25, lagna 3 (İki).")).toBeInTheDocument();
  expect(screen.getByText(/^Toplam = N × G × A \+ doğum nakshatrası \+ ghati \+ lagna\./)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: avastha.bphs_v1")).toBeInTheDocument();
});

test("the unverified-table note and the reading note are always shown", async () => {
  renderPanel();
  await statesTable();
  expect(screen.getByText("Doğrulanmamış tablo: Lajjitadi kuralları ve Shayanadi formülü basılı kaynakla karşılaştırılmayı bekliyor.")).toBeInTheDocument();
  expect(screen.getByText(/^Burçlar ve evler rasi çerçevesindedir; dostluk ve düşmanlık doğal ilişkidir\./)).toBeInTheDocument();
});

test("a graha in several states lists them in rule order with the grahas that satisfy each rule", async () => {
  const report = fixture().avasthas;
  report.grahas[5].lajjitadi = [
    { state: "Lajjita", grahas: ["Sun"] },
    { state: "Garvita", grahas: [] },
    { state: "Kshudhita", grahas: ["Sun"] },
    { state: "Mudita", grahas: ["Saturn"] },
    { state: "Kshobhita", grahas: ["Sun", "Saturn"] },
  ];
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  const venus = within(await statesTable()).getAllByRole("row")[6];
  expect(states(venus)).toEqual(["Lajjita (Güneş)", "Garvita", "Kshudhita (Güneş)", "Mudita (Satürn)", "Kshobhita (Güneş, Satürn)"]);
  expect(within(venus).getByRole("list")).toBeInTheDocument();
  expect(within(venus).getAllByRole("listitem")).toHaveLength(5);
});

test("unknown birth time is sent to Rust; Lajjita and Shayanadi are absent and the Moon's row is approximate", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(fixture().avasthas_time_unknown));
  renderPanel(unknownTimeOpened());
  const table = await statesTable();
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Lajjita değerlendirilmedi ve Shayanadi hesaplanmadı (ikisi de lagna ister). Ay'ın durumları yaklaşıktır (≈).")).toBeInTheDocument();
  expect(screen.queryByText(/gün doğumu yok/)).toBeNull();
  const rows = within(table).getAllByRole("row").slice(1);
  // Shayanadi sütunu durur ama boştur: çizgi ve "hesaplanmadı".
  for (const row of rows) expect(row.querySelector('td[data-series="shayanadi"]')).toHaveTextContent("—hesaplanmadı");
  expect(rows.flatMap(states).some((s) => s.startsWith("Lajjita"))).toBe(false);
  // Ay'ın Baladi ve Jagradadi hücreleri yaklaşıktır; öteki satırlar değildir.
  expect(rows[1].querySelector('td[data-series="baladi"]')).toHaveTextContent("≈yaklaşık Vriddha");
  expect(rows[1].querySelector('td[data-series="jagradadi"]')).toHaveTextContent("≈yaklaşık Swapna");
  expect(rows[0].querySelector('[title="yaklaşık"]')).toBeNull();
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
  // Terim tablosu ve ortak terimler yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.queryByText(/^Ortak terimler/)).toBeNull();
  expect(screen.queryByText(/^Toplam = /)).toBeNull();
});

test("no sunrise on the birth day: only Shayanadi is missing and the panel says why", async () => {
  const report = fixture().avasthas;
  report.shayanadi_basis = null;
  for (const row of report.grahas) row.shayanadi = null;
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  const table = await statesTable();
  expect(screen.getByText("Doğum yerinde o gün gün doğumu yok: Shayanadi hesaplanmadı (formül gün doğumundan geçen ghatiyi ister).")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  const rows = within(table).getAllByRole("row").slice(1);
  expect(states(rows[6])).toEqual(["Kshudhita (Ay)", "Mudita (Jüpiter)"]);
  expect(rows[6].querySelector('td[data-series="shayanadi"]')).toHaveTextContent("—hesaplanmadı");
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.getAllByRole("table")).toHaveLength(1);
});

test("a low-precision warning from the report is shown as text", async () => {
  const report = fixture().avasthas;
  report.warnings = ["LowPrecision"];
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  await statesTable();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeAvasthas).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await statesTable();
  expect(api.computeAvasthas).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  const panel: Panel = { id: "av", kind: "avastha", x: 0, y: 0, w: 7, h: 16, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Avasthalar">
      <AvasthaPanel panel={panel} />
    </section>,
    { layout, opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await statesTable();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Avasthalar" }));
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<AvasthaReport>) => void;
    vi.mocked(api.computeAvasthas).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(2);
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a pinned graha marks its row in both tables; both tables scroll by keyboard", async () => {
  renderPanel();
  const table = await statesTable();
  const saturn = within(table).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(2);
  expect(document.querySelector('tr[data-graha="Saturn"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('tr[data-terms-of="Saturn"]')).toHaveClass("is-highlighted");
  fireEvent.keyDown(window, { key: "Escape" });
  expect(saturn).toHaveAttribute("aria-pressed", "false");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(0);
  // Kaydırma kapları klavye odağı alır ve tablolarının adını taşır.
  const groups = screen.getAllByRole("group");
  expect(groups.map((g) => g.getAttribute("tabindex"))).toEqual(["0", "0"]);
  expect(groups[1]).toHaveAccessibleName("Shayanadi terimleri");
});

test("catalog: the panel is registered, opens from the palette at 7 × 16 and is named in three languages", () => {
  expect(PANEL_REGISTRY.avastha.component).toBe(AvasthaPanel);
  expect(PANEL_REGISTRY.avastha.titleKey).toBe("panel.avastha");
  expect(MODULE_PANELS.find((m) => m.kind === "avastha")).toEqual({ kind: "avastha", w: 7, h: 16 });
  expect([tr.panel.avastha, en.panel.avastha, fr.panel.avastha]).toEqual(["Avasthalar", "Avasthas", "Avasthas"]);
  // Fransızca metinlerde düz kesme işareti yoktur (C12).
  for (const text of Object.values(fr.avastha)) expect(text).not.toContain("'");
});

test("every avastha id that Rust can return has a name, and the names are the same in every language", () => {
  const ids = fixture().avastha_ids;
  expect(Object.keys(BALADI_NAMES)).toEqual(ids.baladi);
  expect(Object.keys(JAGRADADI_NAMES)).toEqual(ids.jagradadi);
  expect(Object.keys(LAJJITADI_NAMES)).toEqual(ids.lajjitadi);
  expect(Object.keys(SHAYANADI_NAMES)).toEqual(ids.shayanadi);
  const names = [BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES].flatMap((m) => Object.values(m));
  expect(new Set(names).size).toBe(5 + 3 + 6 + 12);
  expect(SHAYANADI_NAMES.NrityaLipsa).toBe("Nritya Lipsa");
  // Dizi adları transliterasyondur: üç dilde aynıdır (K11).
  for (const key of ["baladi", "jagradadi", "lajjitadi", "shayanadi"] as const) {
    expect([en.avastha[key], fr.avastha[key]]).toEqual([tr.avastha[key], tr.avastha[key]]);
  }
});
