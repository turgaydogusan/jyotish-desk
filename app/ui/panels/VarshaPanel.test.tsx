import { readFileSync } from "node:fs";
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVarsha: vi.fn(), saveVarshaChart: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, HomePlace, VarshaView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { useSession } from "../session/store";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { MAX_VARSHA_AGE, resetVarshaAge, useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { useLayoutStore } from "../view/store";
import { PANEL_REGISTRY } from "./registry";
import { MUNTHA_MARK, VarshaPanel } from "./VarshaPanel";

const FX = fixture();
const ok = (view: VarshaView = fixture().varsha.current): Result<VarshaView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const DELHI = { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } };
const SYDNEY: HomePlace = { place: { name: "Sydney", country: "Australia", admin1: "New South Wales", lat: -33.8688, lon: 151.2093, geoname_id: null }, tz: { Iana: "Australia/Sydney" } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "vp", kind: "varsha", x: 0, y: 0, w: 7, h: 24, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

let revision = 0;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  revision = useSession().revision;
  return <VarshaPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = {}, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

const facts = () => screen.findByRole("table", { name: "Varshaphala: yılın olguları" });
const positions = () => screen.getByRole("table", { name: "Yıllık harita konumları" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const rows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const ageInput = () => screen.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" });
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  resetVarshaAge();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  // Fikstürde iki yıl vardır: 27 yaşı `next`, öteki her istek süren yıl (26).
  vi.mocked(api.computeVarsha).mockImplementation(async (_input, age) => ok(age === 27 ? fixture().varsha.next : fixture().varsha.current));
  vi.mocked(api.saveVarshaChart).mockImplementation(async () => ({ status: "ok", data: { ...fixture().opened.chart_record, id: 31, kind: "Event", label: "Varshaphala 2026-01-01" } }));
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("asks for the running year at the birth place and lists the facts of the year", async () => {
  renderPanel();
  const table = await facts();
  const opened = fixture().opened;
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Yaş gönderilmez: süren yılı Rust bulur (arayüz saat okumaz). Yer doğum yeridir.
  expect(api.computeVarsha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null }, null, DELHI);
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Olgu", "Değer"]);
  // Delhi fikstürü, 2026-11-25'te: 26 yıl tamamlanmış. Doğum lagnası İkizler; 26 = 2 × 12 + 2:
  // muntha Aslan. Yıllık lagna Oğlak: Aslan, Oğlak'tan 8. evdir.
  expect(rows(table)).toEqual([
    ["Yıl", "27. yıl (tamamlanan yıl 26)"],
    ["Dönüş anı", "2026-01-01 09:28 (UTC+05:30)"],
    ["Yıl sonu (sonraki dönüş)", "2027-01-01 15:41 (UTC+05:30)"],
    ["Harita yeri", "New Delhi, Delhi, India"],
    ["Vara", "Guruvara"],
    ["Yıl lagnası", "Oğl 22°04′ · lordu Satürn"],
    ["Muntha", "Asl · yıllık haritada 8. ev · doğum haritasında 3. ev · lordu Güneş"],
    ["Doğum lagnası", "İki · lordu Merkür"],
    ["Mudda dasanın ilk lordu", "Mars"],
  ]);
  expect(ageInput()).toHaveValue(26);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveValue("birth");
  expect(screen.getByText(/^Dönüş her kipte sidereal boylamdandır/)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: varsha.sidereal_return_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
});

test("draws the annual chart with the muntha mark and gives its table equivalent", async () => {
  const { container, persistence } = renderPanel();
  await facts();
  const svg = screen.getByRole("group", { name: "Yıllık harita, 27. yıl" });
  expect(svg).toHaveAttribute("data-style", "north");
  // Yıllık haritanın dokuz grahası ve muntha işareti (Aslan).
  expect(svg.querySelectorAll("[data-graha]")).toHaveLength(9);
  expect(container.querySelector(".chart-mark")).toHaveTextContent(MUNTHA_MARK);
  expect(screen.getByText("Muntha Asl burcunda (çizimde “Mu”).")).toHaveClass("visually-hidden");
  const table = rows(positions());
  expect(table).toHaveLength(10);
  expect(table[0]).toEqual(["Lagna", "Oğl", "22°04′", "Shravana", "1"]);
  expect(table[1]).toEqual(["Güneş", "Yay", "16°30′", "Purva Ashadha", "12"]);
  // Retro: görünen "R", okunan sözcük. Düğümlerde işaret yoktur.
  expect(table[5]).toEqual(["Jüpiter Rretro", "İki", "27°06′", "Punarvasu", "6"]);
  expect(table[8][0]).toBe("Rahu");
  // Biçim panelin ayarıdır ve düzenle saklanır.
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "south" } });
  expect(screen.getByRole("group", { name: "Yıllık harita, 27. yıl" })).toHaveAttribute("data-style", "south");
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ style: "south" }));
  // Biçim değişimi raporu yeniden istemez.
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
});

