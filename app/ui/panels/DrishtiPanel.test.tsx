import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeDrishti: vi.fn() } }));

import { api, type Result } from "../api";
import type { DrishtiTable } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ASPECTING_GRAHAS, DrishtiPanel, drishtiValue, formatVirupa, fullAspectPairs, FULL_MARK } from "./DrishtiPanel";

const ok = (): Result<DrishtiTable> => ({ status: "ok", data: fixture().drishti });

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "dr", kind: "drishti", x: 0, y: 0, w: 6, h: 10, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<DrishtiPanel panel={panel} />, { layout, opened });
}

const rowHeader = (name: string) => within(document.querySelector("tbody")!).getByRole("button", { name });
const rowCells = (name: string) => [...rowHeader(name).closest("tr")!.querySelectorAll("td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeDrishti).mockReset();
  vi.mocked(api.computeDrishti).mockImplementation(async () => ok());
});

test("asks for the open chart's aspects and marks full sign aspects, seven rows by nine columns", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Graha bakışları: satırlar bakan, sütunlar bakılan graha" });
  const opened = fixture().opened;
  expect(api.computeDrishti).toHaveBeenCalledTimes(1);
  expect(api.computeDrishti).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Bakan", "Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"]);
  expect(screen.getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn"]);
  // Delhi fikstürü: Ay Terazi'den Koç'a (Jüpiter, Satürn); Jüpiter Koç'tan Terazi'ye (Ay) ve
  // Yay'a (Güneş, Merkür); Satürn Koç'tan Terazi'ye (Ay) ve Oğlak'a (Ketu) bakar.
  const full = `${FULL_MARK}tam bakış`;
  expect(rowCells("Güneş")).toEqual(["—", "", "", "", "", "", "", "", ""]);
  expect(rowCells("Ay")).toEqual(["", "—", "", "", full, "", full, "", ""]);
  expect(rowCells("Mars")).toEqual(["", "", "—", "", "", "", "", "", ""]);
  expect(rowCells("Jüpiter")).toEqual([full, full, "", full, "—", "", "", "", ""]);
  expect(rowCells("Satürn")).toEqual(["", full, "", "", "", "", "—", "", full]);
  // İşaret ekran okuyucudan gizlidir; "tam bakış" sözcükle okunur.
  expect(document.querySelectorAll('td [aria-hidden="true"]')).toHaveLength(7);
  expect(screen.getByText(`${FULL_MARK} burç temelli tam bakış. Rahu ve Ketu'ya graha bakışı atanmaz.`)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: graha_drishti.bphs_v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("the virupa view shows the degree-based value with one decimal", async () => {
  renderPanel({ value: "virupa" });
  await screen.findByRole("table");
  expect(screen.getByRole("combobox", { name: "Değer" })).toHaveDisplayValue("Virupa");
  expect(rowCells("Güneş")).toEqual(["—", "0.0", "8.8", "0.0", "37.6", "0.0", "30.0", "47.7", "0.0"]);
  expect(rowCells("Ay")[4]).toBe("23.9");
  expect(document.querySelector("td [aria-hidden]")).toBeNull();
  expect(screen.getByText("Derece temelli bakış değeri (sphuta drishti), virupa: 0–60.")).toBeInTheDocument();
});

test("the value setting is saved with the layout; unknown values fall back to the full mark", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(await screen.findByRole("combobox", { name: "Değer" }), { target: { value: "virupa" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ value: "virupa" });
  // Ayar değişimi yeniden hesap istemez.
  expect(api.computeDrishti).toHaveBeenCalledTimes(1);
  expect([drishtiValue({}), drishtiValue({ value: "rashi" }), drishtiValue({ value: "virupa" })]).toEqual(["full", "full", "virupa"]);
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeDrishti).not.toHaveBeenCalled();
});

test("unknown birth time is sent to Rust and the Moon is marked approximate", async () => {
  renderPanel({}, unknownTimeOpened());
  await screen.findByRole("table");
  expect(vi.mocked(api.computeDrishti).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay'ın konumu ve bakışları yaklaşıktır (≈).")).toBeInTheDocument();
  // Ay'ın satır ve sütun başlığı işaretlidir; başka graha değil.
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
  expect(rowHeader("Ay").closest("th")!.querySelector('[title="yaklaşık"]')).not.toBeNull();
  expect(rowHeader("Güneş").closest("th")!.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeDrishti).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(api.computeDrishti).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeDrishti).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  const panel: Panel = { id: "dr", kind: "drishti", x: 0, y: 0, w: 6, h: 10, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Bakışlar">
      <DrishtiPanel panel={panel} />
    </section>,
    { layout, opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  expect(document.activeElement).toBe(retry);
  fireEvent.click(retry);
  await screen.findByRole("table");
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bakışlar" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<DrishtiTable>) => void;
    vi.mocked(api.computeDrishti).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.queryByRole("table")).toBeNull();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks its row and its column; nodes have a column only", async () => {
  renderPanel();
  await screen.findByRole("table");
  const marked = () => document.querySelectorAll("td.is-highlighted").length;
  fireEvent.click(rowHeader("Jüpiter"));
  expect(rowHeader("Jüpiter")).toHaveAttribute("aria-pressed", "true");
  expect(rowHeader("Jüpiter").closest("tr")).toHaveClass("is-highlighted");
  expect(document.querySelector('thead th[data-graha="Jupiter"]')).toHaveClass("is-highlighted");
  // Sütun: yedi satırın Jüpiter hücresi.
  expect(marked()).toBe(7);
  // Sütun başlığı kısaltmadır; erişilebilir adı grahanın tam adıdır.
  const ketu = within(document.querySelector("thead")!).getByRole("button", { name: "Ketu" });
  expect(ketu).toHaveTextContent("Ke");
  fireEvent.click(ketu);
  expect(document.querySelector("tbody tr.is-highlighted")).toBeNull();
  expect(document.querySelector('thead th[data-graha="Ketu"]')).toHaveClass("is-highlighted");
  expect(marked()).toBe(7);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toBe(0);
});

test("helpers: full-aspect pairs come from the occupants, virupa text has one decimal", () => {
  const pairs = fullAspectPairs(fixture().drishti);
  expect([...pairs].sort()).toEqual(["Jupiter>Mercury", "Jupiter>Moon", "Jupiter>Sun", "Moon>Jupiter", "Moon>Saturn", "Saturn>Ketu", "Saturn>Moon"]);
  expect([formatVirupa(0), formatVirupa(60), formatVirupa(37.55), formatVirupa(null), formatVirupa(Number.NaN)]).toEqual(["0.0", "60.0", "37.5", "—", "—"]);
  expect(ASPECTING_GRAHAS).toEqual(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"]);
});
