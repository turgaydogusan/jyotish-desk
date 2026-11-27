import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { sankrantiList: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { OpenedClient, SankrantiListView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { sankrantiFixture } from "../test/sankrantiFixture";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { parseYear, SankrantiPanel, YEAR_MAX, YEAR_MIN } from "./SankrantiPanel";

const SX = sankrantiFixture();
const ok = (view: SankrantiListView): Result<SankrantiListView> => ({ status: "ok", data: view });

let transit: Transit;
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <SankrantiPanel panel={active.panels[0]} />;
}
/** Varsayılan: yer haritanın doğum yeridir (fikstürde bulunulan yer seçilmemiştir). */
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "sk", kind: "sankranti", x: 0, y: 0, w: 7, h: 16, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened });
}
const rows = () => within(screen.getByRole("table")).getAllByRole("row").slice(1);
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const yearBox = () => screen.getByRole("spinbutton", { name: "Yıl" });
const settle = () => act(async () => {});

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.sankrantiList).mockImplementation(async (year) => ok({ ...sankrantiFixture().list, year: year ?? 2026 }));
});

test("without a place nothing is asked and the panel says how to pick one", () => {
  renderPanel({});
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveDisplayValue("Bulunulan yer");
  expect(screen.getByRole("note")).toHaveTextContent(/Bulunulan yer seçilmedi/);
  expect(api.sankrantiList).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("this year's list is asked with the place and the chart's settings; twelve rows in the place's time", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" });
  const opened = fixture().opened;
  const { lat, lon } = opened.chart_record.birth.place;
  expect(api.sankrantiList).toHaveBeenCalledTimes(1);
  // Yıl boştur: "bu yıl" Rust'ta okunur. Ayar haritanın hesap ayarıdır.
  expect(vi.mocked(api.sankrantiList).mock.calls[0]).toEqual([null, { lat, lon, tz: opened.chart_record.birth.tz }, opened.chart.meta.settings]);
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Giriş anı", "Girilen burç", "Tithi", "Vara", "Ay nakshatrası"]);
  expect(rows()).toHaveLength(12);
  // İlk satır (fikstür, Delhi +05:30): Oğlak girişi 2026-01-14 15:07. Tithi 26, Krishna
  // pakshanın 26 − 15 = 11. tithisi (Ekadashi); 17. nakshatra Anuradha, 3. pada; Çarşamba.
  const first = SX.list.rows[0];
  expect([first.sankranti.sign, first.sankranti.tithi, first.sankranti.nakshatra, first.sankranti.pada, first.sankranti.vara, first.offset_secs]).toEqual(["Capricorn", 26, 17, 3, "Wednesday", 19_800]);
  expect(cells(rows()[0])).toEqual(["2026-01-14 15:07", "Oğl", "Krishna 11 · Ekadashi", "Budhavara · Çarşamba", "Anuradha · pada 3"]);
  // Burç sırası: Oğlak'tan Yay'a.
  expect(rows().map((r) => cells(r)[1])).toEqual(["Oğl", "Kov", "Bal", "Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay"]);
  expect(yearBox()).toHaveValue(2026);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText(/Yöntem: sankranti\.sidereal_v1/)).toBeInTheDocument();
  // Harita sidereal: çerçeve notu yok; uyarı yok.
  expect(screen.queryByRole("note")).toBeNull();
});

test("a typed year is asked on submit; an invalid year is not sent", async () => {
  renderPanel();
  await screen.findByRole("table");
  fireEvent.change(yearBox(), { target: { value: "2027" } });
  // Yazarken istek gitmez.
  expect(api.sankrantiList).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  await screen.findByRole("table", { name: "Sankranti listesi, 2027: 12 giriş" });
  expect(vi.mocked(api.sankrantiList).mock.calls.at(-1)![0]).toBe(2027);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "false");
  for (const bad of ["0", "3000", "", "20.5"]) {
    fireEvent.change(yearBox(), { target: { value: bad } });
    expect(yearBox()).toHaveAttribute("aria-invalid", "true");
    fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  }
  await settle();
  expect(api.sankrantiList).toHaveBeenCalledTimes(2);
  // "Bu yıl" yılı boşaltır ve kutu Rust'ın döndürdüğü yılı gösterir.
  fireEvent.click(screen.getByRole("button", { name: "Bu yıl" }));
  await screen.findByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" });
  expect(vi.mocked(api.sankrantiList).mock.calls.at(-1)![0]).toBeNull();
  expect(yearBox()).toHaveValue(2026);
  expect(yearBox()).not.toHaveAttribute("aria-invalid");
});

