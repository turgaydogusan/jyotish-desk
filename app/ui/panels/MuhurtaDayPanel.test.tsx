import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { dayPanchang: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, DayPanchangView } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { useHighlight } from "../highlight/store";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { NAKSHATRA_NATURE_NAMES, TARA_NAMES } from "./labels";
import { dayLink, MuhurtaDayPanel } from "./MuhurtaDayPanel";

const FX = fixture();
const ok = (view: DayPanchangView = fixture().muhurta.day): Result<DayPanchangView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "md", kind: "muhurta_day", x: 0, y: 0, w: 7, h: 18, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const withHome = (): AppPrefs => ({ ...FX.prefs.prefs, home_place: FX.muhurta.home_place });

let transit: Transit;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <MuhurtaDayPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

const limbs = () => screen.findByRole("table", { name: "Beş uzuv ve değişim saatleri" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const rows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.dayPanchang).mockImplementation(async () => ok());
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("draws the day of the transit moment for the birth place: strip, limbs, factors and horas", async () => {
  renderPanel();
  const table = await limbs();
  // Bir istek: transit anı ("şimdi" Rust'ta okunur), yerin koordinatı ve dilimi (ad gitmez),
  // haritanın hesap ayarı ve doğum Ay'ı.
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.dayPanchang).mock.calls[0]).toEqual(["Now", FX.muhurta.place, FX.opened.chart.meta.settings, FX.muhurta.natal]);
  expect(screen.getByText("New Delhi, Delhi, India")).toBeInTheDocument();
  expect(screen.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(screen.getByText("Gün doğumu 06:55 · gün batımı 17:20 · sonraki gün doğumu 06:56 (+1)")).toBeInTheDocument();

  // Şerit: sekiz dilim; Çarşamba günü Yamaganda 2., Gulika 4., Rahu 5. dilimdir. İşaret yazıdır.
  const strip = screen.getByRole("list", { name: "Gündüzün sekiz dilimi" });
  const segments = within(strip).getAllByRole("listitem");
  expect(segments.map((s) => s.getAttribute("data-mark"))).toEqual([null, "yamaganda", null, "gulika", "rahu", null, null, null]);
  expect(segments[0]).toHaveTextContent("106:55–08:13");
  expect(segments[1]).toHaveTextContent("2Yama08:13–09:31");
  expect(segments[3]).toHaveTextContent("4Guli10:49–12:07");
  expect(segments[4]).toHaveTextContent("5Rahu12:07–13:26");
  expect(segments[7]).toHaveTextContent("816:02–17:20");
  expect(segments.filter((s) => s.classList.contains("is-marked"))).toHaveLength(3);
  expect(screen.getByText("Abhijit 11:47–12:28 (Çarşamba: bazı derlemeler bu gün Abhijit'i kullanmaz.)")).toBeInTheDocument();

  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Öğe", "Değer", "Başlangıç", "Bitiş"]);
  const body = rows(table);
  expect(body.map((r) => r[0])).toEqual(["Tithi", "Tithi", "Nakshatra", "Nakshatra", "Nakshatra", "Nakshatra", "Nakshatra", "Nitya yoga", "Nitya yoga", "Karana", "Karana", "Karana"]);
  expect(body.map((r) => r[1])).toEqual([
    "Krishna 1 · Pratipada · Nanda grubu",
    "Krishna 2 · Dwitiya · Bhadra grubu",
    "Rohini · pada 2 · Dhruva (sabit)",
    "Rohini · pada 3 · Dhruva (sabit)",
    "Rohini · pada 4 · Dhruva (sabit)",
    "Mrigashira · pada 1 · Mridu (yumuşak)",
    "Mrigashira · pada 2 · Mridu (yumuşak)",
    "Shiva",
    "Siddha",
    "Kaulava",
    "Taitila",
    "Garaja",
  ]);
  // Günün dışına taşan uçlar okla ve ekran okuyucu metniyle işaretlidir.
  expect(body[0].slice(2)).toEqual(["06:55 ←günün başından önce başlar", "16:50"]);
  expect(body[1].slice(2)).toEqual(["16:50", "06:56 (+1) →günün sonundan sonra sürer"]);
  expect(body[5].slice(2)).toEqual(["20:36", "01:53 (+1)"]);

  expect(screen.getByText("Bu gün işaret yok.")).toBeInTheDocument();
  const personal = screen.getByRole("table", { name: "Açık haritanın Ay'ına göre" });
  expect(rows(personal)).toEqual([
    ["Tara bala", "8 · Mitra", "elverişli kümede", "06:55 ←günün başından önce başlar", "20:36"],
    ["Tara bala", "9 · Parama Mitra", "elverişli kümede", "20:36", "06:56 (+1) →günün sonundan sonra sürer"],
    ["Chandra bala", "doğum Ay burcundan 8. ev", "elverişli kümede değil", "06:55 ←günün başından önce başlar", "06:56 (+1) →günün sonundan sonra sürer"],
  ]);
  const horas = rows(screen.getByRole("table", { name: "Hora tablosu" }));
  expect(horas).toHaveLength(24);
  expect(horas[0]).toEqual(["1", "Merkür", "06:55", "07:55"]);
  expect(horas.slice(0, 8).map((r) => r[1])).toEqual(["Merkür", "Ay", "Satürn", "Jüpiter", "Mars", "Güneş", "Venüs", "Merkür"]);
  expect(horas[23]).toEqual(["24", "Satürn", "05:56 (+1)", "06:56 (+1)"]);
  expect(screen.getByText("Yöntem: muhurta.v1")).toBeInTheDocument();
  expect(screen.getByText(/olgudur; uygunluk hükmü değildir/)).toBeInTheDocument();
  expect(screen.queryByText("≈")).toBeNull();
});

