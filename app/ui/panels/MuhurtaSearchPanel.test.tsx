import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { muhurtaSearch: vi.fn(), loadMuhurtaFilters: vi.fn(), saveMuhurtaFilters: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, MuhurtaCriteria, MuhurtaSearchView, MuhurtaWindowView, SavedMuhurtaFilter } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { EMPTY_CRITERIA } from "../modules/muhurta";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { MAX_SHOWN_WINDOWS, MuhurtaSearchPanel } from "./MuhurtaSearchPanel";

const FX = fixture();
const PAGES = FX.muhurta.pages;
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "ms", kind: "muhurta_search", x: 0, y: 0, w: 8, h: 18, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const criteria = (patch: Partial<MuhurtaCriteria> = {}): MuhurtaCriteria => ({ ...EMPTY_CRITERIA, ...patch });
const saved = (name: string, patch: Partial<MuhurtaCriteria> = {}): SavedMuhurtaFilter => ({ name, criteria: criteria(patch) });

let transit: Transit;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <MuhurtaSearchPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

/** Elle çözülen sayfa: geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<MuhurtaSearchView>) => void;
  const promise = new Promise<Result<MuhurtaSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
/** Fikstürün üç sayfasını sırayla döner. */
function threePages() {
  for (const page of fixture().muhurta.pages) vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok(page));
}
const run = () => screen.getByRole("button", { name: "Ara" });
const summary = () => document.querySelector("[data-summary]")?.textContent;
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const resultRows = () => within(screen.getByRole("table", { name: "Bulunan pencereler" })).getAllByRole("row").slice(1);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
const box = (name: string) => screen.getByRole("checkbox", { name });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([]));
  vi.mocked(api.saveMuhurtaFilters).mockImplementation(async (filters) => ok(filters));
  vi.mocked(api.setAppPrefs).mockResolvedValue(ok(null));
});

