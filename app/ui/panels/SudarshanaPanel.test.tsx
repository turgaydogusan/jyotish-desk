import { fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeSudarshana: vi.fn() } }));

import { api, type Result } from "../api";
import type { SudarshanaChakra } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { SudarshanaPanel } from "./SudarshanaPanel";

const ok = (chakra: SudarshanaChakra = fixture().bhava.sudarshana): Result<SudarshanaChakra> => ({ status: "ok", data: chakra });
const PANEL: Panel = { id: "su", kind: "sudarshana", x: 0, y: 0, w: 6, h: 24, settings: {} };

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<SudarshanaPanel panel={PANEL} />, { opened });
}

const CAPTION = "Sudarshana chakra: on iki evin lagnadan, Ay'dan ve Güneş'ten burcu ve grahaları";
const table = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeSudarshana).mockReset();
  vi.mocked(api.computeSudarshana).mockImplementation(async () => ok());
});

test("draws the three rings and lists the same houses in a table", async () => {
  const { container } = renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeSudarshana).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeSudarshana).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Ev", "Lagnadan", "Ay'dan", "Güneş'ten"]);
  // Delhi fikstürü: lagna İkizler, Ay Terazi, Güneş Yay.
  expect(rows.slice(1).map(cells)).toEqual([
    ["1", "İki", "Ter Ay", "Yay Gü Me"],
    ["2", "Yen Ra", "Akr Ve", "Oğl Ke"],
    ["3", "Asl", "Yay Gü Me", "Kov Ma"],
    ["4", "Baş", "Oğl Ke", "Bal"],
    ["5", "Ter Ay", "Kov Ma", "Koç Jü Sa"],
    ["6", "Akr Ve", "Bal", "Boğ"],
    ["7", "Yay Gü Me", "Koç Jü Sa", "İki"],
    ["8", "Oğl Ke", "Boğ", "Yen Ra"],
    ["9", "Kov Ma", "İki", "Asl"],
    ["10", "Bal", "Yen Ra", "Baş"],
    ["11", "Koç Jü Sa", "Asl", "Ter Ay"],
    ["12", "Boğ", "Baş", "Akr Ve"],
  ]);
  // Çizim tablonun aynısıdır: 36 hücre, her graha üç kez, lagna işareti iç halkanın 1. evinde.
  expect(screen.getByRole("img", { name: "Sudarshana chakra: içten dışa lagna, Ay ve Güneş için üç halka" })).toBeInTheDocument();
  expect(container.querySelectorAll("svg.sudarshana path.chart-cell")).toHaveLength(36);
  expect(container.querySelectorAll("svg.sudarshana .chart-planet")).toHaveLength(27);
  for (const [ring, graha, house] of [
    ["0", "Sun", "7"],
    ["1", "Sun", "3"],
    ["2", "Sun", "1"],
    ["0", "Moon", "5"],
    ["1", "Moon", "1"],
    ["2", "Moon", "11"],
  ]) {
    expect(container.querySelector(`svg.sudarshana .chart-planet[data-ring="${ring}"][data-graha="${graha}"]`), `${ring} ${graha}`).toHaveAttribute("data-house", house);
  }
  expect(container.querySelector("svg.sudarshana .chart-mark")).toHaveTextContent("Lg");
  expect(screen.getByText(/^Üç halka içten dışa: lagna \(Lg\), Ay, Güneş\./)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: sudarshana.v1")).toBeInTheDocument();
  expect(screen.queryByRole("note")).toBeNull();
});

