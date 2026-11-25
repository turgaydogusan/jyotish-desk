import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listClients: vi.fn(),
    searchClients: vi.fn(),
    listCharts: vi.fn(),
  },
}));

import { api } from "../api";
import type { Client, OpenedClient } from "../bindings";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { foldText, matches } from "./items";
import { Palette, PALETTE_CLIENTS, PALETTE_RECENT, PALETTE_SEARCH_DEBOUNCE_MS } from "./Palette";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const opened = () => fixture().opened;
const client = (id: number, name: string): Client => ({ ...opened().client, id, name });
const onClose = vi.fn();
const onRun = vi.fn();

function renderPalette(o: OpenedClient | null = opened()) {
  return renderWithStores(<Palette onClose={onClose} onRun={onRun} />, { opened: o });
}

const box = () => screen.getByRole("combobox", { name: "Danışan, harita, varga, düzen ya da komut ara" });
const options = () => within(screen.getByRole("listbox", { name: "Sonuçlar" })).getAllByRole("option");

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  onClose.mockReset();
  onRun.mockReset();
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, client(2, "Ayşe Yılmaz")]));
  vi.mocked(api.listCharts).mockResolvedValue(ok([opened().chart_record]));
});

test("opens on the search box with recent clients first, then commands, charts, layouts and vargas", async () => {
  renderPalette();
  expect(document.activeElement).toBe(box());
  await waitFor(() => expect(options()[0]).toHaveTextContent("DanışanÖrnek Danışan"));
  expect(api.listClients).toHaveBeenCalledWith(PALETTE_RECENT);
  const kinds = options().map((o) => o.firstChild?.textContent);
  expect(kinds.slice(0, 3)).toEqual(["Danışan", "Danışan", "Komut"]);
  expect(kinds).toContain("Harita");
  expect(kinds).toContain("Düzen");
  expect(kinds).toContain("Varga");
  expect(options()[0]).toHaveAttribute("aria-selected", "true");
  expect(box()).toHaveAttribute("aria-activedescendant", options()[0].id);
});

test("typing searches clients after the debounce and filters the local items accent-blind", async () => {
  vi.useFakeTimers();
  vi.mocked(api.searchClients).mockResolvedValue(ok([{ client: client(2, "Ayşe Yılmaz"), matched: "Name", snippet: null }]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "navāmsa" } });
  expect(api.searchClients).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(PALETTE_SEARCH_DEBOUNCE_MS));
  vi.useRealTimers();
  expect(api.searchClients).toHaveBeenCalledWith("navāmsa", PALETTE_CLIENTS);
  await waitFor(() => expect(options().map((o) => o.textContent)).toEqual(["DanışanAyşe Yılmaz", "VargaD9 Navamsa"]));
  expect(matches("Navāmsa", "NAVAMSA", "tr")).toBe(true);
  expect(foldText("IŞIK", "tr")).toBe("isik");
});

test("arrow keys move the active option and wrap; Enter closes and runs it", async () => {
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(options().at(-1)).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  expect(options()[1]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onClose).toHaveBeenCalledOnce();
  expect(onRun).toHaveBeenCalledWith(expect.objectContaining({ kind: "client", clientId: 2 }), false);
});

test("the active option is scrolled into view as the arrows move it", async () => {
  const scroll = vi.spyOn(Element.prototype, "scrollIntoView");
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(scroll.mock.contexts.at(-1)).toBe(options().at(-1));
  expect(scroll).toHaveBeenLastCalledWith({ block: "nearest" });
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  expect(scroll.mock.contexts.at(-1)).toBe(options()[0]);
  scroll.mockRestore();
});

test("Cmd+Enter on a client is passed through as meta", async () => {
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "Enter", metaKey: true });
  expect(onRun).toHaveBeenCalledWith(expect.objectContaining({ kind: "client", clientId: 2 }), true);
});

test("without an open client there are no charts, vargas or client-only commands", async () => {
  renderPalette(null);
  await waitFor(() => expect(options()[0]).toHaveTextContent("Örnek Danışan"));
  const labels = options().map((o) => o.textContent ?? "");
  expect(labels.some((l) => l.startsWith("Harita") || l.startsWith("Varga"))).toBe(false);
  expect(labels).not.toContain("KomutAçık haritayı düzenle");
  expect(labels).toContain("KomutYeni danışan ve harita");
  expect(api.listCharts).not.toHaveBeenCalled();
});

test("nothing matching shows the empty text and Enter does nothing", async () => {
  vi.mocked(api.searchClients).mockResolvedValue(ok([]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "zzzz" } });
  expect(await screen.findByText("Eşleşen öğe yok.")).toBeInTheDocument();
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onRun).not.toHaveBeenCalled();
  expect(box()).not.toHaveAttribute("aria-activedescendant");
});