test("the form has visibly labelled fields and nothing is searched until asked", async () => {
  renderPanel();
  await waitFor(() => expect(api.loadMuhurtaFilters).toHaveBeenCalledTimes(1));
  const form = screen.getByRole("form", { name: "Muhurta süzgeci" });
  for (const name of ["Kayıtlı süzgeç", "Yer"]) expect(within(form).getByRole("combobox", { name })).toBeInTheDocument();
  for (const name of ["Başlangıç tarihi", "Bitiş tarihi"]) expect(within(form).getByLabelText(name)).toHaveAttribute("type", "date");
  expect(within(form).getByRole("spinbutton", { name: "En kısa süre (dk)" })).toHaveValue(0);
  expect(within(form).getByRole("textbox", { name: "Süzgeç adı" })).toHaveValue("");
  // Beş küme: her biri başlıklı bir gruptur; kutuların etiketi görünür.
  expect([...form.querySelectorAll("fieldset > legend")].map((l) => l.textContent)).toEqual(["Günler", "Tithiler", "Nakshatralar", "Lagna burçları", "Dışlananlar"]);
  expect(within(screen.getByRole("group", { name: "Günler" })).getAllByRole("checkbox").map((c) => c.closest("label")?.textContent)).toEqual([
    "Pazar",
    "Pazartesi",
    "Salı",
    "Çarşamba",
    "Perşembe",
    "Cuma",
    "Cumartesi",
  ]);
  expect(within(screen.getByRole("group", { name: "Tithiler" })).getAllByRole("checkbox")).toHaveLength(30);
  expect(within(screen.getByRole("group", { name: "Nakshatralar" })).getAllByRole("checkbox")).toHaveLength(27);
  expect(within(screen.getByRole("group", { name: "Lagna burçları" })).getAllByRole("checkbox")).toHaveLength(12);
  expect(within(screen.getByRole("group", { name: "Dışlananlar" })).getAllByRole("checkbox").map((c) => c.closest("label")?.textContent)).toEqual([
    "Rikta tithi",
    "Vishti karana",
    "Vyatipata",
    "Vaidhriti",
    "Rahu dilimi",
    "Yamaganda",
    "Gulika",
    "Panchaka",
    "Jüpiter yanık",
    "Venüs yanık",
  ]);
  expect(box("S9 Navami")).not.toBeChecked();
  expect(box("K15 Amavasya")).toBeInTheDocument();
  expect(screen.getByText("Tithiler · kısıt yok")).toBeInTheDocument();
  expect(screen.getByText("Hiçbir kutusu işaretli olmayan küme kısıt değildir.")).toBeInTheDocument();
  expect(screen.getByText(/^Süzgeci kurup “Ara”ya basın\./)).toBeInTheDocument();
  expect(api.muhurtaSearch).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("a search sends the filter, the place and the natal Moon, then lists the windows of every page", async () => {
  threePages();
  renderPanel({ place: "chart", criteria: FX.muhurta.query.criteria });
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("30 pencere · 2026-11-26 – 2027-01-31"));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
  // Tarihler boş: Rust bugünden başlar. Yerin adı gitmez.
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0]).toEqual([
    { from_date: null, to_date: null, place: FX.muhurta.place, criteria: FX.muhurta.query.criteria, natal: FX.muhurta.natal },
    FX.opened.chart.meta.settings,
    null,
  ]);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[1][2]).toEqual(PAGES[0].next);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[2][2]).toEqual(PAGES[1].next);
  const table = screen.getByRole("table", { name: "Bulunan pencereler" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Başlangıç", "Bitiş", "Süre", "Vara", "Tithi", "Nakshatra", "Nitya yoga", "Karana", "Lagna"]);
  const rows = resultRows();
  expect(rows).toHaveLength(30);
  expect(cells(rows[0])).toEqual(["2026-11-26 08:14", "2026-11-26 09:32", "1 sa 17 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Akr"]);
  expect(cells(rows[1])).toEqual(["2026-11-26 10:50", "2026-11-26 13:26", "2 sa 35 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Oğl"]);
  // Süzgeç Perşembe gündüzleriydi; her pencere en az 30 dakikadır.
  expect(new Set(rows.map((r) => cells(r)[3]))).toEqual(new Set(["Perşembe"]));
  expect(screen.getByText("Yöntem: muhurta.v1")).toBeInTheDocument();
  expect(screen.getByText(/Pencereler olgudur; uygunluk hükmü değildir\.$/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Durdur" })).toBeNull();
});

test("the criteria are edited with checkboxes and saved with the layout", async () => {
  const { persistence } = renderPanel();
  fireEvent.click(box("Perşembe"));
  fireEvent.click(box("Pazartesi"));
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: criteria({ weekdays: ["Monday", "Thursday"] }) }));
  fireEvent.click(box("K14 Chaturdashi"));
  fireEvent.click(box("S2 Dwitiya"));
  fireEvent.click(box("Rohini"));
  fireEvent.click(box("Asl"));
  fireEvent.click(box("Rahu dilimi"));
  fireEvent.click(box("Rikta tithi"));
  fireEvent.click(box("Yalnızca gündüz"));
  fireEvent.click(box("Tara bala elverişli"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "En kısa süre (dk)" }), { target: { value: "45" } });
  const want = criteria({
    weekdays: ["Monday", "Thursday"],
    tithis: [2, 29],
    nakshatras: [4],
    lagna_signs: ["Leo"],
    daytime_only: true,
    min_minutes: 45,
    exclude: ["Rikta", "RahuSegment"],
    require_tara: true,
  });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: want }));
  expect(box("Perşembe")).toBeChecked();
  expect(screen.getByText("Tithiler · 2 seçili")).toBeInTheDocument();
  expect(screen.getByText("Nakshatralar · 1 seçili")).toBeInTheDocument();
  // Kutuyu yeniden işaretlemek kümeden çıkarır; aralık dışı süre sınıra çekilir.
  fireEvent.click(box("Pazartesi"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "En kısa süre (dk)" }), { target: { value: "99999" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: { ...want, weekdays: ["Thursday"], min_minutes: 10080 } }));
  // Arama, saklanan koşullarla gider.
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  fireEvent.click(run());
  await waitFor(() => expect(api.muhurtaSearch).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0].criteria).toEqual({ ...want, weekdays: ["Thursday"], min_minutes: 10080 });
});

test("typed dates are sent as they are and broken stored criteria fall back to no constraint", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  renderPanel({ place: "chart", criteria: { weekdays: "Friday", tithis: [99], min_minutes: -1 } });
  fireEvent.change(screen.getByLabelText("Başlangıç tarihi"), { target: { value: "2027-01-10" } });
  fireEvent.change(screen.getByLabelText("Bitiş tarihi"), { target: { value: "2027-01-31" } });
  // Enter da aramayı başlatır (form gönderimi).
  fireEvent.submit(screen.getByRole("form", { name: "Muhurta süzgeci" }));
  await waitFor(() => expect(summary()).toBe("3 pencere · 2026-11-26 – 2027-01-31"));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0]).toMatchObject({ from_date: "2027-01-10", to_date: "2027-01-31", criteria: EMPTY_CRITERIA });
});

