import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn(), charaDashaAt: vi.fn() } }));

import { api, type Result } from "../api";
import type { CharaDasha, OpenedClient, SignDashaPath, SignDashaPeriod, TransitAt } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { DashaStripPanel } from "./DashaStripPanel";
import { stripLevel } from "./system";

function renderStrip(settings: Record<string, unknown> = { system: "chara" }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "ds", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Dasa şeridi paneli">
      <DashaStripPanel panel={panel} />
    </section>,
    { layout, opened },
  );
}

const list = () => screen.findByRole("list", { name: /Dasa şeridi/ });
const buttons = async () => within(await list()).getAllByRole("button");
const names = (items: HTMLElement[]) => items.map((b) => b.querySelector(".dasha-strip-name")!.textContent);
const current = (items: HTMLElement[]) => items.filter((b) => b.getAttribute("aria-current") === "true");

beforeEach(() => {
  // Transit anı: istenen an bir JD ise yanıt o anı taşır; değilse fikstürün "şimdi"si.
  vi.mocked(api.transitsAt).mockImplementation(async (at: TransitAt) => {
    const view = fixture().transit;
    if (typeof at === "object" && "Jd" in at) view.snapshot.jd_ut = at.Jd ?? null;
    return { status: "ok", data: view };
  });
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha }));
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha_at }));
  vi.mocked(api.charaDashaChildren).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha_at_children }));
});
afterEach(() => {
  for (const f of [api.transitsAt, api.computeCharaDasha, api.charaDashaAt, api.charaDashaChildren]) vi.mocked(f).mockReset();
});

test("Vimshottari is the default system; chara dasha is requested only once chosen and both settings are saved", async () => {
  const { persistence } = renderStrip({ level: "maha" });
  expect(await buttons()).toHaveLength(9);
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Vimshottari");
  expect(api.computeCharaDasha).not.toHaveBeenCalled();
  expect(api.charaDashaAt).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("combobox", { name: "Sistem" }), { target: { value: "chara" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ level: "maha", system: "chara" });
  expect([stripLevel({}), stripLevel({ level: "prana" }), stripLevel({ level: "maha" })]).toEqual(["antar", "antar", "maha"]);
});

test("chara, antar (default): the twelve sub-periods of the dasha at the transit moment, the current one marked", async () => {
  renderStrip();
  const items = await buttons();
  const opened = fixture().opened;
  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true };
  expect(api.computeCharaDasha).toHaveBeenCalledWith(input);
  // Güncel dönem Rust'tan gelir: transit anı (`snapshot.jd_ut`) ile sorulur.
  expect(api.charaDashaAt).toHaveBeenCalledTimes(1);
  expect(api.charaDashaAt).toHaveBeenCalledWith(input, fixture().transit.snapshot.jd_ut);
  // Fikstür: an ilk çevrimin Balık dashasında, Başak alt döneminde. Yön geri: Kova … Balık.
  expect(fixture().chara_dasha_at).toEqual({ cycle: 1, signs: ["Pisces", "Virgo", "Aries"] });
  expect(names(items)).toEqual(["Bal–Kov", "Bal–Oğl", "Bal–Yay", "Bal–Akr", "Bal–Ter", "Bal–Baş", "Bal–Asl", "Bal–Yen", "Bal–İki", "Bal–Boğ", "Bal–Koç", "Bal–Bal"]);
  expect(current(items)).toHaveLength(1);
  expect(current(items)[0]).toHaveAttribute("data-sign", "Virgo");
  expect(current(items)[0]).toHaveTextContent("transit anında");
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
});

test("chara, maha: both cycles in one strip, the second one prefixed", async () => {
  renderStrip({ system: "chara", level: "maha" });
  const items = await buttons();
  expect(names(items)).toEqual([
    ...["İki", "Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen"],
    ...["2. İki", "2. Boğ", "2. Koç", "2. Bal", "2. Kov", "2. Oğl", "2. Yay", "2. Akr"],
  ]);
  // İlk çevrimin Balık'ı günceldir; ikinci çevrimin Balık'ı değildir.
  expect(current(items).map((b) => [b.getAttribute("data-sign"), b.getAttribute("data-cycle")])).toEqual([["Pisces", "1"]]);
  expect(items[0]).toHaveTextContent("2000-01-01");
});

