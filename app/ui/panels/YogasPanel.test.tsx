import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeYogas: vi.fn() } }));

import { api, type Result } from "../api";
import type { YogaReport } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { YOGA_GROUPS, yogaGroupFilter, YogasPanel } from "./YogasPanel";

const ok = (report: YogaReport = fixture().yogas): Result<YogaReport> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "yg", kind: "yogas", x: 0, y: 0, w: 7, h: 14, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <YogasPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const tables = () => screen.getAllByRole("table");
const caption = (table: HTMLElement) => table.querySelector("caption")?.textContent;
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeYogas).mockReset();
  vi.mocked(api.computeYogas).mockImplementation(async () => ok());
});

test("lists the yogas that form, grouped in catalog order, with grahas, houses, relation and conditions", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(api.computeYogas).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeYogas).mock.calls[0][0].time_known).toBe(true);
  // Delhi fikstürü, İkizler lagna: dokuz grupta on üç kayıt; Mahapurusha ve ashraya yogası yok.
  expect(tables().map(caption)).toEqual([
    "Ay yogaları",
    "Güneş yogaları",
    "Raja yogaları",
    "Dhana yogaları",
    "Değişim yogaları",
    "Viparita raja yogaları",
    "Neecha bhanga",
    "Nabhasa: sayı yogaları",
    "Dosha",
  ]);
  expect(cells(within(tables()[0]).getAllByRole("row")[0])).toEqual(["Yoga", "Grahalar", "Evler", "İlişki", "Koşullar"]);
  expect(tables().map(bodyRows)).toEqual([
    [
      ["Sunapha", "Ay Ve", "5 6", "—", "—"],
      ["Gajakesari", "Ay Jü", "5 11", "—", "—"],
    ],
    [
      ["Vasi", "Gü Ve", "6 7", "—", "—"],
      ["Budha-Aditya", "Gü Me", "7", "birlikte", "—"],
    ],
    [
      ["Kendra–trikona", "Jü Sa", "7 9 10", "birlikte", "—"],
      ["Dharma-Karmadhipati", "Jü Sa", "9 10", "birlikte", "—"],
    ],
    [
      ["Dhana", "Ay Sa", "2 9", "karşılıklı bakış", "—"],
      ["Dhana", "Ma Sa", "9 11", "değişim", "—"],
    ],
    [["Maha değişimi", "Ma Sa", "9 11", "değişim", "—"]],
    [["Vimala", "Ve", "6 12", "—", "—"]],
    [["Neecha bhanga", "Sa", "11", "—", "o burçta yücelen graha kendrada"]],
    [["Pasha", "Gü Ay Ma Me Jü Ve Sa", "5 6 7 9 11", "—", "—"]],
    [["Kuja dosha", "Ma", "9", "—", "Venüs'ten"]],
  ]);
  // Kısaltmalar tam adla, evler "N. ev" diye okunur.
  const moon = within(tables()[0]).getAllByRole("button", { name: "Ay" })[0];
  expect(moon).toHaveTextContent("Ay");
  expect(within(tables()[8]).getByRole("button", { name: "Mars" })).toHaveTextContent("Ma");
  expect(within(tables()[8]).getByRole("button", { name: "9. ev" })).toHaveTextContent("9");
  expect(screen.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeInTheDocument();
  expect(screen.getByText(/^Yalnızca D1 değerlendirilir\./)).toBeInTheDocument();
  expect(screen.getByText("Katalog sürümü: 1")).toBeInTheDocument();
  // Doğum saati biliniyor: yaklaşık işareti ve "değerlendirilmedi" açıklaması yok.
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.queryByText("≈")).toBeNull();
});

test("the group filter is saved with the layout and narrows the list without a new request", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  const select = screen.getByRole("combobox", { name: "Grup" });
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual([
    "Tümü",
    "Pancha Mahapurusha",
    "Ay yogaları",
    "Güneş yogaları",
    "Raja yogaları",
    "Dhana yogaları",
    "Değişim yogaları",
    "Viparita raja yogaları",
    "Neecha bhanga",
    "Nabhasa: sayı yogaları",
    "Nabhasa: ashraya yogaları",
    "Dosha",
  ]);
  fireEvent.change(select, { target: { value: "Dhana" } });
  await waitFor(() => expect(tables().map(caption)).toEqual(["Dhana yogaları"]));
  expect(bodyRows(tables()[0])).toHaveLength(2);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ group: "Dhana" }));
  // Oluşan yogası olmayan grup: tablo yerine açıklama; sayaç satırı raporun tamamını sayar.
  fireEvent.change(select, { target: { value: "Mahapurusha" } });
  expect(await screen.findByText("Bu grupta oluşan yoga yok: Pancha Mahapurusha.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeInTheDocument();
  expect(api.computeYogas).toHaveBeenCalledTimes(1);
});

test("the tables sit in one keyboard-focusable scroll group", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: "Dosha" });
  const group = screen.getByRole("group", { name: "Oluşan yogalar" });
  expect(group).toHaveClass("bala-scroll");
  expect(group).toHaveAttribute("tabindex", "0");
  expect(group).toContainElement(table);
  expect(within(group).getAllByRole("table")).toHaveLength(9);
});

test("the stored filter is read tolerantly", () => {
  expect(yogaGroupFilter({})).toBe("all");
  expect(yogaGroupFilter({ group: "Raja" })).toBe("Raja");
  expect(yogaGroupFilter({ group: "raja" })).toBe("all");
  expect(yogaGroupFilter({ group: 3 })).toBe("all");
});

