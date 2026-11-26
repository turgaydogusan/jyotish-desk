import { act, fireEvent, screen, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { degreeText, JaiminiKarakasPanel, TIE_MARK } from "./JaiminiKarakasPanel";
import { arudhaLabel, KARAKA_ROLES } from "./labels";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "jk", kind: "jaimini_karakas", x: 0, y: 0, w: 5, h: 14, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<JaiminiKarakasPanel panel={panel} />, { layout, opened });
}

const karakaTable = () => screen.findByRole("table", { name: "Chara karaka: rol sırasıyla graha, burç ve burç içi derece" });
const housesTable = () => screen.getByRole("table", { name: "Grahaların karakamshadan ev sayıları: rasi (D1) ve navamsa (D9)" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("asks for the open chart's report and lists the eight karakas in role order", async () => {
  renderPanel();
  const table = await karakaTable();
  const opened = fixture().opened;
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Rol", "Graha", "Burç", "Derece"]);
  const rows = within(table).getAllByRole("row").slice(1);
  // Delhi fikstürü, sekizli şema: Rahu'nun gerçek derecesi 11°11′, etkin derecesi 30° − 11°11′.
  expect(rows.map(cells)).toEqual([
    ["AK Atmakaraka", "Ay", "Ter", "19°28′"],
    ["AmK Amatyakaraka", "Rahu", "Yen", "11°11′ (etkin 18°48′)"],
    ["BK Bhratrikaraka", "Satürn", "Koç", "16°32′"],
    ["MK Matrikaraka", "Güneş", "Yay", "16°30′"],
    ["PiK Pitrikaraka", "Merkür", "Yay", "8°02′"],
    ["PK Putrakaraka", "Venüs", "Akr", "7°42′"],
    ["GK Gnatikaraka", "Mars", "Kov", "4°06′"],
    ["DK Darakaraka", "Jüpiter", "Koç", "1°23′"],
  ]);
  // Rol kısaltmasının açılımı hem `title` hem ekran okuyucu metnidir.
  expect(within(table).getByTitle("Atmakaraka")).toHaveTextContent("AK");
  expect(screen.getByText("Rahu'nun etkin derecesi burcun sonundan ölçülür (30° − derece).")).toBeInTheDocument();
  expect(screen.queryByText(/1″ içinde eşit/)).toBeNull();
  expect(screen.getByText("Şema: Sekiz karaka (Rahu dahil)")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("karakamsha and the houses of the nine grahas from it, in D1 and D9", async () => {
  renderPanel();
  await karakaTable();
  // Atmakaraka Ay'ın navamsa burcu Balık.
  expect(document.querySelector(".jaimini-karakamsha")).toHaveTextContent("Karakamsha: Bal Atmakaraka Ay, navamsa burcu");
  const rows = within(housesTable()).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Graha", "D1 evi", "D9 evi"]);
  expect(rows.slice(1).map(cells)).toEqual([
    ["Güneş", "10", "6"],
    ["Ay", "8", "1"],
    ["Mars", "12", "9"],
    ["Merkür", "10", "4"],
    ["Jüpiter", "2", "2"],
    ["Venüs", "9", "7"],
    ["Satürn", "2", "6"],
    ["Rahu", "5", "8"],
    ["Ketu", "11", "2"],
  ]);
});

test("a tie is marked with the sign and with words, and explained once", async () => {
  const report = fixture().jaimini;
  report.karakas[2].tie = true;
  report.karakas[3].tie = true;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await karakaTable()).getAllByRole("row").slice(1);
  expect(rows[2].lastElementChild).toHaveTextContent(`16°32′ ${TIE_MARK}eşit derece`);
  expect(rows[3].lastElementChild).toHaveTextContent(`16°30′ ${TIE_MARK}eşit derece`);
  expect(rows[1].lastElementChild).not.toHaveTextContent(TIE_MARK);
  // İşaret ekran okuyucudan gizlidir; sözcük okunur.
  expect(rows[2].querySelector('[aria-hidden="true"]')).toHaveTextContent(TIE_MARK);
  expect(screen.getByText(`${TIE_MARK} sıradaki komşusuyla 1″ içinde eşit derece; sıra tam değerle belirlenir.`)).toBeInTheDocument();
});

test("the seven-karaka scheme has no Rahu row, no Pitri role and no Rahu note", async () => {
  const report = fixture().jaimini;
  const roles = ["Atma", "Amatya", "Bhratri", "Matri", "Putra", "Gnati", "Dara"] as const;
  report.scheme = "Seven";
  report.karakas = report.karakas.filter((k) => k.graha !== "Rahu").map((k, i) => ({ ...k, role: roles[i] }));
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await karakaTable()).getAllByRole("row").slice(1);
  expect(rows.map((r) => cells(r).slice(0, 2))).toEqual([
    ["AK Atmakaraka", "Ay"],
    ["AmK Amatyakaraka", "Satürn"],
    ["BK Bhratrikaraka", "Güneş"],
    ["MK Matrikaraka", "Merkür"],
    ["PK Putrakaraka", "Venüs"],
    ["GK Gnatikaraka", "Mars"],
    ["DK Darakaraka", "Jüpiter"],
  ]);
  expect(screen.queryByText(/Rahu'nun etkin derecesi/)).toBeNull();
  expect(screen.getByText("Şema: Yedi karaka (Rahu hariç)")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("unknown birth time is sent to Rust; the Moon's degree and the karakamsha are marked approximate", async () => {
  renderPanel(unknownTimeOpened());
  const table = await karakaTable();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay'ın derecesi yaklaşıktır (≈); karaka sırası ve karakamsha değişebilir.")).toBeInTheDocument();
  const rows = within(table).getAllByRole("row").slice(1);
  expect(rows[0].lastElementChild).toHaveTextContent("≈yaklaşık 19°28′");
  expect(rows[2].querySelector('[title="yaklaşık"]')).toBeNull();
  expect(document.querySelector(".jaimini-karakamsha")!.querySelector('[title="yaklaşık"]')).not.toBeNull();
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
});

test("a low-precision warning from the report is shown as text", async () => {
  const report = fixture().jaimini;
  report.warnings = ["LowPrecision"];
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  await karakaTable();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await karakaTable();
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<JaiminiReport>) => void;
    vi.mocked(api.computeJaimini).mockReturnValueOnce(new Promise((r) => (resolve = r)));
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

test("shared highlight: a graha marks its row in both tables; the karakamsha sign can be pinned", async () => {
  renderPanel();
  const table = await karakaTable();
  const saturn = within(table).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(2);
  expect(document.querySelector('tr[data-role="Bhratri"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('tr[data-house-of="Saturn"]')).toHaveClass("is-highlighted");
  // Ketu karaka değildir: yalnızca ev tablosunda satırı vardır.
  fireEvent.click(within(housesTable()).getByRole("button", { name: "Ketu" }));
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(1);
  const sign = screen.getByRole("button", { name: "Bal" });
  fireEvent.click(sign);
  expect(sign).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(0);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(sign).toHaveAttribute("aria-pressed", "false");
});

test("helpers: degree text, role names and pada labels", () => {
  expect([degreeText(16.5), degreeText(0), degreeText(null), degreeText(Number.NaN)]).toEqual(["16°30′", "0°00′", "—", "—"]);
  expect(Object.values(KARAKA_ROLES).map((r) => r.short)).toEqual(["AK", "AmK", "BK", "MK", "PiK", "PK", "GK", "DK"]);
  expect(new Set(Object.values(KARAKA_ROLES).map((r) => r.name)).size).toBe(8);
  expect([1, 2, 7, 11, 12].map(arudhaLabel)).toEqual(["AL", "A2", "A7", "A11", "UL"]);
});
