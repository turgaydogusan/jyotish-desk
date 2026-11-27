import { act, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { monthCalendar: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, CalendarRequest, CalendarView } from "../bindings";
import { setLanguage } from "../i18n";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { PANEL_REGISTRY } from "../panels/registry";
import { calendarFixture } from "../test/calendarFixture";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { CalendarPanel } from "./CalendarPanel";

const FX = fixture();
const CAL = calendarFixture();
const ok = (view: CalendarView): Result<CalendarView> => ({ status: "ok", data: view });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "cal", kind: "calendar", x: 0, y: 0, w: 9, h: 24, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const withHome = (): AppPrefs => ({ ...FX.prefs.prefs, home_place: FX.muhurta.home_place });

let transit: Transit;
let kinds: string[] = [];
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  kinds = active.panels.map((p) => p.kind);
  return <CalendarPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = {}, { opened = fixture().opened, prefs = withHome() }: Options = {}) {
  // Pano çerçevesi gibi: hata kutusu kalkınca odak panelin çerçevesine taşınır.
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Transit takvimi">
      <LivePanel />
    </section>,
    { layout: LAYOUT(settings), opened, prefs },
  );
}

/** Fikstürün ayları, isteğin ayına ve süzgecine göre (Rust'ın yerine). */
function answer(request: CalendarRequest): Result<CalendarView> {
  const { month } = request;
  if (month !== null && month.year === 2026 && month.month === 12) return ok(calendarFixture().month_next);
  if (month !== null && month.year === 2027 && month.month === 1) return ok(calendarFixture().month_polar);
  if (month !== null && (month.year !== 2026 || month.month !== 11)) return { status: "error", error: { code: "invalid_calendar_input", params: { field: "month" } } };
  return ok(request.filter.moon || request.filter.kinds.includes("TransitToNatal") ? calendarFixture().month_full : calendarFixture().month);
}

const grid = (name = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları") => screen.findByRole("grid", { name });
const DECEMBER = "Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";
const key = (td: HTMLElement, k: string, init: Record<string, unknown> = {}) => fireEvent.keyDown(td, { key: k, ...init });
const focused = () => (document.activeElement as HTMLElement | null)?.getAttribute("data-date");
const detail = () => screen.getByRole("region", { name: /^\d+ \S+ \d+ \S+$/ });
const rowsOf = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map((r) => [...r.querySelectorAll("th, td")].map((c) => c.textContent));
const cell = (table: HTMLElement, date: string) => table.querySelector<HTMLElement>(`td[data-date="${date}"]`)!;
const lines = (td: HTMLElement) => [...td.children].map((c) => c.textContent);
const events = (td: HTMLElement) => [...td.querySelectorAll("li")].map((li) => li.textContent);
const requests = () => vi.mocked(api.monthCalendar).mock.calls.map(([r]) => r);

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.monthCalendar).mockImplementation(async (request) => answer(request));
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

afterEach(async () => {
  await act(async () => {
    await setLanguage("tr");
  });
});

