import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ArgalaPanel, argalaReference, selectedSign } from "./ArgalaPanel";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "ag", kind: "argala", x: 0, y: 0, w: 6, h: 9, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<ArgalaPanel panel={panel} />, { layout, opened });
}

const rows = () => [...document.querySelectorAll("tbody tr")].map((r) => [...r.querySelectorAll("th, td")].map((c) => c.textContent));
const signs = () => screen.getByRole("group", { name: "Referans burcu seç" });

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("by default the reference is the lagna sign: four pairs, two graha lists and the status as text", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Argala, referans İki: argala veren evler, engelleyen evler ve durum" });
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("combobox", { name: "Referans" })).toHaveDisplayValue("Lagna");
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Argala evi", "Argala grahaları", "Engel evi", "Engel grahaları", "Durum"]);
  // Delhi fikstürü, İkizler'den: 2 Yengeç (Rahu) ↔ 12 Boğa; 4 Başak ↔ 10 Balık; 11 Koç (Jüpiter,
  // Satürn) ↔ 3 Aslan; 5 Terazi (Ay) ↔ 9 Kova (Mars).
  expect(rows()).toEqual([
    ["2", "Ra", "12", "—", "geçerli"],
    ["4", "—", "10", "—", "yok"],
    ["11", "Jü Sa", "3", "—", "geçerli"],
    ["5", "Ay", "9", "Ma", "çekişmeli"],
  ]);
  expect(screen.getByText("Referans: İki")).toHaveTextContent("Referans: İki Lg");
  // Kısaltmanın erişilebilir adı grahanın tam adıdır.
  expect(screen.getByRole("button", { name: "Jüpiter" })).toHaveTextContent("Jü");
  expect(screen.queryByRole("group", { name: "Referans burcu seç" })).toBeNull();
  expect(screen.queryByText(/kötücül argalası/)).toBeNull();
  expect(screen.getByText("Durum yalnızca graha sayısına bakar; güç karşılaştırması yapılmaz.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
});

test("selected sign: nothing pinned asks for a sign; picking one shows its row", async () => {
  renderPanel({ reference: "selected" });
  expect(await screen.findByText(/^Bir burç seçin:/)).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(within(signs()).getAllByRole("button").map((b) => b.textContent)).toEqual(["Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay", "Oğl", "Kov", "Bal"]);
  const cancer = within(signs()).getByRole("button", { name: "Yen" });
  fireEvent.click(cancer);
  expect(cancer).toHaveAttribute("aria-pressed", "true");
  await screen.findByRole("table", { name: /^Argala, referans Yen:/ });
  // Yengeç'ten: 4 Terazi (Ay) ↔ 10 Koç (Jüpiter, Satürn) engelli; 5 Akrep (Venüs) ↔ 9 Balık.
  expect(rows()).toEqual([
    ["2", "—", "12", "—", "yok"],
    ["4", "Ay", "10", "Jü Sa", "engelli"],
    ["11", "—", "3", "—", "yok"],
    ["5", "Ve", "9", "—", "geçerli"],
  ]);
  expect(screen.getByText("Referans: Yen")).toHaveTextContent(/^Referans: Yen$/);
  expect(screen.queryByText(/^Bir burç seçin:/)).toBeNull();
  // Aynı komut yeniden çağrılmaz: on iki satır tek raporda gelir.
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  // Tablodaki bir graha sabitlenince referans o grahanın burcu olur: Venüs Akrep'te.
  fireEvent.click(screen.getByRole("button", { name: "Venüs" }));
  await screen.findByRole("table", { name: /^Argala, referans Akr:/ });
  fireEvent.keyDown(window, { key: "Escape" });
  expect(await screen.findByText(/^Bir burç seçin:/)).toBeInTheDocument();
});

test("the reference setting is saved with the layout; unknown values fall back to the lagna", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(await screen.findByRole("combobox", { name: "Referans" }), { target: { value: "selected" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ reference: "selected" });
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect([argalaReference({}), argalaReference({ reference: "moon" }), argalaReference({ reference: "selected" })]).toEqual(["lagna", "lagna", "selected"]);
});

test("unknown birth time: no lagna reference, but a selected sign still has its row", async () => {
  const report = fixture().jaimini;
  // Rust saat bilinmeyince padaları boş, satırları Koç'tan sıralı döner.
  report.arudhas = null;
  report.argala = [...report.argala.slice(10), ...report.argala.slice(0, 10)] as JaiminiReport["argala"];
  expect(report.argala[0].reference).toBe("Aries");
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  const first = renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: lagna referansı yok. Referans olarak “Seçili burç”u kullanın.")).toBeInTheDocument();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  first.unmount();
  renderPanel({ reference: "selected" }, unknownTimeOpened());
  fireEvent.click(within(await screen.findByRole("group", { name: "Referans burcu seç" })).getByRole("button", { name: "İki" }));
  await screen.findByRole("table", { name: /^Argala, referans İki:/ });
  expect(rows()[0]).toEqual(["2", "Ra", "12", "—", "geçerli"]);
  // Lagna bilinmediği için "Lg" işareti yazılmaz.
  expect(screen.getByText("Referans: İki")).toHaveTextContent(/^Referans: İki$/);
});

test("three malefics in the third house are reported as text", async () => {
  const report = fixture().jaimini;
  report.argala[0].third_house_malefic_argala = true;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  expect(await screen.findByText("3. evde üç ya da daha çok doğal kötücül var: 3. ev kötücül argalası.")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  const panel: Panel = { id: "ag", kind: "argala", x: 0, y: 0, w: 6, h: 9, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Argala">
      <ArgalaPanel panel={panel} />
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
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Argala" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<JaiminiReport>) => void;
    vi.mocked(api.computeJaimini).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
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

test("helper: the sign of the pinned target", () => {
  const opened = fixture().opened;
  expect(selectedSign(null, opened, 2)).toBeNull();
  expect(selectedSign({ type: "sign", sign: 7 }, opened, 2)).toBe(7);
  // Ev: lagna burcundan sayılır; lagna yoksa haritanın çizdiği gibi Koç'tan.
  expect(selectedSign({ type: "house", house: 1 }, opened, 2)).toBe(2);
  expect(selectedSign({ type: "house", house: 12 }, opened, 2)).toBe(1);
  expect(selectedSign({ type: "house", house: 4 }, opened, null)).toBe(3);
  // Graha: haritadaki burcu (Ay Terazi'de).
  expect(selectedSign({ type: "graha", id: "Moon" }, opened, 2)).toBe(6);
});