test("the start of a row moves the transit date to that moment", async () => {
  threePages();
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  expect(transit.at).toBe("Now");
  const first = PAGES[0].windows[0];
  const button = within(resultRows()[0]).getByRole("button", { name: "Transit tarihini 2026-11-26 08:14 anına götür" });
  expect(button).toHaveTextContent("2026-11-26 08:14");
  fireEvent.click(button);
  expect(transit.at).toEqual({ Jd: first.start_jd_ut });
  // Sonuç listesi ve odak yerinde durur.
  expect(resultRows()).toHaveLength(30);
});

test("past a second the search shows its progress and a stop button; stopping keeps the windows and the focus", async () => {
  vi.useFakeTimers();
  try {
    const replies = [deferred(), deferred()];
    replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
    renderPanel();
    fireEvent.click(run());
    await act(async () => vi.advanceTimersByTime(999));
    // Bir saniyeden kısa aramada ilerleme ve "Durdur" çıkmaz.
    expect(screen.queryByRole("button", { name: "Durdur" })).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent("");
    expect(summary()).toBe("Aranıyor…");
    await act(async () => vi.advanceTimersByTime(1));
    const stop = screen.getByRole("button", { name: "Durdur" });
    await act(async () => replies[0].resolve(ok(fixture().muhurta.pages[0])));
    expect(screen.getByRole("status")).toHaveTextContent("Taranan bölüm: 1 / 3");
    expect(summary()).toBe("Şimdiye dek 15 pencere · 2026-11-26 – 2027-01-31");
    // Arama sürerken "Ara" ikinci bir arama başlatmaz.
    expect(run()).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(run());
    expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
    stop.focus();
    fireEvent.click(stop);
    // "Durdur" kalkar; odak gövdeye düşmez, "Ara"ya geçer.
    expect(stop).not.toBeInTheDocument();
    expect(run()).toHaveFocus();
    expect(run()).not.toHaveAttribute("aria-disabled");
    expect(summary()).toBe("Arama durduruldu: 15 pencere · 2026-11-26 – 2027-01-31");
    expect(resultRows()).toHaveLength(15);
    // Yoldaki sayfa gelir: eklenmez, yeni sayfa istenmez.
    await act(async () => replies[1].resolve(ok(fixture().muhurta.pages[1])));
    expect(resultRows()).toHaveLength(15);
    expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("status")).toHaveTextContent("");
  } finally {
    vi.useRealTimers();
  }
});

test("the list does not unmount between pages: a focused row keeps the focus while more windows arrive", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  renderPanel();
  fireEvent.click(run());
  await act(async () => replies[0].resolve(ok(fixture().muhurta.pages[0])));
  const table = screen.getByRole("table", { name: "Bulunan pencereler" });
  const button = within(resultRows()[3]).getByRole("button");
  button.focus();
  await act(async () => replies[1].resolve(ok(fixture().muhurta.pages[1])));
  expect(resultRows()).toHaveLength(27);
  await act(async () => replies[2].resolve(ok(fixture().muhurta.pages[2])));
  expect(resultRows()).toHaveLength(30);
  expect(screen.getByRole("table", { name: "Bulunan pencereler" })).toBe(table);
  expect(button).toBeInTheDocument();
  expect(button).toHaveFocus();
  // Arama bittiğinde odak gövdeye düşmemiştir.
  expect(document.activeElement).not.toBe(document.body);
});

test("an error is shown with the windows that arrived, and retry searches again", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok(fixture().muhurta.pages[0])).mockResolvedValueOnce(failure);
  renderPanel();
  fireEvent.click(run());
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(summary()).toBe("Arama tamamlanamadı: 15 pencere · 2026-11-26 – 2027-01-31");
  expect(resultRows()).toHaveLength(15);
  // Form hata durumunda da kullanılabilir.
  expect(box("Perşembe")).toBeEnabled();
  threePages();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(summary()).toBe("30 pencere · 2026-11-26 – 2027-01-31"));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(5);
});

test("an invalid range is reported by the field Rust names", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce({ status: "error", error: { code: "invalid_muhurta_input", params: { field: "range" } } });
  renderPanel();
  fireEvent.click(run());
  expect(await screen.findByRole("alert")).toHaveTextContent("Muhurta isteği geçersiz: range.");
  // Hiç yanıt gelmedi: özet yerine arama satırı durur, tablo yoktur.
  expect(screen.queryByRole("table")).toBeNull();
});