test("asks for this month at the current place and draws one cell per calendar day", async () => {
  renderPanel();
  const table = await grid();
  // Bir istek: ay verilmez ("bugün" Rust'ta okunur), yerin koordinatı ve dilimi (ad gitmez),
  // varsayılan süzgeç, haritanın hesap ayarı; doğum verisi gitmez.
  expect(requests()).toEqual([{ month: null, place: FX.muhurta.place, filter: CAL.request.filter, settings: FX.opened.chart.meta.settings, natal: null }]);
  expect(requests()[0]).toEqual(CAL.request);
  expect(screen.getByText("New Delhi, Delhi, India")).toBeInTheDocument();

  // Türkçede hafta Pazartesi başlar; başlık kısaltmadır, tam ad ekran okuyucuya verilir.
  const heads = within(table).getAllByRole("columnheader");
  expect(heads.map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]);
  expect(heads.map((h) => h.getAttribute("abbr"))).toEqual(["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi", "Pazar"]);
  // 1 Kasım 2026 Pazar'dır: altı hafta satırı, otuz gün, on iki boş hücre.
  expect(table.querySelectorAll("tbody tr")).toHaveLength(6);
  expect(table.querySelectorAll("td[data-date]")).toHaveLength(30);
  expect(table.querySelectorAll("td.calendar-empty")).toHaveLength(12);
  expect([...table.querySelectorAll("tbody tr:first-child td")].map((td) => td.getAttribute("data-date"))).toEqual([null, null, null, null, null, null, "2026-11-01"]);

  // Hücre: ayın günü, gün doğumundaki tithi (kısa yazım; tam adı ekran okuyucuya) ve Ay nakshatrası.
  const first = cell(table, "2026-11-01");
  expect(lines(first)).toEqual(["1", "K7 SaptamiKrishna 7 · Saptami", "Pushya"]);
  expect(first.querySelector("ul")).toBeNull();
  // Yeni ay günü: gün doğumunda Amavasya; olay saatiyle yazılır.
  const amavasya = cell(table, "2026-11-09");
  expect(lines(amavasya).slice(0, 3)).toEqual(["9", "K15 AmavasyaKrishna 15 · Amavasya", "Swati"]);
  expect(events(amavasya)).toEqual(["12:32 Yeni ayYeni ay · Ter · Vishakha"]);
  // Girişler ve dönüşler: kısa yazı görünür, tam ad ekran okuyucuya verilir.
  expect(events(cell(table, "2026-11-06"))).toEqual(["01:04 Ve → BaşVenüs: burç girişi · Baş", "21:03 Gü → VishakhaGüneş: nakshatra girişi · Vishakha"]);
  expect(events(cell(table, "2026-11-13"))).toEqual(["21:24 Me R biterMerkür: R bitiyor · Ter"]);
  expect(events(cell(table, "2026-11-16"))).toEqual(["19:43 Gü → AkrGüneş: burç girişi · Akr"]);
  expect(cell(table, "2026-11-16").querySelector("li")).toHaveAttribute("data-event", "SignIngress");
  const short = cell(table, "2026-11-16").querySelector('li [aria-hidden="true"]');
  expect(short).toHaveTextContent("Gü → Akr");
  expect(lines(cell(table, "2026-11-24")).slice(1, 3)).toEqual(["S15 PurnimaShukla 15 · Purnima", "Krittika"]);

  // Bugün (Rust'ın verdiği gün) sözle ve çerçeveyle işaretlidir; yalnızca bir hücre.
  const today = cell(table, "2026-11-25");
  expect(lines(today)).toEqual(["25bugün", "K1 PratipadaKrishna 1 · Pratipada", "Rohini", "13:41 Me → VishakhaMerkür: nakshatra girişi · Vishakha"]);
  expect(table.querySelectorAll("td.is-today")).toHaveLength(1);
  expect(today).toHaveClass("is-today");

  expect(screen.getByText(/^Yöntem: calendar\.month_v1\. Hücre takvim günüdür/)).toBeInTheDocument();
});

test("a cell lists at most three events and counts the rest", async () => {
  renderPanel({ kinds: CAL.ids.kinds, moon: true });
  const table = await grid();
  expect(requests()[0]).toEqual(CAL.request_full);
  // 24 Kasım: beş olay (Ay'ın iki nakshatra girişi, burç girişi, doğum boylamına geliş, dolunay).
  expect(events(cell(table, "2026-11-24"))).toEqual([
    "02:02 Ay → KrittikaAy: nakshatra girişi · Krittika",
    "07:24 Ay → BoğAy: burç girişi · Boğ",
    "11:09 Gü → doğum VeGüneş, doğum Venüs boylamında · Akr",
    "+2 olay",
  ]);
  // Üç olaylı günde sayaç yoktur.
  expect(events(cell(table, "2026-11-25"))).toHaveLength(3);
  expect(cell(table, "2026-11-25").querySelector(".calendar-more")).toBeNull();
});