test("chara, pratyantar: the third level is asked for the sub-period at the transit moment; choosing a period moves the transit moment", async () => {
  renderStrip({ system: "chara", level: "pratyantar" });
  const items = await buttons();
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.charaDashaChildren).mock.calls[0].slice(1)).toEqual([["Pisces", "Virgo"], 1]);
  expect(items).toHaveLength(12);
  expect(names(items)[0]).toBe("Bal–Baş–Asl");
  expect(names(items)[11]).toBe("Bal–Baş–Baş");
  expect(current(items).map((b) => b.getAttribute("data-sign"))).toEqual(["Aries"]);
  // Tıklama transit anını dönemin başlangıcına taşır; yeni anın yolu yeniden sorulur.
  const start = fixture().chara_dasha_at_children[0].start_jd_ut;
  fireEvent.click(items[0]);
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: start }));
  await waitFor(() => expect(vi.mocked(api.charaDashaAt).mock.calls.at(-1)?.[1]).toBe(start));
  expect(api.charaDashaAt).toHaveBeenCalledTimes(2);
});

test("the current period follows the path Rust returns for the new moment", async () => {
  renderStrip();
  const items = await buttons();
  // Yeni an için Rust başka bir dasha döner: şerit o dashanın alt dönemlerine geçer.
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: { cycle: 2, signs: ["Gemini", "Aries", "Pisces"] } }));
  fireEvent.click(items[0]);
  await waitFor(async () => expect(names(await buttons())[0]).toBe("2. İki–Boğ"));
  const next = await buttons();
  expect(names(next)).toEqual(["Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen", "İki"].map((s) => `2. İki–${s}`));
  expect(current(next).map((b) => [b.getAttribute("data-sign"), b.getAttribute("data-cycle")])).toEqual([["Aries", "2"]]);
});

test("choosing a period keeps the strip and the keyboard focus while the path of the new moment is asked", async () => {
  renderStrip();
  const items = await buttons();
  let resolve!: (r: Result<SignDashaPath | null>) => void;
  vi.mocked(api.charaDashaAt).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  items[1].focus();
  fireEvent.click(items[1]);
  await waitFor(() => expect(api.charaDashaAt).toHaveBeenCalledTimes(2));
  // The answer is still pending: the last settled strip stays, marked busy, and the focus is where it was.
  const strip = screen.getByRole("list", { name: /Dasa şeridi/ });
  expect(strip).toHaveAttribute("aria-busy", "true");
  expect(within(strip).getAllByRole("button")).toHaveLength(12);
  expect(document.activeElement).toBe(items[1]);
  expect(current(within(strip).getAllByRole("button")).map((b) => b.getAttribute("data-sign"))).toEqual(["Virgo"]);
  await act(async () => resolve({ status: "ok", data: { cycle: 1, signs: ["Pisces", "Capricorn", "Aries"] } }));
  await waitFor(() => expect(screen.getByRole("list", { name: /Dasa şeridi/ })).not.toHaveAttribute("aria-busy"));
  expect(document.activeElement).toBe(items[1]);
  expect(items[1]).toHaveAttribute("aria-current", "true");
  expect(items[1]).toHaveTextContent("transit anında");
});