test("without a place the search button stays focusable, says why and does nothing", async () => {
  renderPanel({});
  const button = run();
  expect(button).toHaveAttribute("aria-disabled", "true");
  expect(button).not.toBeDisabled();
  const why = document.getElementById(button.getAttribute("aria-describedby")!);
  expect(why).toHaveTextContent(/^Bulunulan yer seçilmedi\./);
  fireEvent.click(button);
  await act(() => Promise.resolve());
  expect(api.muhurtaSearch).not.toHaveBeenCalled();
  // Yer seçilince düğme kullanılabilir olur.
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "chart" } });
  await waitFor(() => expect(run()).not.toHaveAttribute("aria-disabled"));
  expect(run()).not.toHaveAttribute("aria-describedby");
});

test("a saved filter is loaded into the form; saving adds or overwrites by name", async () => {
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([saved("Yolculuk", { weekdays: ["Thursday"], exclude: ["Vishti"] }), saved("Açılış", { daytime_only: true })]));
  const { persistence } = renderPanel();
  const select = screen.getByRole("combobox", { name: "Kayıtlı süzgeç" });
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "Yolculuk", "Açılış"]));
  const name = screen.getByRole("textbox", { name: "Süzgeç adı" });
  const save = screen.getByRole("button", { name: "Süzgeci kaydet" });
  const remove = screen.getByRole("button", { name: "Süzgeci sil" });
  // Ad ve seçim yokken iki eylem de kullanılamaz ama odaklanabilir.
  expect(save).toHaveAttribute("aria-disabled", "true");
  expect(remove).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(save);
  fireEvent.click(remove);
  expect(api.saveMuhurtaFilters).not.toHaveBeenCalled();

  fireEvent.change(select, { target: { value: "Yolculuk" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: criteria({ weekdays: ["Thursday"], exclude: ["Vishti"] }) }));
  expect(name).toHaveValue("Yolculuk");
  expect(box("Perşembe")).toBeChecked();
  expect(box("Vishti karana")).toBeChecked();

  // Koşulu değiştirip aynı adla kaydetmek üzerine yazar (büyük/küçük harf ayırmadan).
  fireEvent.click(box("Cuma"));
  fireEvent.change(name, { target: { value: " yolculuk " } });
  fireEvent.click(save);
  await waitFor(() => expect(api.saveMuhurtaFilters).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveMuhurtaFilters).mock.calls[0][0]).toEqual([saved("yolculuk", { weekdays: ["Thursday", "Friday"], exclude: ["Vishti"] }), saved("Açılış", { daytime_only: true })]);
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "yolculuk", "Açılış"]));
  expect(select).toHaveValue("yolculuk");

  // Yeni ad listeye eklenir. Kayıt yalnızca ad ve koşul taşır.
  fireEvent.change(name, { target: { value: "Hasat" } });
  fireEvent.click(save);
  await waitFor(() => expect(api.saveMuhurtaFilters).toHaveBeenCalledTimes(2));
  const sent = vi.mocked(api.saveMuhurtaFilters).mock.calls[1][0];
  expect(sent.map((f) => f.name)).toEqual(["yolculuk", "Açılış", "Hasat"]);
  expect(Object.keys(sent[2]).sort()).toEqual(["criteria", "name"]);
  expect(Object.keys(sent[2].criteria).sort()).toEqual(Object.keys(EMPTY_CRITERIA).sort());
  await waitFor(() => expect(select).toHaveValue("Hasat"));
});

test("deleting the selected filter moves the focus to the picker; a failed save is reported", async () => {
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([saved("Yolculuk"), saved("Açılış")]));
  renderPanel();
  const select = screen.getByRole("combobox", { name: "Kayıtlı süzgeç" });
  await waitFor(() => expect(within(select).getAllByRole("option")).toHaveLength(3));
  fireEvent.change(select, { target: { value: "Açılış" } });
  const remove = screen.getByRole("button", { name: "Süzgeci sil" });
  expect(remove).not.toHaveAttribute("aria-disabled");
  // Kayıt düşerse liste ve seçim yerinde durur.
  vi.mocked(api.saveMuhurtaFilters).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "muhurta_filters" } } });
  remove.focus();
  fireEvent.click(remove);
  expect(await screen.findByRole("alert")).toHaveTextContent("Geçersiz değer: muhurta_filters.");
  expect(select).toHaveValue("Açılış");
  expect(within(select).getAllByRole("option")).toHaveLength(3);
  fireEvent.click(remove);
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "Yolculuk"]));
  expect(vi.mocked(api.saveMuhurtaFilters).mock.calls[1][0]).toEqual([saved("Yolculuk")]);
  // Silinen süzgecin düğmesi kullanılamaz olur: odak seçiciye taşınmıştır.
  expect(select).toHaveFocus();
  expect(select).toHaveValue("");
  expect(screen.getByRole("textbox", { name: "Süzgeç adı" })).toHaveValue("");
  expect(remove).toHaveAttribute("aria-disabled", "true");
  expect(screen.queryByRole("alert")).toBeNull();
});