test("the month is stepped with Rust's previous and next months, chosen by month and year, and brought back with This month", async () => {
  renderPanel();
  await grid();
  const nav = screen.getByRole("group", { name: "Ay seçimi" });
  const month = within(nav).getByRole("combobox", { name: "Ay" }) as HTMLSelectElement;
  // Yıl alanı gösterilen yıl değişince yeniden kurulur: her seferinde yeniden bulunur.
  const year = () => within(nav).getByRole("textbox", { name: "Yıl" }) as HTMLInputElement;
  expect([month.value, year().value]).toEqual(["11", "2026"]);
  expect([...month.options].map((o) => o.textContent)).toEqual(["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"]);

  // Sonraki ay: yanıttaki `next` istenir; arayüz ay hesabı yapmaz.
  await act(async () => within(nav).getByRole("button", { name: "Sonraki ay" }).click());
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 12 });
  const december = await grid(DECEMBER);
  expect(december.querySelectorAll("td[data-date]")).toHaveLength(31);
  // Başka bir ayda "bugün" yoktur.
  expect(december.querySelectorAll("td.is-today")).toHaveLength(0);
  expect([month.value, year().value]).toEqual(["12", "2026"]);
  // Aynı gün ve saatteki olaylar ayrı satırlardır (düğümlerin burç girişi).
  expect(events(cell(december, "2026-12-05")).slice(0, 2)).toEqual(["22:32 Ra → OğlRahu: burç girişi · Oğl", "22:32 Ke → YenKetu: burç girişi · Yen"]);

  // Ay alanı: gösterilen yılın seçilen ayı istenir.
  await act(async () => fireEvent.change(month, { target: { value: "1" } }));
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 1 });
  await act(async () => {});
  // Bu testte o ayın yanıtı hatadır: alan adı arayüz dilinde yazılır ve ay seçimi istenen ayı
  // göstererek kullanılabilir durur.
  expect(screen.getByRole("alert")).toHaveTextContent("Takvim isteği geçersiz: ay (yıl 1–2999).");
  expect([month.value, month.disabled, year().value, year().disabled]).toEqual(["1", false, "2026", false]);
  await act(async () => within(nav).getByRole("button", { name: "Bu ay" }).click());
  expect(requests().at(-1)?.month).toBeNull();
  await grid();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(month.value).toBe("11");

  // Yıl: geçerli değer Enter ile istenir; aralık dışı değer uygulanmaz ve gösterilen yıla döner.
  const before = requests().length;
  fireEvent.change(year(), { target: { value: "3000" } });
  expect(year()).toHaveAttribute("aria-invalid", "true");
  fireEvent.blur(year());
  expect(year().value).toBe("2026");
  expect(year()).not.toHaveAttribute("aria-invalid");
  fireEvent.change(year(), { target: { value: "20x6" } });
  await act(async () => fireEvent.submit(year().closest("form")!));
  expect(year().value).toBe("2026");
  expect(requests()).toHaveLength(before);

  // Önceki ay.
  await act(async () => within(nav).getByRole("button", { name: "Önceki ay" }).click());
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 10 });
});

test("This month reads again even when the month is already the current one", async () => {
  renderPanel();
  await grid();
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
  await act(async () => screen.getByRole("button", { name: "Bu ay" }).click());
  // Aynı istek yeniden gider: gece yarısından sonra Rust'ın "bugün"ü değişmiş olabilir.
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);
  expect(requests()[1]).toEqual(requests()[0]);
  await grid();
});

test("the year field asks for the typed year in the shown month", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async (request) => (request.month === null ? answer(request) : ok({ ...calendarFixture().month_next, month: request.month, today: null })));
  renderPanel();
  await grid();
  const year = screen.getByRole("textbox", { name: "Yıl" }) as HTMLInputElement;
  fireEvent.change(year, { target: { value: "2031" } });
  await act(async () => fireEvent.submit(year.closest("form")!));
  expect(requests().at(-1)?.month).toEqual({ year: 2031, month: 11 });
  await grid("Kasım 2031 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  expect((screen.getByRole("textbox", { name: "Yıl" }) as HTMLInputElement).value).toBe("2031");
});

