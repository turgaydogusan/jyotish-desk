import { act, fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { eventSearch: vi.fn(), eventSearchCancel: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, EventSearchView, OpenedClient } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { chartInput } from "../modules/input";
import { EVENT_KINDS } from "../search/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { DEFAULT_EVENT_GRAHAS, DEFAULT_EVENT_KINDS, EventSearchPanel, MAX_SHOWN_EVENTS, parseSet } from "./EventSearchPanel";

const FX = fixture();
const VIEW = FX.search.events;
const ok = (view: EventSearchView): Result<EventSearchView> => ({ status: "ok", data: view });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "es", kind: "event_search", x: 0, y: 0, w: 7, h: 18, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

let transit: Transit;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <EventSearchPanel panel={active.panels[0]} />;
}
function renderPanel(settings: Record<string, unknown> = {}, { opened = fixture().opened, prefs = FX.prefs.prefs }: { opened?: OpenedClient | null; prefs?: AppPrefs | null } = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}
function deferred() {
  let resolve!: (r: Result<EventSearchView>) => void;
  const promise = new Promise<Result<EventSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
const run = () => screen.getByRole("button", { name: "Ara" });
const box = (name: string) => screen.getByRole("checkbox", { name });
const summary = () => document.querySelector("[data-summary]")?.textContent;
const rows = () => within(screen.getByRole("table", { name: "Olaylar" })).getAllByRole("row").slice(1);
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const settle = () => act(async () => {});
const sent = () => vi.mocked(api.eventSearch).mock.calls.at(-1)![0];

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.eventSearchCancel).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.eventSearch).mockResolvedValue(ok(VIEW));
});

