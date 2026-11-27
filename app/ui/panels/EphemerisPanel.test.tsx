import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { ephemerisMonth: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { EphemerisMonthView, MonthId } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { ephemerisFixture } from "../test/ephemerisFixture";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { EphemerisPanel, formatDay, formatMonth } from "./EphemerisPanel";
import { formatDegree } from "./format";

const EX = ephemerisFixture();
const ok = (view: EphemerisMonthView): Result<EphemerisMonthView> => ({ status: "ok", data: view });
const NOVEMBER: MonthId = { year: 2026, month: 11 };

let transit: Transit;
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <EphemerisPanel panel={active.panels[0]} />;
}
/** Varsayılan: yer haritanın doğum yeridir (fikstürde bulunulan yer seçilmemiştir). */
function renderPanel(settings: Record<string, unknown> = { place: "chart" }) {
  const panel: Panel = { id: "ep", kind: "ephemeris", x: 0, y: 0, w: 12, h: 22, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened: fixture().opened });
}
const rows = () => within(screen.getByRole("table")).getAllByRole("row").slice(1);
const row = (date: string) => document.querySelector<HTMLElement>(`tr[data-date="${date}"]`)!;
const cellOf = (date: string, graha: string) => row(date).querySelector<HTMLElement>(`td[data-graha="${graha}"]`)!;
const settle = () => act(async () => {});
const lastMonth = () => vi.mocked(api.ephemerisMonth).mock.calls.at(-1)![0];

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  // Tablo yeniden hesaplanmaz: istenen ay yanıtın `month` alanına yazılır.
  vi.mocked(api.ephemerisMonth).mockImplementation(async (month) => ok({ ...ephemerisFixture().month, month: month ?? NOVEMBER }));
});

test("without a place nothing is asked and the panel says how to pick one", () => {
  renderPanel({});
  expect(screen.getByRole("note")).toHaveTextContent(/Bulunulan yer seçilmedi/);
  expect(api.ephemerisMonth).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("this month's table is asked with the place and the chart's settings; one row a day, nine grahas", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: /^Kasım 2026: her günün başında/ });
  const opened = fixture().opened;
  const { lat, lon } = opened.chart_record.birth.place;
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
  // Ay boştur: "bu ay" Rust'ta okunur.
  expect(vi.mocked(api.ephemerisMonth).mock.calls[0]).toEqual([null, { lat, lon, tz: opened.chart_record.birth.tz }, opened.chart.meta.settings]);
  // Sütun başlıkları kısaltmadır; ekran okuyucu tam adı okur.
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Gün", "GüGüneş", "AyAy", "MaMars", "MeMerkür", "JüJüpiter", "VeVenüs", "SaSatürn", "RaRahu", "KeKetu"]);
  expect(rows()).toHaveLength(30);
  // 1 Kasım 2026 Pazar'dır.
  const first = EX.month.days[0];
  expect(first.date).toBe("2026-11-01");
  expect(within(rows()[0]).getByRole("rowheader")).toHaveTextContent("01 Paz");
  // Güneş Terazi'dedir; derece Rust'ın değeridir, yalnızca biçimlenir.
  const sun = first.row.cells[0];
  expect([sun.graha, sun.sign, sun.retrograde]).toEqual(["Sun", "Libra", false]);
  expect(cellOf("2026-11-01", "Sun")).toHaveTextContent(new RegExp(`^Ter ${formatDegree(sun.deg_in_sign!)}$`));
  // Merkür o gün retrodur: düz "R" ve ekran okuyucu için sözü. Rahu ve Ketu'da "R" yazılmaz.
  expect(first.row.cells[3].retrograde && first.row.cells[7].retrograde).toBe(true);
  expect(cellOf("2026-11-01", "Mercury")).toHaveTextContent(/ Rretro$/);
  expect(cellOf("2026-11-01", "Rahu")).not.toHaveTextContent(/R/);
  // Nakshatra kutusu kapalıdır.
  expect(document.querySelector(".ephemeris-nakshatra")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu ay" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText(/Yöntem: ephemeris_table\.v1/)).toBeInTheDocument();
  expect(screen.getByText(/→ burç: graha o gün içinde bu burca girer\. SR: o gün retroya döner; SD: o gün düze döner/)).toBeInTheDocument();
});

