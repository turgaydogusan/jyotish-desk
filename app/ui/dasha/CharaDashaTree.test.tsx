import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { dashaChildren: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn() } }));

import { api, type Result } from "../api";
import type { CharaDasha, OpenedClient, SignDashaPath, SignDashaPeriod } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { charaChildrenOf, charaDashas, charaKey, charaYears, isCharaOpen } from "./chara";
import { DashaTreePanel } from "./DashaTreePanel";
import { dashaSystem } from "./system";

const ok = (): Result<CharaDasha | null> => ({ status: "ok", data: fixture().chara_dasha });
const okChildren = (): Result<SignDashaPeriod[]> => ({ status: "ok", data: fixture().chara_dasha_children.periods });

function renderPanel(settings: Record<string, unknown> = { system: "chara" }, path: SignDashaPath = { cycle: 1, signs: [] }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 6, h: 9, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  opened.view_state.chara_expanded_path = path;
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Dasa ağacı paneli">
      <DashaTreePanel panel={panel} />
    </section>,
    { layout, opened },
  );
}

const tree = () => screen.findByRole("tree", { name: "Dasa ağacı" });
const level = (n: number) => screen.queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(n));
const names = (items: HTMLElement[]) => items.map((i) => i.getAttribute("aria-label")!.split(",")[0]);
const click = (name: RegExp) => fireEvent.click(screen.getByRole("treeitem", { name }).querySelector(".dasha-row")!);

beforeEach(() => {
  vi.mocked(api.dashaChildren).mockReset();
  vi.mocked(api.computeCharaDasha).mockReset();
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ok());
  vi.mocked(api.charaDashaChildren).mockReset();
  vi.mocked(api.charaDashaChildren).mockImplementation(async () => okChildren());
});

test("Vimshottari is the default system; chara dasha is requested only once chosen and the choice is saved with the layout", async () => {
  const { persistence } = renderPanel({});
  const select = screen.getByRole("combobox", { name: "Sistem" });
  expect(select).toHaveDisplayValue("Vimshottari");
  expect(within(await tree()).getAllByRole("treeitem")).toHaveLength(9);
  expect(api.computeCharaDasha).not.toHaveBeenCalled();
  fireEvent.change(select, { target: { value: "chara" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ system: "chara" });
  expect([dashaSystem({}), dashaSystem({ system: "yogini" }), dashaSystem({ system: "vimshottari" }), dashaSystem({ system: "chara" })]).toEqual(["vimshottari", "vimshottari", "vimshottari", "chara"]);
});

test("chara: direction and the twelve years in a header line; rows carry sign abbreviations; the second cycle is prefixed", async () => {
  renderPanel();
  await tree();
  const opened = fixture().opened;
  expect(api.computeCharaDasha).toHaveBeenCalledTimes(1);
  expect(api.computeCharaDasha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
  // Delhi fikstürü: lagna İkizler, 9. burç Kova (ters sayan): geri.
  expect(screen.getByText("Yön: geri")).toBeInTheDocument();
  expect(screen.getByText("Yıllar: İki 6 · Boğ 6 · Koç 10 · Bal 11 · Kov 10 · Oğl 9 · Yay 4 · Akr 3 · Ter 1 · Baş 9 · Asl 8 · Yen 9")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: chara_dasha.padakrama_v1")).toBeInTheDocument();
  // On iki dasha ve ikinci çevrimde üretilen sekiz dasha; hepsi kapalı.
  const dashas = level(1);
  expect(names(dashas)).toEqual([
    ...["İki", "Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen"],
    ...["2. İki", "2. Boğ", "2. Koç", "2. Bal", "2. Kov", "2. Oğl", "2. Yay", "2. Akr"],
  ]);
  expect(dashas[0]).toHaveAccessibleName("İki, 2000-01-01 – 2006-01-01");
  expect(dashas[12]).toHaveAccessibleName("2. İki, 2086-01-01 – 2092-01-01");
  expect(dashas[12].querySelector(".dasha-lord")).toHaveTextContent("2. İki");
  expect(dashas.every((d) => d.getAttribute("aria-expanded") === "false")).toBe(true);
  expect(dashas[0]).toHaveAttribute("aria-setsize", "20");
  // Chara ağacında "yaklaşık" notu yoktur ve Vimshottari'nin alt dönem komutu çağrılmaz.
  expect(screen.queryByRole("note")).toBeNull();
  expect(api.dashaChildren).not.toHaveBeenCalled();
});

test("one open path, kept apart from the Vimshottari path and saved with the chart", async () => {
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = ["Rahu"];
  const { persistence } = renderPanel({ system: "chara" }, { cycle: 1, signs: [] }, opened);
  await tree();
  click(/^İki, /);
  // Geri yön: ilk alt dönem dasha burcundan sonraki (Boğa), sonuncusu dasha burcu (İkizler).
  expect(names(level(2))).toEqual(["İki–Boğ", "İki–Koç", "İki–Bal", "İki–Kov", "İki–Oğl", "İki–Yay", "İki–Akr", "İki–Ter", "İki–Baş", "İki–Asl", "İki–Yen", "İki–İki"]);
  expect(level(2)[0].querySelector(".dasha-lord")).toHaveTextContent("Boğ");
  // İkinci çevrimin aynı burcu ayrı bir dönemdir: açılınca ilki kapanır.
  click(/^2\. İki, /);
  expect(screen.getByRole("treeitem", { name: /^İki, / })).toHaveAttribute("aria-expanded", "false");
  expect(names(level(2))[0]).toBe("2. İki–Boğ");
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.chara_expanded_path).toEqual({ cycle: 2, signs: ["Gemini"] }));
  expect(persistence.saved.at(-1)!.viewState.dasha_expanded_path).toEqual(["Rahu"]);
  expect(persistence.saved.at(-1)!.chartId).toBe(opened.chart_record.id);
  click(/^2\. İki, /);
  expect(level(2)).toHaveLength(0);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.chara_expanded_path).toEqual({ cycle: 2, signs: [] }));
});