test("a new panel starts with the slow grahas, the chart's place and nothing searched", () => {
  renderPanel();
  const form = screen.getByRole("form", { name: "Olay araması ayarları" });
  expect(within(form).getByRole("combobox", { name: "Yer" })).toHaveDisplayValue("Haritanın doğum yeri");
  for (const name of ["Başlangıç tarihi", "Bitiş tarihi"]) expect(within(form).getByLabelText(name)).toHaveValue("");
  const kinds = within(form).getByRole("group", { name: "Olay türleri" });
  expect(within(kinds).getAllByRole("checkbox")).toHaveLength(EVENT_KINDS.length);
  expect(within(kinds).getAllByRole("checkbox", { checked: true }).map((c) => c.parentElement?.textContent)).toEqual(["Burç girişi", "R başlangıcı", "R bitişi"]);
  const grahas = within(form).getByRole("group", { name: "Grahalar" });
  expect(within(grahas).getAllByRole("checkbox", { checked: true }).map((c) => c.parentElement?.textContent)).toEqual(["Jüpiter", "Satürn", "Rahu"]);
  expect(screen.getByText(/Türleri, grahaları ve aralığı seçip/)).toBeInTheDocument();
  expect(api.eventSearch).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("the request carries the settings, and the natal chart only when a natal kind is selected", async () => {
  const opened = fixture().opened;
  const { persistence } = renderPanel();
  fireEvent.change(screen.getByLabelText("Başlangıç tarihi"), { target: { value: "2026-11-26" } });
  fireEvent.click(run());
  await settle();
  const birth = opened.chart_record.birth;
  expect(sent()).toEqual({
    from_date: "2026-11-26",
    to_date: null,
    place: { lat: birth.place.lat, lon: birth.place.lon, tz: birth.tz },
    kinds: [...DEFAULT_EVENT_KINDS],
    grahas: [...DEFAULT_EVENT_GRAHAS],
    settings: opened.chart.meta.settings,
    natal: null,
  });
  // Seçim panelin ayarıdır: Rust'ın sırasıyla saklanır.
  fireEvent.click(box("Doğum boylamına geliş"));
  fireEvent.click(box("Güneş"));
  await settle();
  const saved = persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
  expect(saved).toMatchObject({ kinds: ["SignIngress", "StationRetrograde", "StationDirect", "TransitToNatal"], grahas: ["Sun", "Jupiter", "Saturn", "Rahu"] });
  fireEvent.click(run());
  await settle();
  expect(sent().natal).toEqual(chartInput(opened));
  expect(sent().grahas).toEqual(["Sun", "Jupiter", "Saturn", "Rahu"]);
});

test("the events are listed in the place's time and a row moves the transit date", async () => {
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(summary()).toBe(`${VIEW.events.length} olay · 2026-11-26 – 2027-03-31`);
  expect(rows()).toHaveLength(VIEW.events.length);
  // İlk üç olay: yeni ay, Satürn'ün R bitişi, Jüpiter'in R başlangıcı (fikstür; +05:30).
  expect(cells(rows()[0]).slice(1)).toEqual(["Yeni ay", "Akr · Jyeshtha"]);
  expect(cells(rows()[1]).slice(1)).toEqual(["Satürn: R bitiyor", "Bal"]);
  expect(cells(rows()[2]).slice(1)).toEqual(["Jüpiter: R başlıyor", "Asl"]);
  const kinds = new Set(VIEW.events.map((e) => e.event.kind));
  // Her tür için bir ad vardır; ham tür adı görünmez.
  for (const kind of kinds) expect(screen.getByRole("table", { name: "Olaylar" })).not.toHaveTextContent(kind);
  const first = VIEW.events[0];
  const when = within(rows()[0]).getByRole("button");
  expect(when).toHaveAccessibleName(/transit tarihini bu ana götür/);
  fireEvent.click(when);
  expect(transit.at).toEqual({ Jd: first.event.jd_ut });
  expect(screen.getByText(/Yöntem: event_search\.v1/)).toBeInTheDocument();
});

test("while searching the panel says so and stopping keeps the focus on the search button", async () => {
  vi.useFakeTimers();
  try {
    const pending = deferred();
    vi.mocked(api.eventSearch).mockReset().mockReturnValueOnce(pending.promise);
    renderPanel();
    fireEvent.click(run());
    expect(run()).toHaveAttribute("aria-disabled", "true");
    await act(async () => vi.advanceTimersByTime(150));
    expect(screen.getByRole("status")).toHaveTextContent("Aranıyor…");
    const stop = screen.getByRole("button", { name: "Durdur" });
    stop.focus();
    fireEvent.click(stop);
    expect(api.eventSearchCancel).toHaveBeenCalledWith(vi.mocked(api.eventSearch).mock.calls[0][1]);
    expect(screen.queryByRole("button", { name: "Durdur" })).toBeNull();
    expect(run()).toHaveFocus();
    expect(screen.getByText("Arama durduruldu.")).toBeInTheDocument();
    // Durdurulan aramanın geç gelen yanıtı gösterilmez.
    await act(async () => pending.resolve(ok(VIEW)));
    expect(screen.queryByRole("table")).toBeNull();
  } finally {
    vi.useRealTimers();
  }
});

test("no events, a cut list and a long list are said in words", async () => {
  vi.mocked(api.eventSearch).mockResolvedValueOnce(ok({ ...VIEW, events: [] }));
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(summary()).toMatch(/^Bu aralıkta olay yok \(2026-11-26 – 2027-03-31\)\./);
  expect(screen.queryByRole("table")).toBeNull();
  // Kesilen liste: nereye dek tarandığı yazılır.
  const many = Array.from({ length: MAX_SHOWN_EVENTS + 5 }, () => VIEW.events[0]);
  vi.mocked(api.eventSearch).mockResolvedValueOnce(ok({ ...VIEW, events: many, truncated: true, scanned_to_jd_ut: VIEW.events[0].event.jd_ut }));
  fireEvent.click(run());
  await settle();
  expect(rows()).toHaveLength(MAX_SHOWN_EVENTS);
  expect(screen.getByText(/Liste 2000 olayda kesildi; 2026-12-0\d tarihinden sonrası taranmadı\./)).toBeInTheDocument();
  expect(screen.getByText(`İlk ${MAX_SHOWN_EVENTS} olay gösteriliyor (${MAX_SHOWN_EVENTS + 5} olay). Aralığı ya da seçimi daraltın.`)).toBeInTheDocument();
});

test("an incomplete selection says why, a changed form marks the list, and an error can be retried", async () => {
  renderPanel({ kinds: ["Conjunction"], grahas: ["Sun"] });
  expect(run()).toHaveAccessibleDescription("Kavuşum için en az iki graha seçin.");
  fireEvent.click(run());
  expect(api.eventSearch).not.toHaveBeenCalled();
  fireEvent.click(box("Güneş"));
  expect(run()).toHaveAccessibleDescription("En az bir graha seçin.");
  fireEvent.click(box("Tam kavuşum"));
  expect(run()).toHaveAccessibleDescription("En az bir olay türü seçin.");
  // Yalnızca graha istemeyen türler: graha seçimi gerekmez.
  fireEvent.click(box("Dolunay"));
  expect(run()).not.toHaveAttribute("aria-disabled");
  vi.mocked(api.eventSearch).mockResolvedValueOnce({ status: "error", error: { code: "invalid_search_input", params: { field: "range" } } });
  fireEvent.click(run());
  await settle();
  expect(screen.getByRole("alert")).toHaveTextContent("Arama isteği geçersiz: range.");
  // Hata: önceki arama olmadığı gibi tablo da yoktur; yeniden deneme odağı "Ara"da tutar.
  expect(screen.queryByRole("table")).toBeNull();
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(run()).toHaveFocus();
  await settle();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(rows().length).toBeGreaterThan(0);
  expect(screen.queryByRole("note")).toBeNull();
  fireEvent.click(box("Yeni ay"));
  expect(screen.getByRole("note")).toHaveTextContent("Ayarlar değişti; liste önceki aramaya ait.");
});

test("without a chart, without a place and without a birth time the panel says what is missing", () => {
  const view = renderPanel({}, { opened: null });
  expect(screen.getByText(tr.panel.noChart)).toBeInTheDocument();
  view.unmount();
  // Bulunulan yer seçili ama girilmemiş: arama kullanılamaz ve nedeni yazılıdır.
  const noHome = renderPanel({ place: "home" }, { prefs: { ...FX.prefs.prefs, home_place: null } });
  expect(run()).toHaveAttribute("aria-disabled", "true");
  expect(screen.getByRole("note")).toHaveTextContent(tr.muhurta.place.missingHome);
  noHome.unmount();
  renderPanel({ kinds: ["TransitToNatal"], grahas: ["Saturn"] }, { opened: unknownTimeOpened() });
  expect(screen.getByRole("note")).toHaveTextContent("Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir");
});

test("stored settings are read tolerantly and every kind has a name in every language", () => {
  expect(parseSet(["FullMoon", "Eclipse", "NewMoon"], EVENT_KINDS, DEFAULT_EVENT_KINDS)).toEqual(["NewMoon", "FullMoon"]);
  expect(parseSet("bozuk", EVENT_KINDS, DEFAULT_EVENT_KINDS)).toEqual([...DEFAULT_EVENT_KINDS]);
  expect(parseSet([], EVENT_KINDS, DEFAULT_EVENT_KINDS)).toEqual([]);
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.eventSearch.kind)).toEqual([...EVENT_KINDS]);
    for (const kind of EVENT_KINDS) expect(lang.eventSearch.event).toHaveProperty(kind);
  }
  expect(JSON.stringify(fr.eventSearch)).not.toContain("'");
});