test("a long result is cut at the display limit and says so", async () => {
  const window = PAGES[0].windows[0];
  const many: MuhurtaWindowView[] = Array.from({ length: MAX_SHOWN_WINDOWS + 100 }, (_, i) => ({
    ...window,
    start_jd_ut: (window.start_jd_ut ?? 0) + i,
    end_jd_ut: (window.end_jd_ut ?? 0) + i,
  }));
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], windows: many, scanned: 1, chunks: 1 }));
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("600 pencere · 2026-11-26 – 2027-01-31"));
  expect(resultRows()).toHaveLength(500);
  expect(screen.getByText("İlk 500 pencere gösteriliyor (toplam 600). Süzgeci daraltın ya da aralığı kısaltın.")).toBeInTheDocument();
});

test("no window, a day without a sunrise and a fallback ephemeris are said in words", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], windows: [], scanned: 1, chunks: 1, warnings: ["SunriseUnavailable", "LowPrecision"] }));
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("0 pencere · 2026-11-26 – 2027-01-31"));
  expect(screen.queryByRole("table")).toBeNull();
  const notes = screen.getAllByRole("note").map((n) => n.textContent);
  expect(notes[0]).toMatch(/^Aralığın bir bölümünde Güneş doğmuyor ya da batmıyor/);
  expect(notes[1]).toBe("Düşük hassasiyet: Güneş veya Ay yedek ephemeris ile hesaplandı.");
});

test("the result list is a keyboard-focusable scroll group named by its caption", async () => {
  threePages();
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  const group = screen.getByRole("group", { name: "Bulunan pencereler" });
  expect(group).toHaveAttribute("tabindex", "0");
  expect(within(group).getByRole("table", { name: "Bulunan pencereler" })).toBeInTheDocument();
});

test("an unknown birth time marks the two personal conditions as approximate", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  renderPanel({ place: "chart" }, { opened: unknownTimeOpened() });
  for (const name of ["Tara bala elverişli", "Chandra bala elverişli"]) {
    const label = screen.getByText(name).closest("label")!;
    expect(label).toHaveTextContent(`≈yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir ${name}`);
    expect(within(label).getByRole("checkbox")).toBeEnabled();
  }
  // Doğum Ay'ı yine gönderilir (öğle konumu).
  fireEvent.click(run());
  await waitFor(() => expect(api.muhurtaSearch).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0].natal).toEqual(FX.muhurta.natal);
});

test("another chart drops the results, and without a chart the panel asks for nothing", async () => {
  threePages();
  const { unmount } = renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  unmount();
  vi.mocked(api.loadMuhurtaFilters).mockClear();
  renderPanel({ place: "chart" }, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  await act(() => Promise.resolve());
  expect(api.loadMuhurtaFilters).not.toHaveBeenCalled();
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
});

test("the search texts exist in three languages and carry no verdict words", () => {
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.muhurtaSearch.summary).sort()).toEqual(["done", "error", "running", "stopped"]);
    expect(Object.keys(lang.muhurtaSearch.unit).sort()).toEqual(["day", "hour", "minute"]);
  }
  // Panel olgu gösterir: "uygun", "uğurlu", "auspicious", "favorable jour" gibi hüküm sözleri yoktur
  // ("elverişli" yalnızca spec'in adlandırdığı tara ve chandra bala kümesinin adıdır).
  const words = /uğur|iyi gün|kötü gün|auspicious|inauspicious|good day|bad day|faste|néfaste/i;
  for (const lang of [tr, en, fr]) {
    for (const group of [lang.muhurtaSearch, lang.muhurtaDay, lang.muhurta]) expect(JSON.stringify(group)).not.toMatch(words);
  }
});
