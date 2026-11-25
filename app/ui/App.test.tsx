import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./api", () => ({
  api: {
    printWindow: vi.fn(async () => ({ status: "ok", data: null })),
    previewMoment: vi.fn(async () => ({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } })),
    searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
    createClient: vi.fn(),
    updateChart: vi.fn(),
    saveViewState: vi.fn(async () => ({ status: "ok", data: null })),
    savePanelLayout: vi.fn(async () => ({ status: "ok", data: null })),
    listClients: vi.fn(async () => ({ status: "ok", data: [] })),
    listTags: vi.fn(async () => ({ status: "ok", data: [] })),
    listCharts: vi.fn(async () => ({ status: "ok", data: [] })),
    searchClients: vi.fn(async () => ({ status: "ok", data: [] })),
    openClient: vi.fn(),
    listNotes: vi.fn(async () => ({ status: "ok", data: [] })),
    setChartTags: vi.fn(async (_chartId: number, tags: string[]) => ({ status: "ok", data: tags })),
  },
}));

import { api } from "./api";
import { App } from "./App";
import type { Boot } from "./boot";
import { fixture, unknownTimeOpened } from "./test/fixture";
import { THEME_STORAGE_KEY } from "./theme/theme";
import { memoryPersistence } from "./view/persistence";

/** `store: true` gerçek store kalıcılığını (taklit `api`) kullanır; varsayılan bellek içi. */
function renderApp(boot: Partial<Boot> = {}, { store = false } = {}) {
  const full: Boot = { layout: null, layoutError: null, opened: fixture().opened, error: null, ...boot };
  return render(<App boot={full} persistence={store ? undefined : memoryPersistence()} />);
}

const toggleNotes = () => fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });

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

