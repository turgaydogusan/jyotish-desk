import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { listClients: vi.fn(), searchClients: vi.fn(), listCharts: vi.fn() } }));

import { api, type Result } from "../api";
import type { ChartRecord, Client, SearchHit } from "../bindings";
import "../i18n";
import { fixture } from "../test/fixture";
import { PartnerPicker, PICKER_CLIENTS, PICKER_SEARCH_DEBOUNCE_MS } from "./PartnerPicker";

const FX = fixture();
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure = { status: "error" as const, error: { code: "store_sqlite", params: { detail: "x" } } };
const client = (id: number, name: string): Client => ({ ...FX.opened.client, id, name, gender: null });
const chart = (clientId: number, id: number, label = ""): ChartRecord => ({ ...FX.opened.chart_record, id, client_id: clientId, label, uuid: `00000000-0000-4000-8000-0000000000${id}` });
const hit = (c: Client): SearchHit => ({ client: c, matched: "Name", snippet: null });

const AYSE = client(2, "Ayşe Yılmaz");
const MEHMET = client(3, "Mehmet Kaya");
/** Açık harita 10 numaradır ve Ayşe'nindir. */
const OPEN_CHART = 10;

function setup() {
  const onClose = vi.fn();
  const onPick = vi.fn();
  render(<PartnerPicker openChartId={OPEN_CHART} onClose={onClose} onPick={onPick} />);
  return { onClose, onPick };
}

const options = () => screen.queryAllByRole("option").map((o) => o.textContent);
const input = () => screen.getByRole("combobox", { name: "Danışan ara" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.listClients).mockResolvedValue(ok([AYSE, MEHMET]));
  vi.mocked(api.searchClients).mockResolvedValue(ok([hit(MEHMET)]));
  vi.mocked(api.listCharts).mockImplementation(async (id) => ok(id === 2 ? [chart(2, OPEN_CHART), chart(2, 11, "Evlilik")] : [chart(3, 30)]));
});

test("lists the recently opened clients for an empty search and searches by name", async () => {
  vi.useFakeTimers();
  try {
    setup();
    await act(async () => vi.advanceTimersByTime(0));
    expect(api.listClients).toHaveBeenCalledWith(PICKER_CLIENTS);
    expect(options()).toEqual(["Ayşe Yılmaz", "Mehmet Kaya"]);
    expect(input()).toHaveFocus();
    fireEvent.change(input(), { target: { value: " meh " } });
    // Yeni yanıt gelene dek eski danışanlar sunulmaz.
    expect(options()).toEqual([]);
    await act(async () => vi.advanceTimersByTime(PICKER_SEARCH_DEBOUNCE_MS - 1));
    expect(api.searchClients).not.toHaveBeenCalled();
    await act(async () => vi.advanceTimersByTime(1));
    expect(api.searchClients).toHaveBeenCalledWith("meh", PICKER_CLIENTS);
    expect(options()).toEqual(["Mehmet Kaya"]);
    expect(screen.getByRole("status")).toHaveTextContent("1 seçenek");
  } finally {
    vi.useRealTimers();
  }
});

test("a client with one other chart is picked at once; the dialog closes first", async () => {
  const { onClose, onPick } = setup();
  fireEvent.click(await screen.findByRole("option", { name: "Mehmet Kaya" }));
  await waitFor(() => expect(onPick).toHaveBeenCalledWith(chart(3, 30), MEHMET));
  expect(api.listCharts).toHaveBeenCalledWith(3);
  expect(onClose).toHaveBeenCalledTimes(1);
  expect(onClose.mock.invocationCallOrder[0]).toBeLessThan(onPick.mock.invocationCallOrder[0]);
});

test("the open chart is never offered: its client's other chart is picked", async () => {
  const { onPick } = setup();
  fireEvent.click(await screen.findByRole("option", { name: "Ayşe Yılmaz" }));
  await waitFor(() => expect(onPick).toHaveBeenCalledWith(chart(2, 11, "Evlilik"), AYSE));
});