test("the table stays while another year is read", async () => {
  renderPanel();
  await screen.findByRole("table");
  let resolve!: (r: Result<SankrantiListView>) => void;
  vi.mocked(api.sankrantiList).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.change(yearBox(), { target: { value: "2030" } });
  fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  await settle();
  expect(screen.getByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" })).toBeInTheDocument();
  await act(async () => resolve(ok({ ...sankrantiFixture().list, year: 2030 })));
  expect(screen.getByRole("table", { name: "Sankranti listesi, 2030: 12 giriş" })).toBeInTheDocument();
});

test("the moment in a row moves the transit date", async () => {
  renderPanel();
  await screen.findByRole("table");
  const button = within(rows()[3]).getByRole("button");
  expect(button).toHaveAccessibleName(/transit tarihini bu ana götür/);
  fireEvent.click(button);
  expect(transit.at).toEqual({ Jd: SX.list.rows[3].sankranti.jd_ut });
});

test("warnings are written out: a day without sunrise and the fallback ephemeris", async () => {
  vi.mocked(api.sankrantiList).mockResolvedValue(ok({ ...sankrantiFixture().list_polar, warnings: ["LowPrecision", "SunriseUnavailable"] }));
  renderPanel();
  await screen.findByRole("table");
  expect(SX.list_polar.warnings).toEqual(["SunriseUnavailable"]);
  const notes = screen.getAllByRole("note").map((n) => n.textContent);
  expect(notes).toEqual([expect.stringMatching(/^Düşük hassasiyet: /), expect.stringMatching(/^Gün doğumu yok: Girişlerden en az birinin gününde bu yerde gün doğmuyor/)]);
});

test("a chart with tropical signs is told that the list is sidereal", async () => {
  const opened = fixture().opened;
  opened.chart.meta.settings.zodiac = "TropicalRasiSiderealNakshatra";
  renderPanel({ place: "chart" }, opened);
  await screen.findByRole("table");
  expect(screen.getByRole("note")).toHaveTextContent("Harita tropikal burç kullanıyor; bu liste her durumda seçili ayanamsayla sidereal girişleri verir.");
  expect(vi.mocked(api.sankrantiList).mock.calls[0][2].zodiac).toBe("TropicalRasiSiderealNakshatra");
});

test("an input error names the field in the interface language and can be retried", async () => {
  vi.mocked(api.sankrantiList).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "year" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: yıl.");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(screen.getByRole("table")).toBeInTheDocument());
  expect(screen.queryByRole("alert")).toBeNull();
});

test("the year box accepts whole years in the range only", () => {
  expect([YEAR_MIN, YEAR_MAX]).toEqual([1, 2999]);
  expect(["1", "2026", " 2999 ", "0001"].map(parseYear)).toEqual([1, 2026, 2999, 1]);
  expect(["0", "3000", "", "20.5", "-5", "2e3", "12345", "yıl"].map(parseYear)).toEqual([null, null, null, null, null, null, null, null]);
});

test("the texts exist in the three languages; French uses the typographic apostrophe", () => {
  expect([tr.panel.sankranti, en.panel.sankranti, fr.panel.sankranti]).toEqual(["Sankranti", "Sankranti", "Sankranti"]);
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.sankranti).sort()).toEqual(Object.keys(tr.sankranti).sort());
    expect(m.sankranti.method).toContain("{{method}}");
    expect(m.sankranti.caption).toContain("{{year}}");
  }
  expect(JSON.stringify(fr.sankranti)).not.toContain("'");
  expect(fr.sankranti.show).toBe("Afficher l’année");
});
