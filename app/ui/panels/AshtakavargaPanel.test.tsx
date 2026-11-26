import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, test } from "vitest";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ashtakavargaFormat, AshtakavargaPanel, ashtakavargaView, BAV_GRAHAS } from "./AshtakavargaPanel";

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "av", kind: "ashtakavarga", x: 0, y: 0, w: 6, h: 11, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(
    <>
      <HousePin />
      <AshtakavargaPanel panel={panel} />
    </>,
    { layout, opened },
  );
}

/** Harita panelinin yerine: 1. evi vurgular. */
function HousePin() {
  const h = useHighlight();
  return <button onClick={() => h.togglePin({ type: "house", house: 1 })}>pin house 1</button>;
}

const SIGNS = ["Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay", "Oğl", "Kov", "Bal"];
const rowCells = (name: string) => [...screen.getByRole("rowheader", { name }).closest("tr")!.querySelectorAll("td")].map((c) => c.textContent);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

test("default view: seven BAV rows and the SAV row, twelve sign columns from Aries, row totals", () => {
  renderPanel();
  // Lagna İkizler: burç başlığında "Lg" metniyle işaretli.
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Koç", "Boğ", "İki Lg", ...SIGNS.slice(3), "Toplam"]);
  // tests/golden/new-delhi-2000-01-01.json: SAV birebir.
  expect(rowCells("SAV")).toEqual(["29", "21", "23", "26", "34", "33", "30", "30", "28", "21", "32", "30", "337"]);
  expect(rowCells("Güneş")).toEqual(["4", "5", "1", "3", "6", "5", "5", "4", "5", "2", "4", "4", "48"]);
  // BPHS: her BAV'ın toplamı yerleşimden bağımsızdır (Rust `BAV_TOTALS`).
  expect(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn"].map((g) => rowCells(g).at(-1))).toEqual(["48", "49", "39", "54", "56", "52", "39"]);
  expect(screen.getAllByRole("row")).toHaveLength(9);
  expect(screen.getByRole("table", { name: "Ashtakavarga, SAV: burç başına bindu" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Görünüm" })).toHaveDisplayValue("SAV");
  expect(screen.getByRole("combobox", { name: "Biçim" })).toHaveDisplayValue("Tablo");
});

test("a single graha view shows only that BAV row", () => {
  renderPanel({ view: "Moon" });
  expect(screen.getByRole("combobox", { name: "Görünüm" })).toHaveDisplayValue("Ay BAV");
  expect(screen.getAllByRole("row")).toHaveLength(2);
  expect(rowCells("Ay")).toEqual(["5", "2", "4", "6", "4", "4", "5", "3", "3", "2", "5", "6", "49"]);
  expect(screen.queryByRole("rowheader", { name: "SAV" })).toBeNull();
});

test("in-chart format: twelve fixed sign boxes with the chosen view's numbers", () => {
  renderPanel({ format: "chart" });
  const group = screen.getByRole("group", { name: "Ashtakavarga, SAV: burç başına bindu" });
  const items = [...group.querySelectorAll<HTMLElement>("li[data-sign]")];
  expect(items.map((li) => li.textContent)).toEqual(["Koç29", "Boğ21", "İki Lg23", "Yen26", "Asl34", "Baş33", "Ter30", "Akr30", "Yay28", "Oğl21", "Kov32", "Bal30"]);
  // Güney düzeni: Balık sol üst, Koç onun sağı, Terazi alt sırada.
  const cell = (sign: number) => [items[sign].style.gridRow, items[sign].style.gridColumn];
  expect([cell(11), cell(0), cell(3), cell(6), cell(9)]).toEqual([
    ["1", "1"],
    ["1", "2"],
    ["2", "4"],
    ["4", "3"],
    ["3", "1"],
  ]);
  expect(within(group).getByText("SAV toplamı 337")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("in-chart format follows the view", () => {
  renderPanel({ format: "chart", view: "Saturn" });
  const group = screen.getByRole("group", { name: "Ashtakavarga, Satürn BAV: burç başına bindu" });
  expect([...group.querySelectorAll(".av-cell-value")].map((v) => v.textContent)).toEqual(["3", "1", "4", "3", "5", "6", "3", "3", "3", "2", "2", "4"]);
  expect(within(group).getByText("Satürn BAV toplamı 39")).toBeInTheDocument();
});

test("view and format are panel settings, saved with the layout; unknown values fall back", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(screen.getByRole("combobox", { name: "Görünüm" }), { target: { value: "Jupiter" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(savedSettings(persistence)).toEqual({ view: "Jupiter" });
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "chart" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ view: "Jupiter", format: "chart" }));
  expect(ashtakavargaView({ view: "Rahu" })).toBe("sav");
  expect(ashtakavargaView({ view: 3 })).toBe("sav");
  expect(ashtakavargaView({})).toBe("sav");
  expect(BAV_GRAHAS.map((g) => ashtakavargaView({ view: g }))).toEqual([...BAV_GRAHAS]);
  expect([ashtakavargaFormat({}), ashtakavargaFormat({ format: "pie" }), ashtakavargaFormat({ format: "chart" })]).toEqual(["table", "table", "chart"]);
});

test("without an open chart, and with an unknown birth time, no numbers are shown", () => {
  const first = renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  first.unmount();
  renderPanel({}, unknownTimeOpened());
  expect(screen.getByText("Doğum saati bilinmiyor: ashtakavarga lagnaya da bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.queryByRole("combobox")).toBeNull();
});

test("shared highlight: a graha marks its row, a sign or a house its column", () => {
  renderPanel();
  const marked = () => document.querySelectorAll("td.is-highlighted").length;
  const mars = screen.getByRole("button", { name: "Mars" });
  fireEvent.click(mars);
  expect(mars).toHaveAttribute("aria-pressed", "true");
  expect(mars.closest("tr")).toHaveClass("is-highlighted");
  expect(screen.getByRole("button", { name: "Güneş" }).closest("tr")).not.toHaveClass("is-highlighted");
  // Burç başlığı: yedi BAV hücresi ve SAV hücresi.
  const leo = screen.getByRole("button", { name: "Asl" });
  fireEvent.click(leo);
  expect(leo).toHaveAttribute("aria-pressed", "true");
  expect(mars.closest("tr")).not.toHaveClass("is-highlighted");
  expect(marked()).toBe(8);
  expect(document.querySelector('th[data-sign="4"]')).toHaveClass("is-highlighted");
  expect(screen.getByRole("rowheader", { name: "SAV" }).closest("tr")!.querySelector("td.is-highlighted")).toHaveTextContent("34");
  // 1. ev: lagna burcu İkizler'in sütunu.
  fireEvent.click(screen.getByRole("button", { name: "pin house 1" }));
  expect(document.querySelector('th[data-sign="2"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('th[data-sign="4"]')).not.toHaveClass("is-highlighted");
  expect(marked()).toBe(8);
  // Esc vurguyu kaldırır.
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toBe(0);
});