test("pratyantar: the strip and the focus stay while the path and then the third level are asked again", async () => {
  renderStrip({ system: "chara", level: "pratyantar" });
  const items = await buttons();
  let resolve!: (r: Result<SignDashaPeriod[]>) => void;
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: { cycle: 1, signs: ["Pisces", "Leo", "Aries"] } }));
  vi.mocked(api.charaDashaChildren).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  items[2].focus();
  fireEvent.click(items[2]);
  await waitFor(() => expect(api.charaDashaChildren).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.charaDashaChildren).mock.calls[1].slice(1)).toEqual([["Pisces", "Leo"], 1]);
  expect(screen.getByRole("list", { name: /Dasa şeridi/ })).toHaveAttribute("aria-busy", "true");
  expect(document.activeElement).toBe(items[2]);
  expect(names(await buttons())[0]).toBe("Bal–Baş–Asl");
  await act(async () => resolve({ status: "ok", data: fixture().chara_dasha_at_children.map((p) => ({ ...p, signs: ["Pisces", "Leo", p.signs[2]] })) }));
  // Another sub-period's list: the chosen button is gone, the focus moves to the current period.
  await waitFor(async () => expect(names(await buttons())[0]).toBe("Bal–Asl–Asl"));
  expect(current(await buttons())).toHaveLength(1);
  expect(document.activeElement).toBe(current(await buttons())[0]);
});

test("when the chosen period leaves the strip the focus moves to the current period; to the panel when the strip gives way to an error", async () => {
  renderStrip();
  const items = await buttons();
  vi.mocked(api.charaDashaAt).mockImplementationOnce(async () => ({ status: "ok", data: { cycle: 2, signs: ["Gemini", "Aries", "Pisces"] } }));
  items[0].focus();
  fireEvent.click(items[0]);
  await waitFor(async () => expect(names(await buttons())[0]).toBe("2. İki–Boğ"));
  const next = await buttons();
  expect(current(next)).toHaveLength(1);
  expect(document.activeElement).toBe(current(next)[0]);
  // The next request fails: the strip is replaced by the error and the focus stays in the panel.
  vi.mocked(api.charaDashaAt).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  fireEvent.click(current(next)[0]);
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Dasa şeridi paneli" }));
});

test("a mouse user who never focused the strip does not get the focus moved", async () => {
  renderStrip();
  const items = await buttons();
  const system = screen.getByRole("combobox", { name: "Sistem" });
  system.focus();
  vi.mocked(api.charaDashaAt).mockImplementationOnce(async () => ({ status: "ok", data: { cycle: 2, signs: ["Gemini", "Aries", "Pisces"] } }));
  fireEvent.click(items[0]);
  await waitFor(async () => expect(names(await buttons())[0]).toBe("2. İki–Boğ"));
  expect(document.activeElement).toBe(system);
});

test("a moment outside the produced periods: a message instead of the strip", async () => {
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: null }));
  renderStrip();
  expect(await screen.findByText("Transit anı chara dasha dönemlerinin dışında.")).toBeInTheDocument();
  expect(screen.queryByRole("list")).toBeNull();
});

test("unknown birth time: Rust returns no dasha, the panel says why and asks for no path", async () => {
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ({ status: "ok", data: null }));
  renderStrip({ system: "chara" }, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.")).toBeInTheDocument();
  expect(vi.mocked(api.computeCharaDasha).mock.calls[0][0].time_known).toBe(false);
  expect(api.charaDashaAt).not.toHaveBeenCalled();
  expect(screen.queryByRole("list")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
});

test("errors keep the selectors; retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderStrip();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Sistem" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Seviye" })).toBeInTheDocument();
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(await buttons()).toHaveLength(12);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Dasa şeridi paneli" }));
});

test("a failed path or third-level request is an error with retry; an empty third level is not a silent empty strip", async () => {
  vi.mocked(api.charaDashaAt).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "jd_ut" } } });
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  renderStrip({ system: "chara", level: "pratyantar" });
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: jd_ut.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await buttons()).toHaveLength(12);
});

test("loading: nothing under a second, then a skeleton, then the strip", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<CharaDasha | null>) => void;
    vi.mocked(api.computeCharaDasha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderStrip();
    await act(async () => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    await act(async () => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve({ status: "ok", data: fixture().chara_dasha }));
    await act(async () => {});
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("list", { name: /Dasa şeridi/ })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("no open chart: a message, no request", () => {
  renderWithStores(<DashaStripPanel panel={{ id: "x", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings: { system: "chara" } }} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  for (const f of [api.transitsAt, api.computeCharaDasha, api.charaDashaAt, api.charaDashaChildren]) expect(f).not.toHaveBeenCalled();
});
