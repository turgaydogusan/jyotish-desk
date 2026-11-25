import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { useState } from "react";
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

test("Enter that commits IME text does not run the active item", async () => {
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  // WKWebView bileşimi bitiren Enter'ı `isComposing` olmadan, 229 koduyla da bildirir.
  for (const composing of [{ isComposing: true }, { keyCode: 229 }]) {
    fireEvent.keyDown(box(), { key: "ArrowDown", ...composing });
    fireEvent.keyDown(box(), { key: "Enter", ...composing });
  }
  expect(options()[0]).toHaveAttribute("aria-selected", "true");
  expect(onClose).not.toHaveBeenCalled();
  expect(onRun).not.toHaveBeenCalled();
});

test("a failed client search is shown instead of the empty text and clears on the next answer", async () => {
  vi.mocked(api.searchClients).mockResolvedValueOnce({ status: "error", error: { code: "db_failed", params: {} } });
  vi.mocked(api.searchClients).mockResolvedValueOnce(ok([]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "zzzz" } });
  expect(await screen.findByRole("alert")).toHaveTextContent(/^Liste okunamadı: /);
  expect(screen.queryByText("Eşleşen öğe yok.")).toBeNull();
  fireEvent.change(box(), { target: { value: "zzzy" } });
  expect(await screen.findByText("Eşleşen öğe yok.")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("clients found for the previous text are not offered while the next search is pending", async () => {
  vi.mocked(api.searchClients).mockResolvedValueOnce(ok([{ client: client(2, "Ayşe Yılmaz"), matched: "Name", snippet: null }]));
  let answer: (r: ReturnType<typeof ok<never[]>>) => void = () => {};
  vi.mocked(api.searchClients).mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
  renderPalette();
  fireEvent.change(box(), { target: { value: "ayşe" } });
  await waitFor(() => expect(options()[0]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.change(box(), { target: { value: "ayşez" } });
  expect(screen.queryByRole("option")).toBeNull();
  // Yanıt gelmeden "eşleşen yok" denmez ve Enter eski danışanı açmaz.
  expect(screen.queryByText("Eşleşen öğe yok.")).toBeNull();
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onRun).not.toHaveBeenCalled();
  await waitFor(() => expect(api.searchClients).toHaveBeenCalledTimes(2));
  await act(async () => answer(ok([])));
  expect(screen.getByText("Eşleşen öğe yok.")).toBeInTheDocument();
});

test("Esc closes the palette and focus returns to what was focused before it opened", async () => {
  function Host() {
    const [open, setOpen] = useState(false);
    return (
      <>
        <button type="button" onClick={() => setOpen(true)}>
          aç
        </button>
        {open && <Palette onClose={() => setOpen(false)} onRun={onRun} />}
      </>
    );
  }
  renderWithStores(<Host />, { opened: opened() });
  const opener = screen.getByRole("button", { name: "aç" });
  opener.focus();
  fireEvent.click(opener);
  await waitFor(() => expect(document.activeElement).toBe(box()));
  fireEvent.keyDown(box(), { key: "Escape" });
  expect(screen.queryByRole("combobox")).toBeNull();
  expect(document.activeElement).toBe(opener);
});

test("an older search reply that arrives after a newer one is dropped", async () => {
  let older: (r: Awaited<ReturnType<typeof api.searchClients>>) => void = () => {};
  vi.mocked(api.searchClients).mockReturnValueOnce(new Promise((resolve) => (older = resolve)));
  vi.mocked(api.searchClients).mockResolvedValueOnce(ok([{ client: client(2, "Ayşe Yılmaz"), matched: "Name", snippet: null }]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "ayş" } });
  await waitFor(() => expect(api.searchClients).toHaveBeenCalledTimes(1));
  fireEvent.change(box(), { target: { value: "ayşe" } });
  await waitFor(() => expect(options()[0]).toHaveTextContent("Ayşe Yılmaz"));
  await act(async () => older(ok([{ client: client(3, "Ayla Demir"), matched: "Name", snippet: null }])));
  expect(screen.queryByText("Ayla Demir")).toBeNull();
  expect(options()[0]).toHaveTextContent("Ayşe Yılmaz");
});
