import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeNavatara: vi.fn(), transitsAt: vi.fn() } }));

import { api, type Result } from "../api";
import type { Navatara } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, formatMoment } from "../transit/store";
import { TARA_NAMES } from "./labels";
import { NavataraPanel } from "./NavataraPanel";

const panel: Panel = { id: "nv", kind: "navatara", x: 0, y: 0, w: 7, h: 16, settings: {} };
const ok = (table: Navatara = fixture().natal_extras.navatara): Result<Navatara> => ({ status: "ok", data: table });
const NATAL = "Navatara: doğum nakshatrası Swati; dokuz tara, üç tur";
const natalTable = () => screen.findByRole("table", { name: NATAL });
const when = () => formatMoment(fixture().transit.utc, "tr", browserTimeZone());
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeNavatara).mockReset();
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok());
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "ok", data: fixture().transit }));
});

test("nine taras in three rounds from the birth nakshatra, with the natal grahas, the lord and the set", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const table = await natalTable();
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Tara", "1. tur", "2. tur", "3. tur", "Lord", "Küme"]);
  // Delhi 2000-01-01: Ay Swati'de (15). Türetme: tara n'nin üç nakshatrası 15 + (n − 1),
  // dokuz ve on sekiz sonrası (27'de sarılır); lord üçünde de aynıdır.
  expect(bodyRows(table)).toEqual([
    ["1 · Janma", "Swati (Ay)", "Shatabhisha", "Ardra", "Rahu", "elverişli kümede değil"],
    ["2 · Sampat", "Vishakha", "Purva Bhadrapada", "Punarvasu", "Jüpiter", "elverişli kümede"],
    ["3 · Vipat", "Anuradha (Ve)", "Uttara Bhadrapada", "Pushya (Ra)", "Satürn", "elverişli kümede değil"],
    ["4 · Kshema", "Jyeshtha", "Revati", "Ashlesha", "Merkür", "elverişli kümede"],
    ["5 · Pratyak", "Mula (Me)", "Ashwini (Jü)", "Magha", "Ketu", "elverişli kümede değil"],
    ["6 · Sadhana", "Purva Ashadha (Gü)", "Bharani (Sa)", "Purva Phalguni", "Venüs", "elverişli kümede"],
    ["7 · Naidhana", "Uttara Ashadha", "Krittika", "Uttara Phalguni", "Güneş", "elverişli kümede değil"],
    ["8 · Mitra", "Shravana (Ke)", "Rohini", "Hasta", "Ay", "elverişli kümede"],
    ["9 · Parama Mitra", "Dhanishta (Ma)", "Mrigashira", "Chitra", "Mars", "elverişli kümede"],
  ]);
  // Kısaltmayla yazılan graha tam adıyla okunur.
  expect(within(table).getAllByRole("button", { name: "Jüpiter" }).map((b) => b.textContent)).toEqual(["Jüpiter", "Jü"]);
  expect(screen.getByText(/Sayım doğum Ay'ının nakshatrasından başlar/)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: navatara.moon_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
});

test("the taras of the nine transit grahas at the transit moment, which is sent to Rust as it is", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const table = await screen.findByRole("table", { name: `Transit grahalarının tarası, ${when()}` });
  const opened = fixture().opened;
  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null };
  // İlk istek transit anı gelmeden (yalnızca doğum tablosu), ikincisi anla yapılır.
  expect(vi.mocked(api.computeNavatara).mock.calls).toEqual([
    [input, null],
    [input, fixture().transit.snapshot.jd_ut],
  ]);
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Nakshatra", "Tara", "Tur", "Küme"]);
  // Fikstürün transit anı (2026-11-26): Swati'den (15) sayım. Güneş Anuradha (17): 3 → Vipat;
  // Ay Rohini (4): 17 → 8, 2. tur; Mars, Jüpiter ve Ketu Magha (10): 23 → 5, 3. tur; Merkür
  // Vishakha (16): 2; Venüs Chitra (14): 27 → 9, 3. tur; Satürn Uttara Bhadrapada (26): 12 →
  // 3, 2. tur; Rahu Dhanishta (23): 9.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "Anuradha · pada 2", "3 · Vipat", "1", "elverişli kümede değil"],
    ["Ay", "Rohini · pada 4", "8 · Mitra", "2", "elverişli kümede"],
    ["Mars", "Magha · pada 2", "5 · Pratyak", "3", "elverişli kümede değil"],
    ["Merkür", "Vishakha · pada 1", "2 · Sampat", "1", "elverişli kümede"],
    ["Jüpiter", "Magha · pada 1", "5 · Pratyak", "3", "elverişli kümede değil"],
    ["Venüs", "Chitra · pada 3", "9 · Parama Mitra", "3", "elverişli kümede"],
    ["Satürn", "Uttara Bhadrapada · pada 4", "3 · Vipat", "2", "elverişli kümede değil"],
    ["Rahu", "Dhanishta · pada 3", "9 · Parama Mitra", "1", "elverişli kümede"],
    ["Ketu", "Magha · pada 1", "5 · Pratyak", "3", "elverişli kümede değil"],
  ]);
  // Transit anı panelden de seçilir.
  expect(screen.getByRole("form", { name: "Transit anı" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderWithStores(<NavataraPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeNavatara).not.toHaveBeenCalled();
  expect(api.transitsAt).not.toHaveBeenCalled();
});

test("unknown birth time: the table is shown with a notice", async () => {
  const table = fixture().natal_extras.navatara;
  table.approximate = true;
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok(table));
  renderWithStores(<NavataraPanel panel={panel} />, { opened: unknownTimeOpened() });
  await natalTable();
  expect(vi.mocked(api.computeNavatara).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay öğlen 12:00 konumundadır; Ay o gün nakshatra değiştiriyorsa bütün tablo bir nakshatra kayar.")).toBeInTheDocument();
});

test("shared highlight: a graha marks the tara rows it sits in or lords, and its transit row", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const natal = await natalTable();
  const transit = await screen.findByRole("table", { name: /^Transit grahalarının tarası,/ });
  const marked = (table: HTMLElement) => [...table.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.tara ?? (r as HTMLElement).dataset.graha);
  expect(marked(natal)).toEqual([]);
  // Venüs Anuradha'dadır (3. tara) ve 6. taranın lordudur.
  fireEvent.click(within(natal).getAllByRole("button", { name: "Venüs" })[0]);
  expect(marked(natal)).toEqual(["3", "6"]);
  expect(marked(transit)).toEqual(["Venus"]);
  fireEvent.click(within(transit).getByRole("button", { name: "Rahu" }));
  expect(marked(natal)).toEqual(["1", "3"]);
  expect(marked(transit)).toEqual(["Rahu"]);
});

