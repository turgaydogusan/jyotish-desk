import { fireEvent, render, screen, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./api", () => ({
  api: {
    printWindow: vi.fn(async () => ({ status: "ok", data: null })),
  },
}));

import { api } from "./api";
import { App } from "./App";
import type { Boot } from "./boot";
import { fixture, unknownTimeOpened } from "./test/fixture";
import { THEME_STORAGE_KEY } from "./theme/theme";
import { memoryPersistence } from "./view/persistence";

function renderApp(boot: Partial<Boot> = {}) {
  const full: Boot = { layout: null, opened: fixture().opened, error: null, ...boot };
  return render(<App boot={full} persistence={memoryPersistence()} />);
}

test("opens on the interview layout with the restored chart", () => {
  renderApp();
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
  expect(screen.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeInTheDocument();
  expect(screen.getByText("Doğum haritası")).toBeInTheDocument();
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
  expect(screen.queryByRole("list", { name: "Uyarılar" })).toBeNull();
});

test("without a session the desk shows the app title and empty panels", () => {
  renderApp({ opened: null });
  expect(screen.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeInTheDocument();
  expect(screen.getAllByText("Açık harita yok.").length).toBeGreaterThan(0);
});

test("moment warnings are badges with a text label and a hint", () => {
  renderApp({ opened: unknownTimeOpened() });
  const badges = screen.getByRole("list", { name: "Uyarılar" });
  const badge = within(badges).getByText("Saat bilinmiyor").closest("li")!;
  expect(badge).toHaveAttribute("title", "Doğum saati bilinmiyor; öğlen 12:00 alındı.");
  expect(badge).toHaveAttribute("tabindex", "0");
  expect(badge).toHaveTextContent(": Doğum saati bilinmiyor; öğlen 12:00 alındı.");
});

test("a restore failure and store warnings are shown in the notice bar and can be dismissed", () => {
  const { unmount } = renderApp({ opened: null, error: { code: "store_sqlite", params: { detail: "disk" } } });
  const region = screen.getByRole("region", { name: "Bildirimler" });
  expect(within(region).getByRole("alert")).toHaveTextContent(
    "Son oturum açılamadı: Danışan veritabanı okunamadı ya da yazılamadı (disk).",
  );
  fireEvent.click(within(region).getByRole("button", { name: "Kapat" }));
  expect(screen.queryByRole("region", { name: "Bildirimler" })).toBeNull();
  unmount();
  const opened = fixture().opened;
  opened.warnings = ["LastChartMissing", { TzdbChanged: { stored_version: "2025a", current_version: "2026a", current_utc: null } }];
  renderApp({ opened });
  const statuses = within(screen.getByRole("region", { name: "Bildirimler" })).getAllByRole("status");
  expect(statuses.map((s) => s.textContent)).toEqual([
    "UyarıSon açılan harita silinmiş; en son güncellenen harita açıldı.Kapat",
    "UyarıSaat dilimi verisi değişti (2025a → 2026a); saklanan an kullanılıyor.Kapat",
  ]);
});

test("Cmd+2 switches to the research layout with the graha table", () => {
  renderApp();
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
  const table = screen.getByRole("region", { name: "Graha tablosu" });
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

test("Cmd+Shift+N is ignored from editable fields, on key repeat, during composition and in print view", () => {
  renderApp();
  const notes = () => screen.queryByRole("region", { name: "Notlar" });
  fireEvent.click(screen.getByRole("button", { name: "Düzen işlemleri" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Farklı kaydet…" }));
  const input = screen.getByRole("textbox", { name: "Düzen adı" });
  input.focus();
  fireEvent.keyDown(input, { key: "N", metaKey: true, shiftKey: true });
  expect(notes()).toBeInTheDocument();
  fireEvent.keyDown(screen.getByRole("combobox", { name: "Tema" }), { key: "N", metaKey: true, shiftKey: true });
  expect(notes()).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, repeat: true });
  expect(notes()).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, isComposing: true });
  expect(notes()).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  const ev = new KeyboardEvent("keydown", { key: "N", metaKey: true, shiftKey: true, cancelable: true });
  window.dispatchEvent(ev);
  expect(ev.defaultPrevented).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
  expect(notes()).toBeInTheDocument();
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
  expect(screen.getByText("Natal chart")).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Theme" })).toBeInTheDocument();
});

test("the print test stays reachable and calls printWindow", async () => {
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  expect(await screen.findByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
  expect(screen.getByTestId("sign-8")).toHaveTextContent("Sun");
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

test("? opens the shortcut help as a modal that traps focus, blocks shortcuts and closes with Esc", () => {
  renderApp();
  const opener = screen.getByRole("button", { name: "Kısayollar" });
  opener.focus();
  fireEvent.keyDown(opener, { key: "?", shiftKey: true });
  const dialog = screen.getByRole("dialog", { name: "Klavye kısayolları" });
  expect(dialog).toHaveAttribute("aria-modal", "true");
  expect(within(dialog).getByText("⌘N")).toBeInTheDocument();
  expect(within(dialog).getByRole("row", { name: /⌘⇧N Notlar panelini aç\/kapat/ })).toBeInTheDocument();
  const close = within(dialog).getByRole("button", { name: "Kapat" });
  expect(document.activeElement).toBe(close);
  // Tek odaklanabilir öğe: Tab ve Shift+Tab tutulur (jsdom odağı kendisi taşımaz; ayrıntı Dialog.test.tsx'te).
  expect(fireEvent.keyDown(close, { key: "Tab" })).toBe(false);
  expect(fireEvent.keyDown(close, { key: "Tab", shiftKey: true })).toBe(false);
  expect(document.activeElement).toBe(close);
  // Modal açıkken arkadaki düzen değişmez.
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
  fireEvent.keyDown(close, { key: "Escape" });
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(document.activeElement).toBe(opener);
});