test("without a current place the panel asks for one, and choosing it in the dialog loads the day", async () => {
  renderPanel({});
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveValue("home");
  expect(screen.getByRole("note")).toHaveTextContent(/^Bulunulan yer seçilmedi\./);
  await act(() => Promise.resolve());
  expect(api.dayPanchang).not.toHaveBeenCalled();
  const pick = screen.getByRole("button", { name: "Bulunulan yeri seç" });
  pick.focus();
  fireEvent.click(pick);
  const dialog = screen.getByRole("dialog", { name: "Bulunulan yer" });
  fireEvent.click(within(dialog).getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ }));
  await limbs();
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].home_place).toEqual(FX.muhurta.home_place);
  expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual(FX.muhurta.place);
  fireEvent.click(within(dialog).getByRole("button", { name: "Kapat" }));
  // Pencere kapanınca odak açan düğmeye döner; düğme artık "değiştir"dir.
  expect(screen.getByRole("button", { name: "Bulunulan yeri değiştir" })).toHaveFocus();
  expect(screen.queryByRole("note")).toBeNull();
});

test("a saved current place is used, and the place choice is saved with the layout", async () => {
  const { persistence } = renderPanel({}, { prefs: withHome() });
  await limbs();
  expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual(FX.muhurta.place);
  expect(within(screen.getByRole("combobox", { name: "Yer" })).getAllByRole("option").map((o) => o.textContent)).toEqual(["Bulunulan yer", "Haritanın doğum yeri", "Özel yer"]);
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "custom" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "custom" }));
  // Özel yer seçilene dek gün gösterilmez ve yeni istek gitmez.
  expect(screen.getByRole("note")).toHaveTextContent("Özel yer seçilmedi. Arama kutusundan bir yer seçin.");
  expect(screen.queryByRole("table")).toBeNull();
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
});