test("the table is the keyboard path: signs and grahas are highlight buttons with full names", async () => {
  const { container } = renderPanel();
  const t = await table();
  const group = t.closest(".bala-scroll") as HTMLElement;
  expect(group).toHaveAttribute("tabindex", "0");
  expect(group).toHaveAccessibleName(CAPTION);
  // Çizimde sekme durağı yoktur.
  expect(container.querySelectorAll("svg.sudarshana [tabindex]")).toHaveLength(0);
  const row7 = within(t).getAllByRole("row")[7];
  // Kısaltma tam adıyla okunur.
  const sun = within(row7).getByRole("button", { name: "Güneş" });
  expect(sun).toHaveTextContent("Gü");
  fireEvent.click(sun);
  expect(sun).toHaveAttribute("aria-pressed", "true");
  // Güneş üç halkada ve tablonun üç hücresinde vurgulanır.
  expect([...container.querySelectorAll("svg.sudarshana .chart-planet--highlight")].map((g) => g.getAttribute("data-graha"))).toEqual(["Sun", "Sun", "Sun"]);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Yay Gü Me", "Yay Gü Me", "Yay Gü Me"]);
  // Burç: Terazi üç halkada (5., 1. ve 11. ev).
  fireEvent.click(within(within(t).getAllByRole("row")[5]).getByRole("button", { name: "Ter" }));
  const lit = [...container.querySelectorAll("svg.sudarshana .chart-cell--highlight")].map((p) => p.parentElement!);
  expect(lit.map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-house")])).toEqual([
    ["0", "5"],
    ["1", "1"],
    ["2", "11"],
  ]);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Ter Ay", "Ter Ay", "Ter Ay"]);
});

test("hovering a cell of the drawing highlights the same sign in the table", async () => {
  const { container } = renderPanel();
  const t = await table();
  fireEvent.mouseEnter(container.querySelector('svg.sudarshana g[data-ring="2"][data-house="3"]')!);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Kov Ma", "Kov Ma", "Kov Ma"]);
  fireEvent.mouseLeave(container.querySelector('svg.sudarshana g[data-ring="2"][data-house="3"]')!);
  expect(t.querySelector("td.is-highlighted")).toBeNull();
});

test("unknown birth time: no lagna ring, the lagna column is empty and the Moon is approximate", async () => {
  vi.mocked(api.computeSudarshana).mockImplementation(async () => ok(fixture().bhava.sudarshana_time_unknown));
  const { container } = renderPanel(unknownTimeOpened());
  const rows = within(await table()).getAllByRole("row");
  expect(vi.mocked(api.computeSudarshana).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByRole("note")).toHaveTextContent("Doğum saati bilinmiyor: lagna halkası çizilmiyor. Ay'ın burcu yaklaşıktır (≈).");
  expect(container.querySelectorAll("svg.sudarshana path.chart-cell")).toHaveLength(24);
  expect(container.querySelector('svg.sudarshana [data-ring="0"]')).toBeNull();
  expect(container.querySelector("svg.sudarshana .chart-mark")).toBeNull();
  expect(cells(rows[1])).toEqual(["1", "—", "Ter Ay", "Yay Gü Me"]);
  expect(rows.slice(1).every((r) => cells(r)[1] === "—")).toBe(true);
  // "≈" yalnızca Ay sütunundadır ve sözle de okunur.
  const moon = rows[0].querySelector('th[data-ring="Moon"]')!;
  expect(moon).toHaveTextContent("≈yaklaşık Ay'dan");
  expect(rows[0].querySelector('th[data-ring="Sun"]')).toHaveTextContent(/^Güneş'ten$/);
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeSudarshana).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeSudarshana).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeSudarshana).toHaveBeenCalledTimes(2);
});

test("the panel is named in three languages and the ring names follow the Rust order", async () => {
  expect(fixture().bhava.sudarshana_refs).toEqual(["Lagna", "Moon", "Sun"]);
  try {
    await setLanguage("en");
    renderPanel();
    const t = await screen.findByRole("table", { name: "Sudarshana chakra: sign and grahas of the twelve houses from the lagna, the Moon and the Sun" });
    expect(cells(within(t).getAllByRole("row")[0])).toEqual(["House", "From lagna", "From Moon", "From Sun"]);
    await setLanguage("fr");
    expect(await screen.findByRole("img", { name: "Sudarshana chakra : anneaux du lagna, de la Lune et du Soleil, du centre vers l’extérieur" })).toBeInTheDocument();
    expect(screen.getByText("Méthode : sudarshana.v1")).toBeInTheDocument();
  } finally {
    await setLanguage("tr");
  }
});