test("unknown birth time: lagna rules are skipped in Rust, the rest is marked approximate and carries no houses", async () => {
  vi.mocked(api.computeYogas).mockResolvedValue(ok(fixture().yogas_time_unknown));
  renderPanel({}, unknownTimeOpened());
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(vi.mocked(api.computeYogas).mock.calls[0][0].time_known).toBe(false);
  expect(tables().map(caption)).toEqual(["Ay yogaları", "Güneş yogaları", "Nabhasa: sayı yogaları", "Dosha"]);
  // "≈" işareti ekran okuyucuya "yaklaşık" diye okunur; evler çizgidir.
  expect(bodyRows(tables()[0])).toEqual([
    ["≈yaklaşık Sunapha", "Ay Ve", "—", "—", "—"],
    ["≈yaklaşık Gajakesari", "Ay Jü", "—", "—", "—"],
  ]);
  expect(bodyRows(tables()[3])).toEqual([["≈yaklaşık Kuja dosha", "Ma", "—", "—", "Venüs'ten"]]);
  expect(screen.getAllByText("≈")).toHaveLength(6);
  expect(
    screen.getByText("Doğum saati bilinmiyor: lagnaya bağlı 17 kural değerlendirilmedi. Listelenen yogalar yaklaşıktır (≈) ve ev taşımaz."),
  ).toBeInTheDocument();
  expect(screen.getByText("Değerlendirilen kural: 24. Oluşan kayıt: 6.")).toBeInTheDocument();
});

test("an empty report says so and gives the number of rules evaluated", async () => {
  vi.mocked(api.computeYogas).mockResolvedValue(ok({ catalog_version: 1, hits: [], evaluated: 41, skipped_time_unknown: 0 }));
  renderPanel();
  expect(await screen.findByText("Katalogdaki yogalardan hiçbiri oluşmuyor. Değerlendirilen kural: 41.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Grup" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeYogas).not.toHaveBeenCalled();
});

test("an error is shown in the panel, the filter stays and retry asks again", async () => {
  vi.mocked(api.computeYogas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Grup" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(api.computeYogas).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeYogas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Yogalar">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Yogalar" }));
});

test("loading: nothing under a second, then a skeleton, then the list", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<YogaReport>) => void;
    vi.mocked(api.computeYogas).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(9);
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks every yoga it takes part in, a house every yoga that names it", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.yoga);
  const venus = within(screen.getByRole("table", { name: "Viparita raja yogaları" })).getByRole("button", { name: "Venüs" });
  fireEvent.click(venus);
  expect(venus).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["lunar.sunapha", "solar.vasi", "viparita.vimala", "nabhasa_count.pasha"]);
  // Üzerine gelinen öğe sabitlenenin önüne geçer: 11. ev.
  fireEvent.mouseEnter(within(screen.getByRole("table", { name: "Neecha bhanga" })).getByRole("button", { name: "11. ev" }));
  expect(marked()).toEqual(["lunar.gajakesari", "dhana.dhana", "exchange.maha", "neecha_bhanga.neecha_bhanga", "nabhasa_count.pasha"]);
  fireEvent.keyDown(window, { key: "Escape" });
  fireEvent.mouseLeave(within(screen.getByRole("table", { name: "Neecha bhanga" })).getByRole("button", { name: "11. ev" }));
  expect(marked()).toEqual([]);
});

test("every catalog rule, group, relation and condition has a name in the three languages", () => {
  const { yoga_catalog, yoga_conditions } = fixture();
  expect(yoga_catalog).toHaveLength(41);
  // Arayüzün grup listesi Rust kataloğunun grup sırasıdır.
  expect([...new Set(yoga_catalog.map((r) => r.group))]).toEqual([...YOGA_GROUPS]);
  type Names = Record<string, Record<string, string>>;
  for (const messages of [tr, en, fr]) {
    const names = messages.yogas.name as Names;
    for (const { id } of yoga_catalog) {
      const [group, name] = id.split(".");
      expect(names[group]?.[name], id).toEqual(expect.stringMatching(/\S/));
    }
    // Katalogda olmayan bir kuralın adı da durmaz.
    const named = Object.entries(names).flatMap(([group, rules]) => Object.keys(rules).map((name) => `${group}.${name}`));
    expect(named.sort()).toEqual(yoga_catalog.map((r) => r.id).sort());
    expect(Object.keys(messages.yogas.group).sort()).toEqual([...YOGA_GROUPS].sort());
    expect(Object.keys(messages.yogas.condition).sort()).toEqual([...yoga_conditions].sort());
    expect(Object.keys(messages.yogas.relation).sort()).toEqual(["Conjunction", "Exchange", "MutualAspect"]);
  }
  // Klasik adlar üç dilde aynı yazılır (K11); çevrilenler yalnızca değişim sınıflarıdır.
  const flat = (m: typeof tr) => Object.entries(m.yogas.name as Names).flatMap(([g, rules]) => Object.entries(rules).map(([n, v]) => [`${g}.${n}`, v] as const));
  const differing = flat(tr)
    .filter(([id, v]) => new Map(flat(en)).get(id) !== v || new Map(flat(fr)).get(id) !== v)
    .map(([id]) => id);
  expect(differing).toEqual(["exchange.maha", "exchange.third_lord", "exchange.dainya"]);
});
