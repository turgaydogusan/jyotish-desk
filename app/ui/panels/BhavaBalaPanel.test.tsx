import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeShadbala: vi.fn() } }));

import { api, type Result } from "../api";
import type { ShadbalaReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { BhavaBalaPanel } from "./BhavaBalaPanel";

const ok = (report: ShadbalaReport | null = fixture().shadbala): Result<ShadbalaReport | null> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "bb", kind: "bhava_bala", x: 0, y: 0, w: 6, h: 14, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <BhavaBalaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const table = () => screen.findByRole("table", { name: "Bhava bala: on iki bhavanın lordu, üç kalemi, toplamı ve sırası" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeShadbala).mockReset();
  vi.mocked(api.computeShadbala).mockImplementation(async () => ok());
});

test("lists the twelve bhavas with lord, three items, total and rank from the shadbala report", async () => {
  renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Bhava", "Lord", "Bhavadhipati", "Dig", "Drishti", "Toplam", "Sıra"]);
  // Delhi fikstürü, İkizler lagna 16°: 1. bhava insan burcu (dig 60), 10. bhava Balık (dig 0).
  expect(rows.slice(1).map(cells)).toEqual([
    ["1", "Merkür", "454.1", "60.0", "58.5", "572.6", "2"],
    ["2", "Ay", "413.5", "40.0", "47.4", "500.9", "6"],
    ["3", "Güneş", "505.3", "10.0", "49.5", "564.8", "3"],
    ["4", "Merkür", "454.1", "30.0", "28.2", "512.3", "5"],
    ["5", "Venüs", "371.0", "20.0", "44.8", "435.8", "9"],
    ["6", "Mars", "412.6", "50.0", "33.4", "496.0", "7"],
    ["7", "Jüpiter", "533.1", "30.0", "55.9", "619.0", "1"],
    ["8", "Satürn", "353.7", "20.0", "4.2", "377.9", "12"],
    ["9", "Satürn", "353.7", "20.0", "4.8", "378.4", "11"],
    ["10", "Jüpiter", "533.1", "0.0", "22.5", "555.6", "4"],
    ["11", "Mars", "412.6", "50.0", "−7.2", "455.4", "8"],
    ["12", "Venüs", "371.0", "40.0", "10.6", "421.6", "10"],
  ]);
  // Bhava numarası tam adıyla okunur.
  expect(within(rows[1]).getByRole("button", { name: "1. bhava" })).toHaveTextContent("1");
  expect(screen.getByText(/^Bhavalar eşittir: bhava ortası lagnadan 30°'lik adımlarla sayılır/)).toBeInTheDocument();
  expect(screen.getByText("Değerler virupa cinsindendir (60 virupa = 1 rupa).")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: shadbala.bphs_v1")).toBeInTheDocument();
});

test("the table sits in a keyboard-focusable scroll group named by its caption", async () => {
  renderPanel();
  const name = "Bhava bala: on iki bhavanın lordu, üç kalemi, toplamı ve sırası";
  const table = await screen.findByRole("table", { name });
  const group = screen.getByRole("group", { name });
  expect(group).toHaveClass("bala-scroll");
  expect(group).toHaveAttribute("tabindex", "0");
  expect(group).toContainElement(table);
});

test("the unit is saved with the layout and redraws the values in rupas", async () => {
  const { persistence } = renderPanel();
  const t = await table();
  fireEvent.change(screen.getByRole("combobox", { name: "Birim" }), { target: { value: "rupa" } });
  await waitFor(() => expect(cells(within(t).getAllByRole("row")[1])).toEqual(["1", "Merkür", "7.57", "1.00", "0.98", "9.54", "2"]));
  expect(cells(within(t).getAllByRole("row")[2])).toEqual(["2", "Ay", "6.89", "0.67", "0.79", "8.35", "6"]);
  expect(screen.getByText("Değerler rupa cinsindendir (1 rupa = 60 virupa).")).toBeInTheDocument();
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ unit: "rupa" }));
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
});

test("unknown birth time: Rust returns no report and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: Shadbala ve Bhava bala doğum saatine bağlı olduğu için hesaplanmıyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
});

test("no sunrise or sunset: with a known birth time the missing report is explained by the Sun", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel();
  expect(
    await screen.findByText("Doğum yerinde o gün gün doğumu ya da batımı yok: zaman gücü ölçülemediği için Shadbala ve Bhava bala hesaplanmıyor."),
  ).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeShadbala).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeShadbala).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Bhava bala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await table();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bhava bala" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<ShadbalaReport | null>) => void;
    vi.mocked(api.computeShadbala).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a bhava marks its row, a graha every bhava it rules", async () => {
  renderPanel();
  const t = await table();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.house);
  const seventh = within(t).getByRole("button", { name: "7. bhava" });
  fireEvent.click(seventh);
  expect(seventh).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["7"]);
  // Satürn iki bhavanın lordudur: 8 ve 9.
  fireEvent.click(within(t).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["8", "9"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});

test("a number prashna has no meridian: the panel says so and asks for no report", () => {
  renderPanel({}, fixture().prashna.opened_number);
  expect(screen.getByRole("note")).toHaveTextContent("Sayı prashnasında lagna sayıdan gelir; meridyen (MC) tanımsızdır. Shadbala ve Bhava bala hesaplanmaz.");
  expect(api.computeShadbala).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});