test("stepping the year asks Rust for that age; this-year goes back to the running year", async () => {
  renderPanel();
  await facts();
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  expect(ageInput()).toHaveValue(27);
  // Yeni yıl gelene dek önceki yılın tablosu yerinde durur.
  expect(screen.getByRole("table", { name: "Varshaphala: yılın olguları" })).toBeInTheDocument();
  await waitFor(() => expect(rows(screen.getByRole("table", { name: "Varshaphala: yılın olguları" }))[0]).toEqual(["Yıl", "28. yıl (tamamlanan yıl 27)"]));
  expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27]);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "false");
  // 27 yaşı: muntha Başak, yıl lagnası Boğa; mudda dasa Rahu ile başlar.
  const table = rows(screen.getByRole("table", { name: "Varshaphala: yılın olguları" }));
  expect(table[6]).toEqual(["Muntha", "Baş · yıllık haritada 5. ev · doğum haritasında 4. ev · lordu Merkür"]);
  expect(table[8]).toEqual(["Mudda dasanın ilk lordu", "Rahu"]);
  expect(screen.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Önceki yıl" }));
  expect(ageInput()).toHaveValue(26);
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27, 26]));
  fireEvent.click(screen.getByRole("button", { name: "Bu yıl" }));
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  // Kanca her anahtar değişiminde ister: dördüncü istek yaşsızdır.
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27, 26, null]));
});

test("the age field takes a typed age and ignores values outside 0…120", async () => {
  renderPanel();
  await facts();
  fireEvent.change(ageInput(), { target: { value: "45" } });
  expect(ageInput()).toHaveValue(45);
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.at(-1)?.[1]).toBe(45));
  const before = vi.mocked(api.computeVarsha).mock.calls.length;
  for (const bad of ["121", "-1", "2.5", ""]) fireEvent.change(ageInput(), { target: { value: bad } });
  expect(ageInput()).toHaveValue(45);
  expect(api.computeVarsha).toHaveBeenCalledTimes(before);
  // Uçlarda adım düğmeleri kapalıdır (odağı tutar, yılı değiştirmez).
  fireEvent.change(ageInput(), { target: { value: "0" } });
  expect(screen.getByRole("button", { name: "Önceki yıl" })).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(screen.getByRole("button", { name: "Önceki yıl" }));
  expect(ageInput()).toHaveValue(0);
  fireEvent.change(ageInput(), { target: { value: String(MAX_VARSHA_AGE) } });
  expect(screen.getByRole("button", { name: "Sonraki yıl" })).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  expect(ageInput()).toHaveValue(MAX_VARSHA_AGE);
  expect(MAX_VARSHA_AGE).toBe(FX.varsha.max_age);
});

test("the selected year belongs to the open chart and is shared between consumers", async () => {
  const seen: Record<string, number | null> = {};
  function Probe({ name, chartId }: { name: string; chartId: number }) {
    seen[name] = useVarshaAge(chartId).age;
    return null;
  }
  const chartId = fixture().opened.chart_record.id;
  renderWithStores(
    <>
      <LivePanel />
      <Probe name="same" chartId={chartId} />
      <Probe name="other" chartId={chartId + 1} />
    </>,
    { layout: LAYOUT({}) },
  );
  await facts();
  expect(seen).toEqual({ same: null, other: null });
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  await waitFor(() => expect(seen).toEqual({ same: 27, other: null }));
});

test("the current place is a panel setting; without one the panel says so and asks nothing", async () => {
  const { persistence } = renderPanel({}, { prefs: { ...FX.prefs.prefs, home_place: null } });
  await facts();
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "home" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "home" }));
  expect(screen.getByText("Bulunulan yer seçilmedi: yıllık harita için bir yer seçin ya da “Doğum yeri”ne dönün.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Bulunulan yer penceresi buradan açılır.
  fireEvent.click(screen.getByRole("button", { name: "Bulunulan yeri seç" }));
  expect(screen.getByRole("dialog", { name: "Bulunulan yer" })).toBeInTheDocument();
});

test("with a current place the annual chart is asked for that place and saved there", async () => {
  renderPanel({ place: "home" }, { prefs: { ...FX.prefs.prefs, home_place: SYDNEY } });
  const table = await facts();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][2]).toEqual({ lat: -33.8688, lon: 151.2093, tz: { Iana: "Australia/Sydney" } });
  expect(rows(table)[3]).toEqual(["Harita yeri", "Sydney, New South Wales, Australia"]);
  expect(screen.getByRole("button", { name: "Bulunulan yeri değiştir" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Olay haritası olarak kaydet" }));
  await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01");
  expect(api.saveVarshaChart).toHaveBeenCalledWith(fixture().opened.chart_record.id, 26, true);
});

test("saving stores the annual chart once, reports its label and refreshes the lists", async () => {
  renderPanel();
  await facts();
  const before = revision;
  const save = screen.getByRole("button", { name: "Olay haritası olarak kaydet" });
  expect(screen.getByRole("status")).toHaveTextContent("");
  fireEvent.click(save);
  expect(await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01")).toBe(screen.getByRole("status"));
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(1);
  expect(api.saveVarshaChart).toHaveBeenCalledWith(fixture().opened.chart_record.id, 26, false);
  // Harita açılmaz; listeler yenilenir.
  expect(revision).toBe(before + 1);
  // Aynı yıl ikinci kez kaydedilmez.
  expect(save).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(save);
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(1);
  // Başka yıla geçince bildirim o yıla taşınmaz ve kayıt yeniden yapılabilir.
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  await screen.findByRole("group", { name: "Yıllık harita, 28. yıl" });
  expect(screen.getByRole("status")).toHaveTextContent("");
  expect(screen.getByRole("button", { name: "Olay haritası olarak kaydet" })).not.toHaveAttribute("aria-disabled");
});

test("a failed save is reported in the panel and can be tried again", async () => {
  vi.mocked(api.saveVarshaChart).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "x" } } });
  renderPanel();
  await facts();
  const save = screen.getByRole("button", { name: "Olay haritası olarak kaydet" });
  fireEvent.click(save);
  expect(await screen.findByRole("alert")).toBeInTheDocument();
  expect(save).not.toHaveAttribute("aria-disabled");
  fireEvent.click(save);
  await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(2);
});

