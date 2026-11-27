import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeBhavaTable: vi.fn() } }));

import { api, type Result } from "../api";
import type { BhavaTable } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, placidusOpened, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { bhavaPointKind, BhavaTablePanel } from "./BhavaTablePanel";

const ok = (table: BhavaTable | null = fixture().bhava.table): Result<BhavaTable | null> => ({ status: "ok", data: table });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "bt", kind: "bhava_table", x: 0, y: 0, w: 7, h: 20, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <BhavaTablePanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const bhavas = (point = "başlangıç") => screen.findByRole("table", { name: `Bhava tablosu: on iki bhavanın ${point} noktası, nakshatrası, lordları ve yayı` });
const grahas = () => screen.findByRole("table", { name: "Grahaların bhavası: burç evi, bhava, bhava başlangıcına ve ortasına uzaklık" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const body = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeBhavaTable).mockReset();
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok());
});

test("whole sign: twelve bhavas start at the sign and no graha leaves its sign house", async () => {
  renderPanel();
  const table = await bhavas();
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].time_known).toBe(true);
  const rows = within(table).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Bhava", "Başlangıç", "Nakshatra", "Pada", "Burç lordu", "Nakshatra lordu", "Yay"]);
  expect(rows).toHaveLength(13);
  // Delhi fikstürü, İkizler lagna 16°20′: 1. bhava İkizler'in başında başlar.
  expect(body(table).slice(0, 3)).toEqual([
    ["1", "İki 0°00′", "Mrigashira", "3", "Merkür", "Mars", "30°00′"],
    ["2", "Yen 0°00′", "Punarvasu", "4", "Ay", "Jüpiter", "30°00′"],
    ["3", "Asl 0°00′", "Magha", "1", "Güneş", "Ketu", "30°00′"],
  ]);
  // Bhava numarası tam adıyla okunur.
  expect(within(rows[1]).getByRole("button", { name: "1. bhava" })).toHaveTextContent("1");
  const g = await grahas();
  expect(cells(within(g).getAllByRole("row")[0])).toEqual(["Graha", "Burç evi", "Bhava", "Başlangıçtan", "Ortadan"]);
  expect(body(g)).toEqual([
    ["Güneş", "7", "7", "16°30′", "+0°10′"],
    ["Ay", "5", "5", "19°28′", "+3°07′"],
    ["Mars", "9", "9", "4°06′", "−12°13′"],
    ["Merkür", "7", "7", "8°02′", "−8°18′"],
    ["Jüpiter", "11", "11", "1°23′", "−14°56′"],
    ["Venüs", "6", "6", "7°42′", "−8°37′"],
    ["Satürn", "11", "11", "16°32′", "+0°12′"],
    ["Rahu", "2", "2", "11°11′", "−5°09′"],
    ["Ketu", "8", "8", "11°11′", "−5°09′"],
  ]);
  expect(screen.getByText(/^Ev sistemi tam burç: bhava burcun kendisidir ve chalit görünümü rasi ile aynıdır\./)).toBeInTheDocument();
  expect(screen.queryByText(/Graha, burcunun evinden başka bir bhavadadır/)).toBeNull();
  expect(screen.getByText("Yöntem: bhava_table.v1")).toBeInTheDocument();
});

test("both tables sit in keyboard-focusable scroll groups named by their captions", async () => {
  renderPanel();
  for (const table of [await bhavas(), await grahas()]) {
    const group = table.closest(".bala-scroll") as HTMLElement;
    expect(group).toHaveAttribute("role", "group");
    expect(group).toHaveAttribute("tabindex", "0");
    expect(group).toHaveAccessibleName(table.querySelector("caption")!.textContent!);
  }
});

test("Sripati: the point setting switches between the sandhi, the Porphyry middle and the end, and is saved with the layout", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_sripati));
  const { persistence } = renderPanel();
  expect(body(await bhavas()).slice(0, 2)).toEqual([
    ["1", "Boğ 29°12′", "Mrigashira", "2", "Venüs", "Mars", "30°00′"],
    ["2", "İki 29°12′", "Punarvasu", "3", "Merkür", "Jüpiter", "25°45′"],
  ]);
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "mid" } });
  // 1. bhavanın ortası lagnadır (İkizler 16°20′).
  expect(body(await bhavas("orta")).slice(0, 2)).toEqual([
    ["1", "İki 16°20′", "Ardra", "3", "Merkür", "Rahu", "30°00′"],
    ["2", "Yen 12°05′", "Pushya", "3", "Ay", "Satürn", "25°45′"],
  ]);
  expect(within(await bhavas("orta")).getAllByRole("row")[0]).toHaveTextContent("Orta");
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ point: "mid" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "end" } });
  // Bitiş, sonraki bhavanın başlangıcıdır.
  expect(body(await bhavas("bitiş"))[0]).toEqual(["1", "İki 29°12′", "Punarvasu", "3", "Merkür", "Jüpiter", "30°00′"]);
  expect(body(await grahas())[1]).toEqual(["Ay", "5", "5", "28°44′", "+11°37′"]);
  expect(screen.getByText(/^Ev sistemi Sripati: başlangıç sandhidir; orta Porphyry noktasıdır/)).toBeInTheDocument();
  expect(screen.queryByText(/Placidus/)).toBeNull();
  // Ayar değişimi yeni bir istek değildir.
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(1);
});