test("a client with several charts shows a second list; a client with no other chart says so", async () => {
  vi.mocked(api.listCharts).mockImplementation(async (id) => ok(id === 2 ? [chart(2, OPEN_CHART)] : [chart(3, 30), chart(3, 31, "Düğün")]));
  const { onPick, onClose } = setup();
  fireEvent.click(await screen.findByRole("option", { name: "Mehmet Kaya" }));
  const list = await screen.findByRole("listbox", { name: "Mehmet Kaya: harita seçin" });
  expect(list).toHaveFocus();
  expect(options()).toEqual([`Doğum haritası${FX.opened.chart_record.birth.date}`, `Düğün${FX.opened.chart_record.birth.date}`]);
  expect(onPick).not.toHaveBeenCalled();
  // Klavye: ok tuşu ikinci haritaya geçer, Enter seçer.
  fireEvent.keyDown(list, { key: "ArrowDown" });
  expect(within(list).getAllByRole("option")[1]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(list, { key: "Enter" });
  expect(onPick).toHaveBeenCalledWith(chart(3, 31, "Düğün"), MEHMET);
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("a client whose only chart is the open one cannot be picked; back returns to the search", async () => {
  vi.mocked(api.listCharts).mockResolvedValue(ok([chart(2, OPEN_CHART)]));
  const { onPick } = setup();
  fireEvent.click(await screen.findByRole("option", { name: "Ayşe Yılmaz" }));
  expect(await screen.findByText("Bu danışanın eş olarak seçilebilecek başka haritası yok.")).toBeInTheDocument();
  expect(onPick).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Danışanlara dön" }));
  expect(await screen.findByRole("option", { name: "Mehmet Kaya" })).toBeInTheDocument();
  expect(input()).toHaveFocus();
});

test("keyboard: arrows move the active client, Enter chooses it, Escape closes", async () => {
  const { onPick, onClose } = setup();
  await screen.findByRole("option", { name: "Mehmet Kaya" });
  expect(screen.getAllByRole("option")[0]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(input(), { key: "ArrowUp" });
  expect(screen.getAllByRole("option")[1]).toHaveAttribute("aria-selected", "true");
  expect(input()).toHaveAttribute("aria-activedescendant", screen.getAllByRole("option")[1].id);
  // IME bileşimini bitiren Enter seçim yapmaz.
  fireEvent.keyDown(input(), { key: "Enter", keyCode: 229 });
  expect(api.listCharts).not.toHaveBeenCalled();
  fireEvent.keyDown(input(), { key: "Enter" });
  await waitFor(() => expect(onPick).toHaveBeenCalledWith(chart(3, 30), MEHMET));
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(2);
});

test("search and chart list failures are shown, not reported as no match", async () => {
  vi.mocked(api.listClients).mockResolvedValue(failure);
  setup();
  expect(await screen.findByRole("alert")).toHaveTextContent(/^Liste okunamadı: /);
  expect(screen.queryByText("Eşleşen danışan yok.")).toBeNull();
  vi.mocked(api.searchClients).mockResolvedValue(ok([hit(MEHMET)]));
  vi.mocked(api.listCharts).mockResolvedValue(failure);
  fireEvent.change(input(), { target: { value: "meh" } });
  fireEvent.click(await screen.findByRole("option", { name: "Mehmet Kaya" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(/^Liste okunamadı: /);
  expect(screen.queryByText("Bu danışanın eş olarak seçilebilecek başka haritası yok.")).toBeNull();
});

test("no matching client is said in words", async () => {
  vi.mocked(api.searchClients).mockResolvedValue(ok([]));
  setup();
  fireEvent.change(input(), { target: { value: "zzz" } });
  expect(await screen.findByText("Eşleşen danışan yok.")).toBeInTheDocument();
  expect(screen.getByRole("status")).toHaveTextContent("0 seçenek");
});