test("a custom place is picked from the atlas and kept in the panel's settings", async () => {
  vi.useFakeTimers();
  try {
    const { persistence } = renderPanel({ place: "custom" });
    const input = screen.getByRole("combobox", { name: "Özel yer ara" });
    fireEvent.change(input, { target: { value: "delhi" } });
    await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
    fireEvent.keyDown(input, { key: "Enter" });
    await act(async () => vi.advanceTimersByTime(0));
    const hit = FX.places[0];
    expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual({ lat: hit.lat, lon: hit.lon, tz: { Iana: "Asia/Kolkata" } });
    expect(savedSettings(persistence)).toEqual({
      place: "custom",
      custom: { place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: "Asia/Kolkata" } },
    });
    expect(input).toHaveValue("");
    expect(screen.getByRole("table", { name: "Hora tablosu" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("the date follows the transit moment or stays on today, and the choice is saved", async () => {
  expect(dayLink({})).toBe("transit");
  expect(dayLink({ link: "today" })).toBe("today");
  expect(dayLink({ link: "tomorrow" })).toBe("transit");
  const { persistence } = renderPanel();
  await limbs();
  // Transit bağında panel tarih seçicisini de taşır.
  const form = screen.getByRole("form", { name: "Transit anı" });
  fireEvent.change(within(form).getByLabelText("Tarih"), { target: { value: "2030-01-01" } });
  fireEvent.change(within(form).getByLabelText("Saat"), { target: { value: "03:00" } });
  fireEvent.click(within(form).getByRole("button", { name: "Uygula" }));
  const local = { Local: { date: "2030-01-01", time: "03:00", tz: browserTimeZone() } };
  await waitFor(() => expect(vi.mocked(api.dayPanchang).mock.calls.at(-1)?.[0]).toEqual(local));
  expect(api.dayPanchang).toHaveBeenCalledTimes(2);
  // "Bugün": transit anı ne olursa olsun "şimdi" gönderilir ve tarih seçici gösterilmez.
  fireEvent.change(screen.getByRole("combobox", { name: "Tarih bağı" }), { target: { value: "today" } });
  await waitFor(() => expect(vi.mocked(api.dayPanchang).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(screen.queryByRole("form", { name: "Transit anı" })).toBeNull();
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", link: "today" }));
  // Başka bir panelden gelen transit anı (ör. arama sonucu) "bugün" bağını etkilemez.
  const calls = vi.mocked(api.dayPanchang).mock.calls.length;
  act(() => transit.setAt({ Jd: 2_461_400.5 }));
  await act(() => Promise.resolve());
  expect(api.dayPanchang).toHaveBeenCalledTimes(calls);
});

test("choosing now again reads the day again", async () => {
  renderPanel();
  await limbs();
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
  act(() => transit.setAt("Now"));
  await waitFor(() => expect(api.dayPanchang).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.dayPanchang).mock.calls[1][0]).toBe("Now");
});

test("a day without a sunrise shows the limbs and the flags, with a badge and no segments", async () => {
  vi.mocked(api.dayPanchang).mockImplementation(async () => ok(fixture().muhurta.day_polar));
  renderPanel();
  const table = await limbs();
  const note = screen.getByRole("note");
  expect(note).toHaveTextContent("Gün doğumu yok");
  expect(note).toHaveTextContent("gündüz dilimleri ve hora tablosu hesaplanmaz. Gün 06:00 yerel ortalama saatte (05:44) başlar.");
  expect(screen.queryByRole("list", { name: "Gündüzün sekiz dilimi" })).toBeNull();
  expect(screen.queryByRole("table", { name: "Hora tablosu" })).toBeNull();
  expect(screen.queryByText(/^Gün doğumu \d/)).toBeNull();
  expect(rows(table)).toHaveLength(12);
  const flags = within(screen.getByRole("list", { name: "İşaretler" })).getAllByRole("listitem");
  expect(flags.map((f) => f.getAttribute("data-flag"))).toEqual(["Rikta", "Vyatipata", "Panchaka"]);
  expect(flags[0]).toHaveTextContent("Rikta tithi 16:15–05:44 (+1) →günün sonundan sonra sürer");
  expect(flags[1]).toHaveTextContent("Vyatipata 05:44 ←günün başından önce başlar–16:08");
  expect(screen.queryByText("Bu gün işaret yok.")).toBeNull();
  // Kişiye bağlı etken istenmediyse (Rust `null`) tablo yoktur.
  expect(screen.queryByRole("table", { name: "Açık haritanın Ay'ına göre" })).toBeNull();
});

test("an unknown birth time marks the personal factors as approximate", async () => {
  renderPanel({ place: "chart" }, { opened: unknownTimeOpened() });
  await limbs();
  // Doğum Ay'ı yine gönderilir (öğle konumu); tablo "≈" ve açıklamasıyla işaretlenir.
  expect(vi.mocked(api.dayPanchang).mock.calls[0][3]).toEqual(FX.muhurta.natal);
  const caption = screen.getByRole("table", { name: /Açık haritanın Ay'ına göre/ }).querySelector("caption")!;
  expect(caption).toHaveTextContent("≈yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir Açık haritanın Ay'ına göre");
  // Günün uzuvları doğum saatine bağlı değildir: işaretlenmez.
  expect(within(screen.getByRole("table", { name: "Beş uzuv ve değişim saatleri" })).queryByText("≈")).toBeNull();
});

test("loading: nothing under a second, then a skeleton, then the day", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<DayPanchangView>) => void;
    vi.mocked(api.dayPanchang).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    await act(async () => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    await act(async () => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    // Yer ve tarih bağı yüklenirken de kullanılabilir.
    expect(screen.getByRole("combobox", { name: "Yer" })).toBeEnabled();
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(3);
  } finally {
    vi.useRealTimers();
  }
});

test("an error keeps the toolbar, and retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.dayPanchang).mockResolvedValueOnce(failure);
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Gün panchangı">
      <LivePanel />
    </section>,
    { layout: LAYOUT({ place: "chart" }) },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  expect(screen.getByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Yer" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Tarih bağı" })).toBeInTheDocument();
  retry.focus();
  fireEvent.click(retry);
  await limbs();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Gün panchangı" }));
  expect(api.dayPanchang).toHaveBeenCalledTimes(2);
});

test("while another day is read the old one stays on screen and the focus stays where it was", async () => {
  renderPanel();
  await limbs();
  const lord = within(screen.getByRole("table", { name: "Hora tablosu" })).getAllByRole("button", { name: "Merkür" })[0];
  lord.focus();
  let resolve!: (r: Result<DayPanchangView>) => void;
  vi.mocked(api.dayPanchang).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  act(() => transit.setAt({ Jd: 2_470_162.26 }));
  await waitFor(() => expect(api.dayPanchang).toHaveBeenCalledTimes(2));
  // Eski gün yerinde durur; panel meşgul olduğunu bildirir; odaktaki düğme kaldırılmaz.
  expect(screen.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(document.querySelector(".muhurta-day")).toHaveAttribute("aria-busy", "true");
  expect(lord).toBeInTheDocument();
  expect(lord).toHaveFocus();
  await act(async () => resolve(ok(fixture().muhurta.day_polar)));
  expect(screen.getByText("2050-12-21 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(document.querySelector(".muhurta-day")).not.toHaveAttribute("aria-busy");
});

test("shared highlight: a hora lord is pinned and every hora of that graha is marked", async () => {
  let highlight!: ReturnType<typeof useHighlight>;
  function Probe() {
    highlight = useHighlight();
    return null;
  }
  renderWithStores(
    <>
      <LivePanel />
      <Probe />
    </>,
    { layout: LAYOUT({ place: "chart" }) },
  );
  const horas = await screen.findByRole("table", { name: "Hora tablosu" });
  const mercury = within(horas).getAllByRole("button", { name: "Merkür" })[0];
  expect(mercury).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(mercury);
  expect(highlight.pinned).toEqual({ type: "graha", id: "Mercury" });
  expect(mercury).toHaveAttribute("aria-pressed", "true");
  const marked = [...horas.querySelectorAll("tr.is-highlighted")].map((r) => r.getAttribute("data-hora"));
  expect(marked).toEqual(["1", "8", "15", "22"]);
  // Başka bir panelden gelen vurgu da satırları işaretler.
  act(() => highlight.togglePin({ type: "graha", id: "Moon" }));
  expect([...horas.querySelectorAll("tr.is-highlighted")].map((r) => r.getAttribute("data-hora"))).toEqual(["2", "9", "16", "23"]);
});

test("the tables sit in keyboard-focusable scroll groups named by their captions", async () => {
  renderPanel();
  await limbs();
  for (const name of ["Beş uzuv ve değişim saatleri", "Açık haritanın Ay'ına göre", "Hora tablosu"]) {
    const group = screen.getByRole("group", { name });
    expect(group).toHaveAttribute("tabindex", "0");
    expect(within(group).getByRole("table", { name })).toBeInTheDocument();
  }
  // Abhijit çubuğu yalnızca çizimdir; saatleri metin olarak da yazılıdır.
  expect(document.querySelector(".day-strip-abhijit")).toHaveAttribute("aria-hidden", "true");
});

test("without an open chart the panel says so and asks for nothing", async () => {
  renderPanel({ place: "chart" }, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  await act(() => Promise.resolve());
  expect(api.dayPanchang).not.toHaveBeenCalled();
});

test("every flag, nature and tara has a name in three languages; classical names are the same in all", () => {
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.muhurta.flag).sort()).toEqual([...FX.muhurta_ids.flags].sort());
    expect(Object.keys(lang.muhurta.nature).sort()).toEqual([...FX.muhurta_ids.nakshatra_natures].sort());
    expect(Object.keys(lang.muhurtaDay.mark).sort()).toEqual(["abhijit", "gulika", "rahu", "yamaganda"]);
    // Şeritteki kısa yazılar ve klasik adlar üç dilde aynıdır (K11).
    expect(lang.muhurtaDay.mark).toEqual(tr.muhurtaDay.mark);
    for (const id of ["Vyatipata", "Vaidhriti", "Yamaganda", "Gulika", "Panchaka"] as const) expect(lang.muhurta.flag[id]).toBe(id);
    expect([lang.muhurta.tara, lang.muhurta.chandra]).toEqual(["Tara bala", "Chandra bala"]);
  }
  expect(Object.keys(NAKSHATRA_NATURE_NAMES).sort()).toEqual([...FX.muhurta_ids.nakshatra_natures].sort());
  expect(TARA_NAMES).toHaveLength(9);
  expect(FX.muhurta_ids.tithi_groups).toEqual(["Nanda", "Bhadra", "Jaya", "Rikta", "Purna"]);
});