test("the third level loads on demand for the open path and is the last level", async () => {
  const { path, cycle } = fixture().chara_dasha_children;
  renderPanel({ system: "chara" }, { cycle, signs: path });
  await tree();
  await waitFor(() => expect(level(3)).toHaveLength(12));
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.charaDashaChildren).mock.calls[0].slice(1)).toEqual([["Gemini", "Taurus"], 1]);
  expect(names(level(3))[0]).toBe("İki–Boğ–Koç");
  expect(names(level(3))[11]).toBe("İki–Boğ–Boğ");
  expect(level(3).every((i) => !i.hasAttribute("aria-expanded"))).toBe(true);
  // Klavye yüklenen seviyeye iner; son seviyede sağ ok bir şey açmaz.
  const parent = screen.getByRole("treeitem", { name: /^İki–Boğ, / });
  act(() => parent.focus());
  fireEvent.keyDown(parent, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^İki–Boğ–Koç, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^İki–Boğ–Koç, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(document.activeElement).toBe(parent);
  // Dalı kapatıp açmak yeniden istemez; başka bir alt dönem kendi isteğini yapar.
  click(/^İki–Boğ, /);
  click(/^İki–Boğ, /);
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
});

test("a failed or empty third level is an in-tree error with retry", async () => {
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "path" } } });
  renderPanel({ system: "chara" }, { cycle: 1, signs: ["Gemini", "Taurus"] });
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: path.");
  // Boş yanıt sessizce boş dal olarak gösterilmez.
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(level(3)).toHaveLength(12));
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(3);
  expect(screen.getByRole("treeitem", { name: /^İki–Boğ, / })).toHaveAttribute("aria-expanded", "true");
});

test("a path that this chart does not have opens nothing and asks for nothing", async () => {
  // Yay ikinci çevrimde üretilir ama Terazi üretilmez (120. yıldan sonra başlardı).
  renderPanel({ system: "chara" }, { cycle: 2, signs: ["Libra", "Scorpio"] });
  await tree();
  expect(level(2)).toHaveLength(0);
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
});

test("unknown birth time: Rust returns no dasha and the panel says why; the selector stays", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValue({ status: "ok", data: null });
  renderPanel({ system: "chara" }, { cycle: 1, signs: ["Gemini", "Taurus"] }, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.")).toBeInTheDocument();
  expect(vi.mocked(api.computeCharaDasha).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("tree")).toBeNull();
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
});

test("an error is shown under the selector; retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Sistem" })).toBeInTheDocument();
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await tree();
  expect(api.computeCharaDasha).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Dasa ağacı paneli" }));
});

test("loading: nothing under a second, then a skeleton, then the tree", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<CharaDasha | null>) => void;
    vi.mocked(api.computeCharaDasha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("tree", { name: "Dasa ağacı" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("helpers group the periods Rust sent; nothing is computed here", () => {
  const dasha = fixture().chara_dasha;
  expect(charaDashas(dasha.periods)).toHaveLength(20);
  expect(charaChildrenOf(dasha.periods, 2, ["Gemini"]).map((p) => p.signs[1])).toEqual(charaChildrenOf(dasha.periods, 1, ["Gemini"]).map((p) => p.signs[1]));
  expect(charaChildrenOf(dasha.periods, 2, ["Libra"])).toEqual([]);
  expect(charaYears(dasha).map((y) => y.years)).toEqual(dasha.years);
  expect(charaYears(dasha)[0].sign).toBe("Gemini");
  expect(charaKey(2, ["Gemini", "Taurus"])).toBe("2:Gemini,Taurus");
  const open = { cycle: 2, signs: ["Gemini", "Taurus"] } satisfies SignDashaPath;
  expect([isCharaOpen(open, 2, ["Gemini"]), isCharaOpen(open, 1, ["Gemini"]), isCharaOpen(open, 2, ["Gemini", "Aries"]), isCharaOpen(open, 2, ["Gemini", "Taurus", "Aries"])]).toEqual([true, false, false, false]);
});