test("ingress and station marks are symbols with words, on the rows Rust marked", async () => {
  renderPanel();
  await screen.findByRole("table");
  // Fikstürde Ay dışındaki girişler: 6 Kasım Venüs Başak, 12 Kasım Mars Aslan, 16 Kasım Güneş
  // Akrep, 22 Kasım Venüs Terazi; dönüşler: 13 Kasım Merkür ve 14 Kasım Venüs düze döner.
  const marked = (kind: "ingress" | "station") =>
    EX.month.days.flatMap((d) => d.row.cells.filter((c) => c.graha !== "Moon" && c[kind] !== null).map((c) => `${d.date} ${c.graha} ${c[kind]}`));
  expect(marked("ingress")).toEqual(["2026-11-06 Venus Virgo", "2026-11-12 Mars Leo", "2026-11-16 Sun Scorpio", "2026-11-22 Venus Libra"]);
  expect(marked("station")).toEqual(["2026-11-13 Mercury Direct", "2026-11-14 Venus Direct"]);
  const mark = (date: string, graha: string, kind: string) => cellOf(date, graha).querySelector(`[data-mark="${kind}"]`);
  expect(mark("2026-11-16", "Sun", "ingress")).toHaveTextContent("→ Akro gün Akr burcuna girer");
  expect(mark("2026-11-12", "Mars", "ingress")).toHaveTextContent("→ Aslo gün Asl burcuna girer");
  expect(mark("2026-11-13", "Mercury", "station")).toHaveTextContent("SDo gün düze döner");
  expect(mark("2026-11-15", "Sun", "ingress")).toBeNull();
  expect(mark("2026-11-13", "Sun", "station")).toBeNull();
  // İşaret sayısı Rust'ın işaretlediği hücre sayısıdır (Ay dahil).
  const count = (kind: "ingress" | "station") => EX.month.days.reduce((n, d) => n + d.row.cells.filter((c) => c[kind] !== null).length, 0);
  expect(document.querySelectorAll('[data-mark="ingress"]')).toHaveLength(count("ingress"));
  expect(document.querySelectorAll('[data-mark="station"]')).toHaveLength(2);
  // Retroya dönüş işareti: sözü ve simgesi.
  const view = ephemerisFixture().month;
  view.days[2].row.cells[2].station = "Retrograde";
  vi.mocked(api.ephemerisMonth).mockResolvedValue(ok(view));
  fireEvent.click(screen.getByRole("button", { name: "Bu ay" }));
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await waitFor(() => expect(mark("2026-11-03", "Mars", "station")).toHaveTextContent("SRo gün retroya döner"));
});

test("the nakshatra box adds the nakshatra and pada to every cell and is saved with the layout", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table");
  fireEvent.click(screen.getByRole("checkbox", { name: "Nakshatra" }));
  await waitFor(() => expect(document.querySelectorAll(".ephemeris-nakshatra")).toHaveLength(30 * 9));
  // 1 Kasım: Güneş 15. nakshatrada (Swati), 3. padada.
  const sun = EX.month.days[0].row.cells[0];
  expect([sun.nakshatra, sun.pada]).toEqual([15, 3]);
  expect(cellOf("2026-11-01", "Sun").querySelector(".ephemeris-nakshatra")).toHaveTextContent(/^Swati 3$/);
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ place: "chart", nakshatra: true });
  // Kutu yeni bir istek başlatmaz: tablo aynıdır.
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
});