test("without a session the desk is one empty state whose action opens the birth form", () => {
  renderApp({ opened: null });
  expect(screen.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeInTheDocument();
  const empty = screen.getByRole("region", { name: "Açık harita yok" });
  expect(screen.queryByRole("group", { name: "D1 haritası" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Düzenle" })).toBeNull();
  fireEvent.click(within(empty).getByRole("button", { name: "Yeni harita" }));
  expect(screen.getByRole("dialog", { name: "Yeni harita" })).toBeInTheDocument();
});

test("Cmd+N creates a chart that replaces the empty state; Cmd+E edits it", async () => {
  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "ok", data: fixture().opened });
  renderApp({ opened: null });
  fireEvent.keyDown(window, { key: "n", metaKey: true });
  const form = screen.getByRole("dialog", { name: "Yeni harita" });
  fireEvent.change(within(form).getByLabelText("Danışan adı"), { target: { value: "Örnek Danışan" } });
  fireEvent.change(within(form).getByLabelText("Tarih"), { target: { value: "2000-01-01" } });
  fireEvent.change(within(form).getByLabelText("Saat"), { target: { value: "17:30" } });
  fireEvent.change(within(form).getByLabelText("Enlem"), { target: { value: "28.6139" } });
  fireEvent.change(within(form).getByLabelText("Boylam"), { target: { value: "77.209" } });
  fireEvent.change(within(form).getByLabelText("Bölge adı"), { target: { value: "Asia/Kolkata" } });
  fireEvent.click(within(form).getByRole("button", { name: "Haritayı kaydet" }));
  expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(screen.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "e", metaKey: true });
  expect(screen.getByRole("dialog", { name: "Haritayı düzenle" })).toBeInTheDocument();
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

test("a layout read failure is announced and view changes are not saved over the stored layout", async () => {
  vi.mocked(api.saveViewState).mockClear();
  renderApp({ layoutError: { code: "store_sqlite", params: { detail: "disk" } } }, { store: true });
  expect(within(screen.getByRole("region", { name: "Bildirimler" })).getByRole("alert")).toHaveTextContent(
    "Kayıtlı düzen okunamadı: Danışan veritabanı okunamadı ya da yazılamadı (disk).",
  );
  toggleNotes();
  await new Promise((r) => setTimeout(r, 600));
  expect(api.saveViewState).not.toHaveBeenCalled();
});

test("a save error notice goes away after the next successful save", async () => {
  vi.mocked(api.saveViewState).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  renderApp({}, { store: true });
  toggleNotes();
  expect(await screen.findByRole("alert", {}, { timeout: 3000 })).toHaveTextContent("Görünüm kaydedilemedi");
  toggleNotes();
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull(), { timeout: 3000 });
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

test("Cmd+Shift+N works from a text field and is ignored on key repeat, during composition and in print view", () => {
  renderApp();
  const notes = () => screen.queryByRole("region", { name: "Notlar" });
  fireEvent.click(screen.getByRole("button", { name: "Düzen işlemleri" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Farklı kaydet…" }));
  const input = screen.getByRole("textbox", { name: "Düzen adı" });
  input.focus();
  fireEvent.keyDown(input, { key: "N", metaKey: true, shiftKey: true });
  expect(notes()).toBeNull();
  fireEvent.keyDown(input, { key: "N", metaKey: true, shiftKey: true });
  expect(notes()).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, repeat: true });
  expect(notes()).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, isComposing: true });
  expect(notes()).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  const ev = new KeyboardEvent("keydown", { key: "N", metaKey: true, shiftKey: true, cancelable: true });
  window.dispatchEvent(ev);
  expect(ev.defaultPrevented).toBe(false);
  // Paletin aynı komutları da baskı görünümünde yok sayılır.
  for (const text of ["notlar panelini", "kenar çubuğunu"]) {
    fireEvent.keyDown(window, { key: "k", metaKey: true });
    const box = within(screen.getByRole("dialog", { name: "Komut paleti" })).getByRole("combobox");
    fireEvent.change(box, { target: { value: text } });
    fireEvent.keyDown(box, { key: "Enter" });
    expect(screen.queryByRole("dialog")).toBeNull();
  }
  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
  expect(notes()).toBeInTheDocument();
  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
});

test("Cmd+Shift+N still works after a choice in a select (M6-2)", () => {
  renderApp();
  const theme = screen.getByRole("combobox", { name: "Tema" });
  theme.focus();
  fireEvent.keyDown(theme, { key: "N", metaKey: true, shiftKey: true });
  expect(screen.queryByRole("region", { name: "Notlar" })).toBeNull();
});

test("Cmd+\\ and the Clients button show and hide the client sidebar", () => {
  renderApp();
  const toggle = screen.getByRole("button", { name: "Danışanlar" });
  expect(toggle).toHaveAttribute("aria-expanded", "true");
  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
  expect(toggle).toHaveAttribute("aria-expanded", "false");
  fireEvent.click(toggle);
  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
});

test("Cmd+\\ from the tags field saves the typed tags and moves focus to the Clients button", async () => {
  renderApp();
  const toggle = screen.getByRole("button", { name: "Danışanlar" });
  const tags = screen.getByLabelText("Etiketler");
  tags.focus();
  fireEvent.change(tags, { target: { value: "kariyer" } });
  fireEvent.keyDown(tags, { key: "\\", code: "Backslash", metaKey: true });
  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
  expect(api.setChartTags).toHaveBeenCalledWith(fixture().opened.chart_record.id, ["kariyer"]);
  expect(toggle).toHaveFocus();
  // Kayıt yanıtı kenar çubuğu kapandıktan sonra gelir; yeniden açılınca alan kayıtlı etiketi gösterir.
  fireEvent.click(toggle);
  await waitFor(() => expect(screen.getByLabelText("Etiketler")).toHaveValue("kariyer"));
});

test("Cmd+K, a name and Enter open that client's last chart with no form in between (spec §8)", async () => {
  const other = fixture().opened;
  other.client = { ...other.client, id: 2, name: "Ayşe Yılmaz" };
  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
  vi.mocked(api.searchClients).mockResolvedValueOnce({ status: "ok", data: [{ client: other.client, matched: "Name", snippet: null }] });
  vi.mocked(api.openClient).mockResolvedValueOnce({ status: "ok", data: other });
  renderApp();
  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
  fireEvent.keyDown(window, { key: "k", metaKey: true });
  const palette = screen.getByRole("dialog", { name: "Komut paleti" });
  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "ayşe" } });
  await within(palette).findByRole("option", { name: /Ayşe Yılmaz/ }, { timeout: 3000 });
  fireEvent.keyDown(within(palette).getByRole("combobox"), { key: "Enter", metaKey: true });
  expect(await screen.findByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeInTheDocument();
  expect(api.openClient).toHaveBeenCalledWith(2, null);
  expect(screen.queryByRole("dialog")).toBeNull();
  // Cmd+Enter kenar çubuğunu da gösterir (sekme yoktur).
  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
});

test("the palette's varga item sets the first chart panel and its layout item switches layouts", async () => {
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Ara" }));
  let palette = screen.getByRole("dialog", { name: "Komut paleti" });
  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "saptamsa" } });
  fireEvent.click(await within(palette).findByRole("option", { name: /D7 Saptamsa/ }));
  expect(screen.getByRole("group", { name: "D7 haritası" })).toBeInTheDocument();
  expect(screen.queryByRole("group", { name: "D1 haritası" })).toBeNull();
  fireEvent.keyDown(window, { key: "k", metaKey: true });
  palette = screen.getByRole("dialog", { name: "Komut paleti" });
  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "araştırma" } });
  fireEvent.click(await within(palette).findByRole("option", { name: /Araştırma/ }));
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
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
