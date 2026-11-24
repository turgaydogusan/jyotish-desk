import { fireEvent, render, screen, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./api", () => ({
  api: {
    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
      ],
    })),
    printWindow: vi.fn(async () => ({ status: "ok", data: null })),
  },
}));

import { api } from "./api";
import { App } from "./App";
import { memoryPersistence } from "./layout/persistence";
import { THEME_STORAGE_KEY } from "./theme/theme";

function renderApp() {
  return render(<App persistence={memoryPersistence()} />);
}

test("opens on the interview layout with the current-sky chart", async () => {
  renderApp();
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
  expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
  expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalled();
});

test("Cmd+2 switches to the research layout with the graha table", async () => {
  renderApp();
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
  const table = await screen.findByRole("region", { name: "Graha tablosu" });
  expect(within(table).getByRole("button", { name: "Satürn" }).closest("tr")).toHaveTextContent("℞");
});

test("Cmd+Shift+N toggles the notes panel in the active layout", () => {
  renderApp();
  expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
  expect(screen.queryByRole("region", { name: "Notlar" })).toBeNull();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
  const notes = screen.getByRole("region", { name: "Notlar" });
  expect(notes).toHaveAttribute("data-x", "8");
  expect(notes).toHaveAttribute("data-w", "4");
});

test("theme menu sets data-theme and remembers the choice", () => {
  renderApp();
  const select = screen.getByRole("combobox", { name: "Tema" });
  expect(select).toHaveDisplayValue("Sistem");
  fireEvent.change(select, { target: { value: "dark" } });
  expect(document.documentElement.dataset.theme).toBe("dark");
  expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  fireEvent.change(select, { target: { value: "light" } });
  expect(document.documentElement.dataset.theme).toBe("light");
});

test("language menu switches every string", async () => {
  renderApp();
  fireEvent.change(screen.getByRole("combobox", { name: "Dil" }), { target: { value: "en" } });
  expect(await screen.findByRole("combobox", { name: "Layout" })).toHaveDisplayValue("Consultation");
  expect(screen.getByRole("group", { name: "D1 chart" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Theme" })).toBeInTheDocument();
});

test("the print test stays reachable and calls printWindow", async () => {
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  expect(await screen.findByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
  expect(api.printWindow).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
  expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
});

test("a print failure is shown with a retry", async () => {
  vi.mocked(api.printWindow).mockResolvedValueOnce({ status: "error", error: { code: "print_failed", params: {} } });
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
});

test("a failed sky request shows an in-panel error instead of a blank desk", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderApp();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
});