test("previous, next and this month use the months Rust returned; a chosen month is asked on submit", async () => {
  renderPanel();
  await screen.findByRole("table");
  expect([EX.month.prev, EX.month.next]).toEqual([
    { year: 2026, month: 10 },
    { year: 2026, month: 12 },
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await screen.findByRole("table", { name: /^Aralık 2026:/ });
  expect(lastMonth()).toEqual({ year: 2026, month: 12 });
  expect(screen.getByRole("button", { name: "Bu ay" })).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(screen.getByRole("button", { name: "Önceki ay" }));
  await settle();
  expect(lastMonth()).toEqual({ year: 2026, month: 10 });
  // Ay ve yıl kutuları gösterilen ayı izler; seçim "Ayı göster" ile gider.
  expect(screen.getByRole("combobox", { name: "Ay" })).toHaveDisplayValue("Ekim");
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveValue(2026);
  fireEvent.change(screen.getByRole("combobox", { name: "Ay" }), { target: { value: "3" } });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Yıl" }), { target: { value: "2030" } });
  const asked = vi.mocked(api.ephemerisMonth).mock.calls.length;
  fireEvent.click(screen.getByRole("button", { name: "Ayı göster" }));
  await screen.findByRole("table", { name: /^Mart 2030:/ });
  expect(vi.mocked(api.ephemerisMonth).mock.calls).toHaveLength(asked + 1);
  expect(lastMonth()).toEqual({ year: 2030, month: 3 });
  // Geçersiz yıl gönderilmez.
  fireEvent.change(screen.getByRole("spinbutton", { name: "Yıl" }), { target: { value: "3000" } });
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveAttribute("aria-invalid", "true");
  fireEvent.click(screen.getByRole("button", { name: "Ayı göster" }));
  await settle();
  expect(vi.mocked(api.ephemerisMonth).mock.calls).toHaveLength(asked + 1);
  fireEvent.click(screen.getByRole("button", { name: "Bu ay" }));
  await screen.findByRole("table", { name: /^Kasım 2026:/ });
  expect(lastMonth()).toBeNull();
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveValue(2026);
});

test("at the end of the range the step button stays focusable and does nothing", async () => {
  vi.mocked(api.ephemerisMonth).mockResolvedValue(ok({ ...ephemerisFixture().month, month: { year: 1, month: 1 }, prev: null }));
  renderPanel();
  await screen.findByRole("table");
  const prev = screen.getByRole("button", { name: "Önceki ay" });
  expect(prev).toHaveAttribute("aria-disabled", "true");
  expect(prev).not.toBeDisabled();
  fireEvent.click(prev);
  await settle();
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Sonraki ay" })).not.toHaveAttribute("aria-disabled");
});

test("the table stays while another month is read; a day moves the transit date", async () => {
  renderPanel();
  await screen.findByRole("table");
  let resolve!: (r: Result<EphemerisMonthView>) => void;
  vi.mocked(api.ephemerisMonth).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await settle();
  expect(screen.getByRole("table", { name: /^Kasım 2026:/ })).toBeInTheDocument();
  await act(async () => resolve(ok({ ...ephemerisFixture().month, month: { year: 2026, month: 12 } })));
  expect(screen.getByRole("table", { name: /^Aralık 2026:/ })).toBeInTheDocument();
  const day = within(rows()[5]).getByRole("button");
  // Erişilebilir ad görünen yazıyla başlar (WCAG 2.5.3).
  expect(day).toHaveTextContent("06 Cum");
  expect(day).toHaveAccessibleName("06 Cum (2026-11-06): transit tarihini bu günün başına götür");
  fireEvent.click(day);
  expect(transit.at).toEqual({ Jd: EX.month.days[5].row.jd_ut });
});

test("the fallback ephemeris is written out; an input error names the field and can be retried", async () => {
  vi.mocked(api.ephemerisMonth).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "month" } } });
  vi.mocked(api.ephemerisMonth).mockResolvedValueOnce(ok({ ...ephemerisFixture().month, warnings: ["LowPrecision"] }));
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: ay.");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(screen.getByRole("note")).toHaveTextContent(/^Düşük hassasiyet: /);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("months and days are written in the interface language", () => {
  expect([formatMonth(NOVEMBER, "tr"), formatMonth(NOVEMBER, "en"), formatMonth(NOVEMBER, "fr")]).toEqual(["Kasım 2026", "November 2026", "novembre 2026"]);
  // İki basamaklı yıl 1900'lere taşınmaz.
  expect(formatMonth({ year: 50, month: 1 }, "en")).toBe("January 50");
  expect(formatDay("2026-11-01", "tr")).toBe("01 Paz");
  expect(formatDay("2026-11-16", "tr")).toBe("16 Pzt");
  expect(formatDay("2026-11-01", "en")).toMatch(/Sun/);
  expect(formatDay("bozuk", "tr")).toBe("bozuk");
});

test("the texts exist in the three languages; French uses the typographic apostrophe", () => {
  expect([tr.panel.ephemeris, en.panel.ephemeris, fr.panel.ephemeris]).toEqual(["Efemeris", "Ephemeris", "Éphémérides"]);
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.ephemeris).sort()).toEqual(Object.keys(tr.ephemeris).sort());
    expect(Object.keys(m.ephemeris.station).sort()).toEqual(["Direct", "Retrograde"]);
    expect(m.ephemeris.legend).toContain("{{ingress}}");
    expect(m.ephemeris.legend).toContain("{{station}}");
  }
  expect(JSON.stringify(fr.ephemeris)).not.toContain("'");
  expect(fr.ephemeris.legend).toContain("recherche d’événements");
});
