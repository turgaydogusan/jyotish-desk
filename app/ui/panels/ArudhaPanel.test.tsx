import { fireEvent, screen, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ArudhaPanel, EXCEPTION_MARK } from "./ArudhaPanel";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "ar", kind: "arudha", x: 0, y: 0, w: 4, h: 12, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<ArudhaPanel panel={panel} />, { layout, opened });
}

const table = () => screen.findByRole("table", { name: "Arudha padalar: on iki evin padası, burcu ve evin lordu" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("lists the twelve padas with sign and lord; AL and UL are labelled; exceptions carry a mark and a note", async () => {
  renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Pada", "Burç", "Evin lordu"]);
  const x = `${EXCEPTION_MARK} istisna`;
  // Delhi fikstürü, İkizler lagna, eş lordlar açık (6. ev Akrep: Mars; 9. ev Kova: Satürn).
  expect(rows.slice(1).map(cells)).toEqual([
    ["A1 · AL", `Bal${x}`, "Merkür"],
    ["A2", `Ter${x}`, "Ay"],
    ["A3", "Koç", "Güneş"],
    ["A4", `Yay${x}`, "Merkür"],
    ["A5", "Yay", "Venüs"],
    ["A6", `Kov${x}`, "Mars"],
    ["A7", "Asl", "Jüpiter"],
    ["A8", `Koç${x}`, "Satürn"],
    ["A9", "İki", "Satürn"],
    ["A10", "Boğ", "Jüpiter"],
    ["A11", "Yay", "Mars"],
    ["A12 · UL", `Kov${x}`, "Venüs"],
  ]);
  // İşaret ekran okuyucudan gizlidir; "istisna" sözcükle okunur.
  expect(document.querySelectorAll('td [aria-hidden="true"]')).toHaveLength(6);
  expect(screen.getByText(`${EXCEPTION_MARK} İstisna: hesaplanan burç evin kendisi ya da 7. burcuydu; pada ondan 10. burçtur.`)).toBeInTheDocument();
  expect(screen.getByText("Jaimini eş lordları açık: Akrep ve Kova'nın lordu, güçlü olan eş lorddur.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
});

test("without exceptions there is no note; switched-off co-lords are stated", async () => {
  const report = fixture().jaimini;
  report.co_lords = false;
  report.arudhas = report.arudhas!.map((p) => ({ ...p, exception: false })) as JaiminiReport["arudhas"];
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  await table();
  expect(screen.queryByText(/İstisna:/)).toBeNull();
  expect(document.querySelector('td [aria-hidden="true"]')).toBeNull();
  expect(screen.getByText("Jaimini eş lordları devre dışı: Akrep'in lordu Mars, Kova'nın lordu Satürn'dür.")).toBeInTheDocument();
});

test("unknown birth time: Rust returns no padas and the panel says why", async () => {
  const report = fixture().jaimini;
  report.arudhas = null;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel(unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: arudha padalar lagnaya bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
});

test("shared highlight: a sign marks every pada in it, a graha every pada it rules, a house its sign", async () => {
  renderPanel();
  const t = await table();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.pada);
  // Yay'da üç pada var: A4, A5, A11.
  const sag = within(t).getAllByRole("button", { name: "Yay" })[0];
  fireEvent.click(sag);
  expect(sag).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["4", "5", "11"]);
  // Satürn iki evin lordu: A8, A9.
  fireEvent.click(within(t).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["8", "9"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});