test("unknown birth time is sent to Rust; there is no annual chart, no muntha and nothing to save", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(fixture().varsha.time_unknown));
  renderPanel({}, { opened: unknownTimeOpened() });
  const table = await facts();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: yıllık harita ve muntha hesaplanmadı (ikisi de lagna ister). Dönüş anı ve mudda dasa yaklaşıktır (≈).")).toBeInTheDocument();
  expect(rows(table).map((r) => r[0])).toEqual(["Yıl", "Dönüş anı", "Yıl sonu (sonraki dönüş)", "Mudda dasanın ilk lordu"]);
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.queryByRole("group", { name: /Yıllık harita/ })).toBeNull();
  expect(screen.queryByRole("button", { name: "Olay haritası olarak kaydet" })).toBeNull();
  expect(screen.queryByRole("combobox", { name: "Biçim" })).toBeNull();
});

test("a low-precision warning from the report is shown as text", async () => {
  const view = fixture().varsha.current;
  view.report.warnings = ["LowPrecision"];
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(view));
  renderPanel();
  await facts();
  expect(screen.getByText("Güneş veya Ay yedek ephemeris ile hesaplandı.")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeVarsha).not.toHaveBeenCalled();
});

test("an error is shown in the panel, the year controls stay, and retry asks again", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu yıl" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await facts();
  expect(api.computeVarsha).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Varshaphala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}) },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await facts();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Varshaphala" }));
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<VarshaView>) => void;
    vi.mocked(api.computeVarsha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
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

test("shared highlight: a pinned graha marks its row; both tables scroll by keyboard", async () => {
  renderPanel();
  await facts();
  const saturn = within(positions()).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(positions().querySelectorAll("tr.is-highlighted")).toHaveLength(1);
  expect(positions().querySelector('tr[data-graha="Saturn"]')).toHaveClass("is-highlighted");
  for (const name of ["Varshaphala: yılın olguları", "Yıllık harita konumları"]) {
    expect(screen.getByRole("group", { name })).toHaveAttribute("tabindex", "0");
  }
});

test("the panel is registered, opens from the palette list, and its texts exist in three languages", () => {
  expect(PANEL_REGISTRY.varsha.component).toBe(VarshaPanel);
  expect(MODULE_PANELS.find((m) => m.kind === "varsha")).toEqual({ kind: "varsha", w: 7, h: 24 });
  // Klasik terimler üç dilde aynı yazılır (K11).
  for (const lang of [tr, en, fr]) {
    expect(lang.panel.varsha).toBe("Varshaphala");
    expect(lang.varsha.muntha).toBe("Muntha");
    expect(lang.varsha.vara).toBe("Vara");
  }
  expect([tr.varsha.mudda.system, en.varsha.mudda.system, fr.varsha.mudda.system]).toEqual(["Mudda dasa", "Mudda dasha", "Mudda dasha"]);
  // Fransızca metinler tipografik kesme işareti kullanır (C12).
  expect(JSON.stringify(fr.varsha)).not.toContain("'");
  expect(zoneLabel(19_800)).toBe("UTC+05:30");
  expect(zoneLabel(-12_600)).toBe("UTC-03:30");
  expect(zoneLabel(0)).toBe("UTC+00:00");
});

test("inert buttons are dimmed: the step buttons at the ends and the save button once it has saved", async () => {
  renderPanel();
  await facts();
  // Kapalı düğme odağı tutar (`aria-disabled`); görünüşü de kapalı olduğunu söylemelidir.
  for (const name of ["Önceki yıl", "Sonraki yıl"]) expect(screen.getByRole("button", { name })).toHaveClass("varsha-step");
  expect(screen.getByRole("button", { name: "Olay haritası olarak kaydet" }).closest(".varsha-save")).not.toBeNull();
  const css = readFileSync("ui/panels/varsha.css", "utf8");
  expect(css).toMatch(/\.varsha-step\[aria-disabled="true"\],\s*\.varsha-save button\[aria-disabled="true"\]\s*\{[^}]*opacity:\s*0\.6;[^}]*cursor:\s*default;/);
});