test("a new transit moment is read again while the tables stay in place", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  await screen.findByRole("table", { name: /^Transit grahalarının tarası,/ });
  const later = fixture().transit;
  later.snapshot.jd_ut = (later.snapshot.jd_ut ?? 0) + 30;
  later.utc = "2026-12-26T01:30:00Z";
  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "ok", data: later }));
  let resolve: (r: Result<Navatara>) => void = () => {};
  vi.mocked(api.computeNavatara).mockImplementation(() => new Promise((r) => (resolve = r)));
  fireEvent.change(screen.getByLabelText("Tarih"), { target: { value: "2026-12-26" } });
  fireEvent.change(screen.getByLabelText("Saat"), { target: { value: "04:30" } });
  fireEvent.click(screen.getByRole("button", { name: "Uygula" }));
  await waitFor(() => expect(vi.mocked(api.computeNavatara).mock.calls.at(-1)?.[1]).toBe(later.snapshot.jd_ut));
  // Yanıt beklenirken iki tablo da durur; geçiş tablosu eski anın taralarını gösterdiği için
  // başlığında an yazmaz.
  expect(screen.getByRole("table", { name: NATAL })).toBeInTheDocument();
  expect(screen.getByRole("table", { name: "Transit grahalarının tarası" })).toBeInTheDocument();
  const next = fixture().natal_extras.navatara;
  next.transit_jd_ut = later.snapshot.jd_ut;
  next.transit![0].nakshatra = 20;
  await act(async () => resolve(ok(next)));
  const table = await screen.findByRole("table", { name: `Transit grahalarının tarası, ${formatMoment(later.utc, "tr", browserTimeZone())}` });
  expect(bodyRows(table)[0][1]).toBe("Purva Ashadha · pada 2");
});

test("errors: the report error is retried; a transit error leaves the natal table", async () => {
  vi.mocked(api.computeNavatara).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  vi.mocked(api.computeNavatara).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  const first = renderWithStores(<NavataraPanel panel={panel} />);
  const alert = await screen.findByRole("alert");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await natalTable();
  expect(screen.queryByRole("alert")).toBeNull();
  first.unmount();

  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "error", error: { code: "invalid_julian_day", params: { jd: "1" } } }));
  const natalOnly = fixture().natal_extras.navatara;
  natalOnly.transit = null;
  natalOnly.transit_jd_ut = null;
  vi.mocked(api.computeNavatara).mockClear();
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok(natalOnly));
  renderWithStores(<NavataraPanel panel={panel} />);
  await natalTable();
  expect(await screen.findByRole("alert")).toBeInTheDocument();
  expect(screen.queryByRole("table", { name: /^Transit grahalarının tarası/ })).toBeNull();
  expect(vi.mocked(api.computeNavatara).mock.calls.every((c) => c[1] === null)).toBe(true);
});

test("tara names stay the same in every language; headers and the set are translated", async () => {
  expect(TARA_NAMES).toHaveLength(9);
  await setLanguage("en");
  const first = renderWithStores(<NavataraPanel panel={panel} />);
  const table = await screen.findByRole("table", { name: "Navatara: birth nakshatra Swati; nine taras in three rounds" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Tara", "Round 1", "Round 2", "Round 3", "Lord", "Set"]);
  expect(bodyRows(table)[0]).toEqual(["1 · Janma", "Swati (Mo)", "Shatabhisha", "Ardra", "Rahu", "not in the favourable set"]);
  first.unmount();
  await setLanguage("fr");
  renderWithStores(<NavataraPanel panel={panel} />);
  const french = await screen.findByRole("table", { name: "Navatara : nakshatra de naissance Swati ; neuf taras en trois tours" });
  expect(bodyRows(french)[1]).toEqual(["2 · Sampat", "Vishakha", "Purva Bhadrapada", "Punarvasu", "Jupiter", "dans l’ensemble favorable"]);
});