test("Placidus: shifted grahas carry a mark read as words, and there is no middle", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus));
  renderPanel({ point: "mid" }, placidusOpened());
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].settings.house_system).toBe("Placidus");
  expect(body(await bhavas("orta"))[0]).toEqual(["1", "—", "—", "—", "—", "—", "22°37′"]);
  expect(screen.getByText("Placidus'ta bhava ortası tanımlanmaz: nokta bhavanın başlangıcıdır.")).toHaveAttribute("role", "note");
  const rows = body(await grahas());
  expect(rows[0]).toEqual(["Güneş", "7", "7", "0°10′", "—"]);
  expect(rows.slice(3, 6)).toEqual([
    ["Merkür", "7", "6* burç evinden farklı", "24°07′", "—"],
    ["Jüpiter", "11", "10* burç evinden farklı", "27°47′", "—"],
    ["Venüs", "6", "5* burç evinden farklı", "29°27′", "—"],
  ]);
  const mercury = within(await grahas()).getAllByRole("row")[4];
  expect(mercury.querySelector('[aria-hidden="true"]')).toHaveTextContent("*");
  expect(mercury.querySelector(".visually-hidden")).toHaveTextContent("burç evinden farklı");
  expect(screen.getByText("* Graha, burcunun evinden başka bir bhavadadır.")).toBeInTheDocument();
  expect(screen.getByText("Ev sistemi Placidus: nokta bhavanın başlangıcıdır; bhava ortası tanımlanmaz.")).toBeInTheDocument();
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "start" } });
  expect(body(await bhavas())[0]).toEqual(["1", "İki 16°20′", "Ardra", "3", "Merkür", "Rahu", "22°37′"]);
  expect(screen.queryByText(/^Placidus'ta bhava ortası tanımlanmaz/)).toBeNull();
});

test("Placidus inside the polar circle: the Porphyry fallback is said in words", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus_polar));
  renderPanel();
  // İki bhava aynı burçta başlar (Aslan 2°46′ ve Aslan 22°00′).
  expect(body(await bhavas()).slice(0, 2).map((r) => r[1])).toEqual(["Asl 2°46′", "Asl 22°00′"]);
  expect(screen.getByText("Placidus bu enlemde hesaplanamadı; başlangıç noktaları Porphyry noktalarıdır.")).toHaveAttribute("role", "note");
});

test("pinning a graha or a bhava highlights its rows", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus));
  renderPanel({}, placidusOpened());
  const g = await grahas();
  const row = (name: string) => within(g).getByRole("button", { name }).closest("tr")!;
  fireEvent.click(within(g).getByRole("button", { name: "Merkür" }));
  expect(row("Merkür")).toHaveClass("is-highlighted");
  expect(row("Güneş")).not.toHaveClass("is-highlighted");
  // 7. bhava: bhava satırı ve o bhavadaki graha (Güneş; Merkür Placidus'ta 6. bhavadadır).
  fireEvent.click(within(await bhavas()).getByRole("button", { name: "7. bhava" }));
  expect(within(await bhavas()).getByRole("button", { name: "7. bhava" }).closest("tr")).toHaveClass("is-highlighted");
  expect(row("Güneş")).toHaveClass("is-highlighted");
  expect(row("Merkür")).not.toHaveClass("is-highlighted");
});

test("unknown birth time: Rust returns no table and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeBhavaTable).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: bhavalar lagnaya bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Nokta" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeBhavaTable).not.toHaveBeenCalled();
});

test("an error is shown in the panel; retry asks again and keeps the focus in the panel", async () => {
  vi.mocked(api.computeBhavaTable).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Bhava tablosu">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await bhavas();
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bhava tablosu" }));
});

test("the point setting falls back to the start for unknown values", () => {
  expect(bhavaPointKind({})).toBe("start");
  expect(bhavaPointKind({ point: "mid" })).toBe("mid");
  expect(bhavaPointKind({ point: "end" })).toBe("end");
  expect(bhavaPointKind({ point: "cusp" })).toBe("start");
  expect(bhavaPointKind({ point: 2 })).toBe("start");
});

test("the panel is named in three languages", async () => {
  try {
    await setLanguage("en");
    renderPanel();
    expect(await screen.findByRole("table", { name: "Bhava table: start point, nakshatra, lords and arc of the twelve bhavas" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Point" })).toBeInTheDocument();
    await setLanguage("fr");
    expect(await screen.findByRole("table", { name: "Table des bhavas : début, nakshatra, maîtres et arc des douze bhavas" })).toBeInTheDocument();
    expect(screen.getByText("Méthode : bhava_table.v1")).toBeInTheDocument();
  } finally {
    await setLanguage("tr");
  }
});