test("the grid stays while the next month loads and the range ends disable a step", async () => {
  let release!: (r: Result<CalendarView>) => void;
  renderPanel();
  const table = await grid();
  vi.mocked(api.monthCalendar).mockImplementationOnce(() => new Promise((r) => (release = r)));
  await act(async () => screen.getByRole("button", { name: "Sonraki ay" }).click());
  // Yeni ay gelene dek eski ızgara durur ve panel meşgul işaretlidir.
  expect(table).toBeInTheDocument();
  expect(table.closest(".calendar")).toHaveAttribute("aria-busy", "true");
  // Aralığın ucunda sonraki ay yoktur: düğme odaklanır ama bir şey istemez.
  await act(async () => release(ok({ ...calendarFixture().month_next, next: null })));
  expect(screen.getByRole("grid").closest(".calendar")).not.toHaveAttribute("aria-busy");
  const next = screen.getByRole("button", { name: "Sonraki ay" });
  expect(next).toHaveAttribute("aria-disabled", "true");
  expect(next).not.toBeDisabled();
  const before = requests().length;
  await act(async () => next.click());
  expect(requests()).toHaveLength(before);
});

test("the grid is one tab stop; arrows, Home, End and their Ctrl forms move the focus between the days", async () => {
  renderPanel();
  const table = await grid();
  // Gezici sekme durağı: yalnızca bir hücre sekmeyle odaklanır (bugün); kaydırma kabı durak değildir.
  expect([...table.querySelectorAll('td[tabindex="0"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-25"]);
  expect(table.querySelectorAll('td[tabindex="-1"]')).toHaveLength(29);
  expect(table.closest(".calendar-scroll")).not.toHaveAttribute("tabindex");
  expect(table).toHaveAccessibleDescription("Ok tuşları günler arasında gezer; Enter günü seçer; Page Up ve Page Down ayı değiştirir.");
  const at = (date: string) => cell(table, date);
  at("2026-11-25").focus();
  // 25 Kasım Çarşamba'dır (Pazartesi başlayan haftanın üçüncü sütunu).
  key(at("2026-11-25"), "ArrowRight");
  expect(focused()).toBe("2026-11-26");
  key(at("2026-11-26"), "ArrowLeft");
  key(at("2026-11-25"), "ArrowLeft");
  expect(focused()).toBe("2026-11-24");
  key(at("2026-11-24"), "ArrowUp");
  expect(focused()).toBe("2026-11-17");
  key(at("2026-11-17"), "ArrowDown");
  key(at("2026-11-24"), "ArrowDown");
  // Son satırda Salı sütunu boştur: odak yerinde durur.
  expect(focused()).toBe("2026-11-24");
  key(at("2026-11-24"), "Home");
  expect(focused()).toBe("2026-11-23");
  key(at("2026-11-23"), "End");
  expect(focused()).toBe("2026-11-29");
  key(at("2026-11-29"), "Home", { ctrlKey: true });
  expect(focused()).toBe("2026-11-01");
  // Ayın ilk günü ilk satırın tek dolu hücresidir: sola ve yukarı gidilmez; Home ve End aynı gündür.
  key(at("2026-11-01"), "ArrowLeft");
  key(at("2026-11-01"), "ArrowUp");
  key(at("2026-11-01"), "Home");
  key(at("2026-11-01"), "End");
  expect(focused()).toBe("2026-11-01");
  key(at("2026-11-01"), "ArrowDown");
  expect(focused()).toBe("2026-11-08");
  key(at("2026-11-08"), "End", { metaKey: true });
  expect(focused()).toBe("2026-11-30");
  key(at("2026-11-30"), "ArrowRight");
  expect(focused()).toBe("2026-11-30");
  // Sekme durağı odağı izler; gezinme seçimi ve isteği değiştirmez.
  expect([...table.querySelectorAll('td[tabindex="0"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-30"]);
  expect([...table.querySelectorAll('td[aria-selected="true"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-25"]);
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
});

test("the detail shows today until a day is chosen with Enter, Space or a click", async () => {
  renderPanel();
  const table = await grid();
  // Seçim yokken ayrıntı bugünündür ve bugünün hücresi seçili işaretlidir.
  expect(detail()).toHaveAccessibleName("25 Kasım 2026 Çarşamba");
  expect(within(detail()).getByText("Gün doğumu 06:55 · Krishna 1 · Pratipada · Rohini")).toBeInTheDocument();
  const eventsTable = within(detail()).getByRole("table", { name: "25 Kasım 2026 Çarşamba: günün olayları" });
  expect(rowsOf(eventsTable)).toEqual([["13:41", "Merkür: nakshatra girişi", "Vishakha"]]);
  expect(cell(table, "2026-11-25")).toHaveClass("is-selected");

  // Enter: 6 Kasım (iki olay).
  cell(table, "2026-11-06").focus();
  key(cell(table, "2026-11-06"), "Enter");
  expect(detail()).toHaveAccessibleName("6 Kasım 2026 Cuma");
  expect(detail()).toHaveAttribute("data-detail", "2026-11-06");
  expect(within(detail()).getByText("Gün doğumu 06:40 · Krishna 12 · Dwadashi · Hasta")).toBeInTheDocument();
  expect(rowsOf(within(detail()).getByRole("table"))).toEqual([
    ["01:04", "Venüs: burç girişi", "Baş"],
    ["21:03", "Güneş: nakshatra girişi", "Vishakha"],
  ]);
  expect([...table.querySelectorAll('td[aria-selected="true"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-06"]);
  expect(cell(table, "2026-11-25")).not.toHaveClass("is-selected");
  // Başlık duyurulur.
  expect(within(detail()).getByText("6 Kasım 2026 Cuma")).toHaveAttribute("aria-live", "polite");

  // Boşluk: olaysız bir gün.
  key(cell(table, "2026-11-01"), " ");
  expect(detail()).toHaveAccessibleName("1 Kasım 2026 Pazar");
  expect(within(detail()).getByText("Seçili türlerde bu güne düşen olay yok.")).toBeInTheDocument();
  expect(within(detail()).queryByRole("table")).toBeNull();

  // Tıklama: gün seçilir ve sekme durağı o güne taşınır.
  fireEvent.click(cell(table, "2026-11-16"));
  expect(detail()).toHaveAccessibleName("16 Kasım 2026 Pazartesi");
  expect(cell(table, "2026-11-16")).toHaveAttribute("tabindex", "0");
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
});

test("a month without today has no detail until a day is chosen; a day without a sunrise says how it was read", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async () => ok(calendarFixture().month_polar));
  renderPanel();
  const table = await grid("Ocak 2027 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  expect(screen.getByText("Ayrıntısını görmek için bir gün seçin.")).toBeInTheDocument();
  expect(table.querySelectorAll('td[aria-selected="true"]')).toHaveLength(0);
  // Bugün yoksa sekme durağı ayın ilk günüdür.
  expect(cell(table, "2027-01-01")).toHaveAttribute("tabindex", "0");
  key(cell(table, "2027-01-18"), "Enter");
  expect(detail()).toHaveAccessibleName("18 Ocak 2027 Pazartesi");
  expect(within(detail()).getByText(/^Gün doğumu yok · 06:00 yerel ortalama saatte \(05:44\) /)).toBeInTheDocument();
  key(cell(table, "2027-01-19"), "Enter");
  expect(within(detail()).getByText(/^Gün doğumu 11:44 · /)).toBeInTheDocument();
  expect(screen.queryByText("Ayrıntısını görmek için bir gün seçin.")).toBeNull();
});

test("an event's time moves the transit date to that moment; the day button moves it to the day and adds the day panchang panel", async () => {
  renderPanel();
  await grid();
  expect(transit.at).toBe("Now");
  const today = calendarFixture().month.days[24];
  // Olayın saati: transit tarihi o ana gider.
  await act(async () => within(detail()).getByRole("button", { name: "13:41: transit tarihini bu ana götür" }).click());
  expect(transit.at).toEqual({ Jd: today.events[0].event.jd_ut });
  // "Gün panchangını aç": Rust'ın verdiği an (o gün doğumuyla başlayan günün içinde) ve panel.
  expect(kinds).toEqual(["calendar"]);
  const open = within(detail()).getByRole("button", { name: "Gün panchangını aç" });
  expect(open).toHaveAttribute("title", "Transit tarihini bu güne götürür; Gün panchangı paneli düzende yoksa ekler.");
  await act(async () => open.click());
  expect(transit.at).toEqual({ Jd: today.panchang_jd_ut });
  expect(kinds).toEqual(["calendar", "muhurta_day"]);
  // Panel zaten düzendeyse kaldırılmaz; an seçili günün anıdır.
  const table = await grid();
  key(cell(table, "2026-11-09"), "Enter");
  await act(async () => within(detail()).getByRole("button", { name: "Gün panchangını aç" }).click());
  expect(transit.at).toEqual({ Jd: calendarFixture().month.days[8].panchang_jd_ut });
  expect(kinds).toEqual(["calendar", "muhurta_day"]);
  // Takvim yeniden istenmez.
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
});

test("Page Down and Page Up change the month and keep the focus on the same day of the new month", async () => {
  renderPanel();
  const november = await grid();
  cell(november, "2026-11-25").focus();
  await act(async () => {
    key(cell(november, "2026-11-25"), "PageDown");
  });
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 12 });
  const december = await grid(DECEMBER);
  expect(focused()).toBe("2026-12-25");
  expect(cell(december, "2026-12-25")).toHaveAttribute("tabindex", "0");
  // Aralık 31 gündür; Kasım'da 31 yoktur: odak ayın son gününe gider.
  cell(december, "2026-12-31").focus();
  await act(async () => {
    key(cell(december, "2026-12-31"), "PageUp");
  });
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 11 });
  await grid();
  expect(focused()).toBe("2026-11-30");
  // Ay seçimi düğmeleri odağı ızgaraya taşımaz.
  const next = screen.getByRole("button", { name: "Sonraki ay" });
  next.focus();
  await act(async () => next.click());
  await grid(DECEMBER);
  expect(document.activeElement).toBe(next);
});

test("at the end of the range Page Down asks nothing; if the new month fails the focus goes to the panel, not the body", async () => {
  vi.mocked(api.monthCalendar).mockImplementationOnce(async () => ok({ ...calendarFixture().month, next: null }));
  renderPanel();
  const table = await grid();
  cell(table, "2026-11-10").focus();
  await act(async () => {
    key(cell(table, "2026-11-10"), "PageDown");
  });
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
  expect(focused()).toBe("2026-11-10");
  // Önceki ay hatayla biter: ızgara kalkar, odak panelin çerçevesine taşınır.
  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  await act(async () => {
    key(cell(table, "2026-11-10"), "PageUp");
  });
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("grid")).toBeNull();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Transit takvimi" }));
});

test("the filter lists the calendar kinds; a choice is the panel's setting and asks for the month again", async () => {
  const { persistence } = renderPanel();
  await grid();
  const saved = () => persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
  const filter = screen.getByRole("group", { name: "Olay türleri" });
  const boxes = within(filter).getAllByRole("checkbox") as HTMLInputElement[];
  expect(boxes.map((b) => b.closest("label")?.textContent)).toEqual(["Burç girişi", "Nakshatra girişi", "R başlangıcı", "R bitişi", "Yeni ay", "Dolunay", "Doğum boylamına geliş"]);
  // Varsayılan: girişler, dönüşler, yeni ay ve dolunay; doğum boylamına gelişler ve Ay kapalı.
  expect(boxes.map((b) => b.checked)).toEqual([true, true, true, true, true, true, false]);
  expect(screen.getByText("Olay türleri: 6 / 7 seçili")).toBeInTheDocument();
  const moon = screen.getByRole("checkbox", { name: /^Ay'ın olayları da/ }) as HTMLInputElement;
  expect(moon.checked).toBe(false);

  // Doğum boylamına gelişler: tür isteğe girer ve açık haritanın girdisi ancak şimdi gönderilir.
  await act(async () => fireEvent.click(within(filter).getByRole("checkbox", { name: "Doğum boylamına geliş" })));
  expect(requests()).toHaveLength(2);
  expect(requests()[1].filter).toEqual({ kinds: CAL.ids.kinds, moon: false });
  expect(requests()[1].natal).toEqual(CAL.request_full.natal);
  expect(screen.getByText("Olay türleri: 7 / 7 seçili")).toBeInTheDocument();
  // Ay'ın olayları: ayrı bir seçimdir.
  await act(async () => fireEvent.click(moon));
  expect(requests()[2]).toEqual(CAL.request_full);
  const table = await grid();
  await act(async () => {});
  expect(events(cell(table, "2026-11-24")).at(-1)).toBe("+2 olay");
  expect(saved()).toEqual({ kinds: CAL.ids.kinds, moon: true });
  // Gün ayrıntısı kesilmez: günün beş olayı da listelenir.
  key(cell(table, "2026-11-24"), "Enter");
  expect(rowsOf(within(detail()).getByRole("table"))).toEqual([
    ["02:02", "Ay: nakshatra girişi", "Krittika"],
    ["07:24", "Ay: burç girişi", "Boğ"],
    ["11:09", "Güneş, doğum Venüs boylamında", "Akr"],
    ["20:23", "Dolunay", "Boğ · Krittika"],
    ["23:25", "Ay: nakshatra girişi", "Rohini"],
  ]);
});

test("with no kind chosen the grid still shows the facts of every day", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async (request) => {
    const view = calendarFixture().month;
    return ok(request.filter.kinds.length === 0 ? { ...view, days: view.days.map((d) => ({ ...d, events: [] })) } : view);
  });
  renderPanel({ kinds: [], moon: false });
  const table = await grid();
  expect(requests()[0].filter).toEqual({ kinds: [], moon: false });
  expect(requests()[0].natal).toBeNull();
  expect(screen.getByText("Olay türleri: 0 / 7 seçili")).toBeInTheDocument();
  expect(table.querySelectorAll("td[data-date]")).toHaveLength(30);
  expect(table.querySelectorAll("li")).toHaveLength(0);
  expect(lines(cell(table, "2026-11-25"))).toEqual(["25bugün", "K1 PratipadaKrishna 1 · Pratipada", "Rohini"]);
  // Bir tür açılınca seçim Rust'ın sırasıyla kaydedilir.
  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Dolunay" })));
  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Burç girişi" })));
  expect(requests().at(-1)?.filter.kinds).toEqual(["SignIngress", "FullMoon"]);
});

test("with an unknown birth time the natal kind is marked approximate and the input says so", async () => {
  renderPanel({ kinds: ["TransitToNatal"] }, { opened: unknownTimeOpened() });
  await grid();
  expect(requests()[0].natal).toMatchObject({ time_known: false, jd_ut: FX.opened.chart.jd_ut });
  expect(screen.getByText("Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir; doğum Ay'ına gelişler yaklaşıktır.")).toHaveAttribute("role", "note");
  // Tür seçili değilken not da doğum verisi de yoktur.
  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Doğum boylamına geliş" })));
  expect(requests().at(-1)?.natal).toBeNull();
  expect(screen.queryByText(/^Doğum saati bilinmiyor/)).toBeNull();
});

test("a cut event list is said above the grid", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async () => ok({ ...calendarFixture().month, truncated: true }));
  renderPanel();
  await grid();
  expect(screen.getByText("Olay listesi 300 olayda kesildi: sonraki olaylar gösterilmiyor. Tür seçimini daraltın.")).toHaveAttribute("role", "note");
});

test("without a current place the panel asks for one and nothing is computed; the chart's place can be used", async () => {
  const { persistence } = renderPanel({}, { prefs: FX.prefs.prefs });
  expect(screen.getByRole("note")).toHaveTextContent("Bulunulan yer seçilmedi.");
  expect(api.monthCalendar).not.toHaveBeenCalled();
  // Ay seçimi yanıt gelene dek kullanılamaz.
  expect(screen.getByRole("combobox", { name: "Ay" })).toBeDisabled();
  expect(screen.getByRole("textbox", { name: "Yıl" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Önceki ay" })).toHaveAttribute("aria-disabled", "true");
  await act(async () => fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "chart" } }));
  await grid();
  expect(requests()[0].place).toEqual(FX.muhurta.place);
  // Yer seçimi panelin ayarıdır ve düzenle birlikte kaydedilir.
  await act(async () => {});
  expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ place: "chart" });
});

test("days without a sunrise are marked and explained; low precision is said", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async () => ok({ ...calendarFixture().month_polar, warnings: ["LowPrecision"] }));
  renderPanel();
  const table = await grid("Ocak 2027 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  // Tromsø, Ocak 2027: Güneş ayın 19'una dek doğmaz.
  const dark = cell(table, "2027-01-18");
  expect(dark.querySelector(".calendar-tithi")).toHaveTextContent(/\*\s*gün doğumu yok$/);
  expect(dark.querySelector('.calendar-tithi [title="gün doğumu yok"]')).toHaveTextContent("*");
  expect(cell(table, "2027-01-19").querySelector(".calendar-tithi")).not.toHaveTextContent("*");
  expect(table.querySelectorAll('.calendar-tithi [title="gün doğumu yok"]')).toHaveLength(18);
  expect(screen.getByText("* Bu günde Güneş doğmuyor: tithi ve nakshatra 06:00 yerel ortalama saatte okundu.")).toBeInTheDocument();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("a failure is shown with retry; the focus does not fall to the body", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Hesap tamamlanamadı (x).");
  // Yer ve ay seçimi hata durumunda da yerindedir.
  expect(screen.getByRole("combobox", { name: "Yer" })).toBeInTheDocument();
  const retry = within(alert).getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  await act(async () => retry.click());
  await grid();
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Transit takvimi" }));
});

test("without an open chart there is only a message and nothing is asked", () => {
  renderPanel({}, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.monthCalendar).not.toHaveBeenCalled();
  expect(screen.queryByRole("group", { name: "Ay seçimi" })).toBeNull();
});

test("the week starts on Sunday in English and on Monday in French; month and weekday names follow the language", async () => {
  await setLanguage("en");
  const { unmount } = renderPanel();
  const english = await grid("Calendar for November 2026: tithi and Moon nakshatra at sunrise, the events of the day");
  expect(within(english).getAllByRole("columnheader").map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
  expect(english.querySelectorAll("tbody tr")).toHaveLength(5);
  expect([...english.querySelectorAll("tbody tr:first-child td")].map((td) => td.getAttribute("data-date"))[0]).toBe("2026-11-01");
  expect(lines(cell(english, "2026-11-25"))).toEqual(["25today", "K1 PratipadaKrishna 1 · Pratipada", "Rohini", "13:41 Me → VishakhaMercury: nakshatra ingress · Vishakha"]);
  expect(events(cell(english, "2026-11-13"))).toEqual(["21:24 Me R endsMercury: R ends · Li"]);
  unmount();

  await setLanguage("fr");
  renderPanel();
  const french = await grid("Calendrier de novembre 2026 : tithi et nakshatra de la Lune au lever du Soleil, événements du jour");
  expect(within(french).getAllByRole("columnheader").map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."]);
  expect(french.querySelectorAll("tbody tr")).toHaveLength(6);
  expect(cell(french, "2026-11-25").querySelector(".calendar-today")).toHaveTextContent("aujourd’hui");
});

test("the panel is in the palette catalog, the names exist in three languages and French uses the typographic apostrophe", () => {
  expect(MODULE_PANELS.find((m) => m.kind === "calendar")).toEqual({ kind: "calendar", w: 9, h: 24 });
  expect(PANEL_REGISTRY.calendar).toEqual({ titleKey: "panel.calendar", component: CalendarPanel });
  expect([tr.panel.calendar, en.panel.calendar, fr.panel.calendar]).toEqual(["Transit takvimi", "Transit calendar", "Calendrier des transits"]);
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.calendar.month)).toEqual(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"]);
    expect(Object.keys(lang.calendar.weekdayShort).sort()).toEqual(Object.keys(lang.panchang.weekday).sort());
    expect(Object.keys(lang.calendar.short).sort()).toEqual([...CAL.ids.kinds, "return"].sort());
    expect(lang.errors.invalid_calendar_input).toContain("{{field}}");
  }
  expect(JSON.stringify(fr.calendar)).not.toContain("'");
});
