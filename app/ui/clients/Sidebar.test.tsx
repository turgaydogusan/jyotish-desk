import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listClients: vi.fn(),
    listClientsByTag: vi.fn(),
    searchClients: vi.fn(),
    listTags: vi.fn(),
    listCharts: vi.fn(),
    openClient: vi.fn(),
    deleteChart: vi.fn(),
    deleteClient: vi.fn(),
    setChartTags: vi.fn(),
  },
}));

import { api } from "../api";
import type { ChartRecord, Client, Graha, OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useViewStore } from "../view/store";
import { CLIENT_SEARCH_DEBOUNCE_MS, sortTags } from "./ClientList";
import { Sidebar } from "./Sidebar";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const opened = () => fixture().opened;
const client = (id: number, name: string): Client => ({ ...opened().client, id, name, last_chart_id: id * 10 });
const chart = (id: number, label: string, date = "2000-01-01"): ChartRecord => ({
  ...opened().chart_record,
  id,
  label,
  birth: { ...opened().chart_record.birth, date },
});
const ayse = client(2, "Ayşe Yılmaz");
const onAddChart = vi.fn();

function OpenedProbe() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened ? `${opened.client.name}#${opened.chart_record.id}` : "-"}</output>;
}

function renderSidebar(o: OpenedClient | null = opened()) {
  return renderWithStores(
    <>
      <OpenedProbe />
      <Sidebar onAddChart={onAddChart} />
    </>,
    { opened: o },
  );
}

const sidebar = () => screen.getByRole("complementary", { name: "Danışanlar" });
const list = () => within(sidebar()).getByRole("region", { name: /Son açılanlar|Arama sonuçları|etiketli/ });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  onAddChart.mockReset();
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, ayse]));
  vi.mocked(api.listTags).mockResolvedValue(ok([]));
  vi.mocked(api.listCharts).mockResolvedValue(ok([opened().chart_record]));
});

test("recent clients come from list_clients, the open one is marked and choosing another opens it", async () => {
  const other = opened();
  other.client = ayse;
  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
  vi.mocked(api.openClient).mockResolvedValueOnce(ok(other));
  renderSidebar();
  const mine = await within(list()).findByRole("button", { name: "Örnek Danışan" });
  expect(mine).toHaveAttribute("aria-current", "true");
  expect(api.listClients).toHaveBeenCalledWith(200);
  fireEvent.click(within(list()).getByRole("button", { name: "Ayşe Yılmaz" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20"));
  expect(api.openClient).toHaveBeenCalledWith(2, null);
  // Açılış listeyi yeniden okur; açık danışan işareti yeni danışana geçer.
  await waitFor(() => expect(within(list()).getByRole("button", { name: "Ayşe Yılmaz" })).toHaveAttribute("aria-current", "true"));
});

test("typing searches after the debounce and shows why a client matched; clearing shows recent again", async () => {
  vi.useFakeTimers();
  vi.mocked(api.searchClients).mockResolvedValue(ok([{ client: ayse, matched: "Note", snippet: "…satürn dönüşü…" }]));
  renderSidebar(null);
  const box = within(sidebar()).getByRole("searchbox", { name: "Danışan ara" });
  fireEvent.change(box, { target: { value: "satürn" } });
  expect(api.searchClients).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(CLIENT_SEARCH_DEBOUNCE_MS));
  vi.useRealTimers();
  expect(api.searchClients).toHaveBeenCalledWith("satürn", 200);
  expect(within(list()).getByRole("heading")).toHaveTextContent("Arama sonuçları");
  expect(within(list()).getByRole("button", { name: /Ayşe Yılmaz/ })).toHaveTextContent("Notta: …satürn dönüşü…");
  fireEvent.change(box, { target: { value: "" } });
  expect(await within(list()).findByRole("button", { name: "Örnek Danışan" })).toBeInTheDocument();
});

test("the tag filter lists clients by tag; tags are in the language's alphabetical order", async () => {
  vi.mocked(api.listTags).mockResolvedValue(ok([{ name: "zeki", charts: 1 }, { name: "çocuk", charts: 2 }, { name: "araştırma", charts: 3 }]));
  vi.mocked(api.listClientsByTag).mockResolvedValue(ok([ayse]));
  renderSidebar(null);
  const select = within(sidebar()).getByRole("combobox", { name: "Etiket" });
  await waitFor(() => expect(within(select).getAllByRole("option")).toHaveLength(4));
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["Tüm etiketler", "araştırma (3)", "çocuk (2)", "zeki (1)"]);
  fireEvent.change(select, { target: { value: "çocuk" } });
  await waitFor(() => expect(api.listClientsByTag).toHaveBeenCalledWith("çocuk", 200));
  expect(await within(list()).findByRole("button", { name: "Ayşe Yılmaz" })).toBeInTheDocument();
  expect(within(list()).getByRole("heading")).toHaveTextContent("“çocuk” etiketli");
  expect(sortTags([{ name: "Öfke", charts: 1 }, { name: "Zeki", charts: 1 }, { name: "Ofis", charts: 1 }], "tr").map((t) => t.name)).toEqual(["Ofis", "Öfke", "Zeki"]);
});

test("a list error is shown with a retry", async () => {
  vi.mocked(api.listClients).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderSidebar(null);
  const alert = await within(sidebar()).findByRole("alert");
  expect(alert).toHaveTextContent("Liste okunamadı: Danışan veritabanı kullanılamıyor.");
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  expect(await within(list()).findByRole("button", { name: "Örnek Danışan" })).toBeInTheDocument();
});

test("a client's only chart cannot be deleted and the reason is attached to the button", async () => {
  renderSidebar();
  const del = await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" });
  expect(del).toHaveAttribute("aria-disabled", "true");
  expect(del).toHaveAccessibleDescription("Danışanın tek haritası silinemez; gerekirse danışanı silin.");
  fireEvent.click(del);
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Harita ekle" }));
  expect(onAddChart).toHaveBeenCalledOnce();
});

test("deleting the open chart after confirmation opens the most recently updated other chart", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValueOnce(ok(charts)).mockResolvedValue(ok([charts[1]]));
  vi.mocked(api.deleteChart).mockResolvedValue(ok(null));
  const next = opened();
  next.chart_record = charts[1];
  vi.mocked(api.openClient).mockResolvedValue(ok(next));
  renderSidebar();
  const del = await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" });
  fireEvent.click(del);
  const dialog = screen.getByRole("dialog", { name: "Harita silinsin mi?" });
  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Vazgeç" }));
  expect(dialog).toHaveTextContent("“Doğum haritası” haritası silinecek. Bu işlem geri alınamaz; danışanın notları silinmez.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Haritayı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan#5"));
  expect(api.deleteChart).toHaveBeenCalledWith(1);
  expect(api.openClient).toHaveBeenCalledWith(1, 5);
});

test("deleting the open client after confirmation empties the desk; cancelling deletes nothing", async () => {
  vi.mocked(api.deleteClient).mockResolvedValue(ok(null));
  renderSidebar();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Danışan silinsin mi?" })).getByRole("button", { name: "Vazgeç" }));
  expect(api.deleteClient).not.toHaveBeenCalled();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  const dialog = screen.getByRole("dialog", { name: "Danışan silinsin mi?" });
  expect(dialog).toHaveTextContent("“Örnek Danışan” bütün haritaları ve notlarıyla silinecek. Bu işlem geri alınamaz.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Danışanı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("-"));
  expect(api.deleteClient).toHaveBeenCalledWith(1);
});

test("tags are saved on Enter as the normalised list Rust returns", async () => {
  vi.mocked(api.setChartTags).mockResolvedValue(ok(["araştırma", "Kariyer"]));
  renderSidebar();
  const box = within(sidebar()).getByRole("textbox", { name: "Etiketler" });
  expect(box).toHaveAccessibleDescription("Virgülle ayırın; Enter kaydeder.");
  fireEvent.change(box, { target: { value: " Kariyer, araştırma ,, kariyer" } });
  fireEvent.keyDown(box, { key: "Enter" });
  await waitFor(() => expect(box).toHaveValue("araştırma, Kariyer"));
  expect(api.setChartTags).toHaveBeenCalledWith(1, ["Kariyer", "araştırma", "kariyer"]);
  // Değişmeyen metin yeniden kaydedilmez.
  fireEvent.blur(box);
  expect(api.setChartTags).toHaveBeenCalledOnce();
});

// C5: bekleyen görünüm kaydı silme komutundan önce yazılır; silinen haritaya sonradan kayıt gitmez.
function PathProbe() {
  const { setDashaPath } = useViewStore();
  return (
    <button type="button" onClick={() => setDashaPath(["Rahu"] as Graha[])}>
      yol
    </button>
  );
}

/** Debounce süresi dolmadan silme başlar: kayıt beklemededir. */
function renderWithPendingSave() {
  const { persistence } = renderWithStores(
    <>
      <PathProbe />
      <OpenedProbe />
      <Sidebar onAddChart={onAddChart} />
    </>,
    { debounceMs: 60_000 },
  );
  fireEvent.click(screen.getByRole("button", { name: "yol" }));
  expect(persistence.saved).toEqual([]);
  return persistence;
}

test("a pending view save is written before the open chart is deleted and nothing is saved to it afterwards", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValueOnce(ok(charts)).mockResolvedValue(ok([charts[1]]));
  const next = opened();
  next.chart_record = charts[1];
  vi.mocked(api.openClient).mockResolvedValue(ok(next));
  const persistence = renderWithPendingSave();
  const savedAtDelete: (number | null)[][] = [];
  vi.mocked(api.deleteChart).mockImplementation(async () => {
    savedAtDelete.push(persistence.saved.map((s) => s.chartId));
    return ok(null);
  });
  fireEvent.click(await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Harita silinsin mi?" })).getByRole("button", { name: "Haritayı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan#5"));
  expect(savedAtDelete).toEqual([[1]]);
  expect(persistence.saved.map((s) => s.chartId)).toEqual([1]);
});

test("a pending view save is written before the open client is deleted and nothing is saved to it afterwards", async () => {
  const persistence = renderWithPendingSave();
  const savedAtDelete: (number | null)[][] = [];
  vi.mocked(api.deleteClient).mockImplementation(async () => {
    savedAtDelete.push(persistence.saved.map((s) => s.chartId));
    return ok(null);
  });
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Danışan silinsin mi?" })).getByRole("button", { name: "Danışanı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("-"));
  expect(savedAtDelete).toEqual([[1]]);
  expect(persistence.saved.map((s) => s.chartId)).toEqual([1]);
});

test("the client list is one tab stop; arrows, Home and End move between clients", async () => {
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, ayse, client(3, "Can Demir")]));
  renderSidebar();
  const buttons = await within(list()).findAllByRole("button");
  expect(buttons.map((b) => b.tabIndex)).toEqual([0, -1, -1]);
  buttons[0].focus();
  fireEvent.keyDown(buttons[0], { key: "ArrowDown" });
  expect(document.activeElement).toBe(buttons[1]);
  fireEvent.keyDown(buttons[1], { key: "End" });
  expect(document.activeElement).toBe(buttons[2]);
  expect(buttons.map((b) => b.tabIndex)).toEqual([-1, -1, 0]);
  fireEvent.keyDown(buttons[2], { key: "ArrowDown" });
  expect(document.activeElement).toBe(buttons[2]);
  fireEvent.keyDown(buttons[2], { key: "Home" });
  expect(document.activeElement).toBe(buttons[0]);
});

test("a list refresh keeps the tab stop and focus on the row being arrowed through", async () => {
  const three = [opened().client, ayse, client(3, "Can Demir")];
  vi.mocked(api.listClients).mockImplementation(async () => ok(three.map((c) => ({ ...c }))));
  function Touch() {
    const { touch } = useSession();
    return (
      <button type="button" onClick={touch}>
        yenile
      </button>
    );
  }
  renderWithStores(
    <>
      <Touch />
      <Sidebar onAddChart={onAddChart} />
    </>,
  );
  const rows = () => within(list()).getAllByRole("button");
  const first = (await within(list()).findAllByRole("button"))[0];
  first.focus();
  fireEvent.keyDown(first, { key: "End" });
  expect(rows().map((b) => b.tabIndex)).toEqual([-1, -1, 0]);
  const calls = vi.mocked(api.listClients).mock.calls.length;
  fireEvent.click(screen.getByRole("button", { name: "yenile" }));
  await waitFor(() => expect(vi.mocked(api.listClients).mock.calls.length).toBeGreaterThan(calls));
  await act(async () => {});
  expect(rows().map((b) => b.tabIndex)).toEqual([-1, -1, 0]);
  expect(document.activeElement).toBe(rows()[2]);
  // Odaktaki danışan listeden çıkarsa durak açık danışana döner.
  three.pop();
  fireEvent.click(screen.getByRole("button", { name: "yenile" }));
  await waitFor(() => expect(rows()).toHaveLength(2));
  expect(rows().map((b) => b.tabIndex)).toEqual([0, -1]);
});

test("arrow keys with a modifier are left to the system", async () => {
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, ayse, client(3, "Can Demir")]));
  renderSidebar();
  const buttons = await within(list()).findAllByRole("button");
  buttons[0].focus();
  for (const mod of [{ metaKey: true }, { ctrlKey: true }, { altKey: true }]) {
    expect(fireEvent.keyDown(buttons[0], { key: "ArrowDown", ...mod })).toBe(true);
  }
  expect(fireEvent.keyDown(buttons[0], { key: "End", shiftKey: true })).toBe(true);
  expect(document.activeElement).toBe(buttons[0]);
  expect(buttons.map((b) => b.tabIndex)).toEqual([0, -1, -1]);
});

test("after deleting a chart focus goes to the client heading, after deleting the client to the search box", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValueOnce(ok(charts)).mockResolvedValue(ok([charts[0]]));
  vi.mocked(api.deleteChart).mockResolvedValue(ok(null));
  vi.mocked(api.deleteClient).mockResolvedValue(ok(null));
  renderSidebar();
  fireEvent.click(await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Düzeltilmiş”" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Harita silinsin mi?" })).getByRole("button", { name: "Haritayı sil" }));
  await waitFor(() => expect(document.activeElement).toBe(within(sidebar()).getByRole("heading", { name: "Örnek Danışan" })));
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Danışan silinsin mi?" })).getByRole("button", { name: "Danışanı sil" }));
  await waitFor(() => expect(document.activeElement).toBe(within(sidebar()).getByRole("searchbox", { name: "Danışan ara" })));
});

test("a tag filter whose tag disappeared falls back to all tags; a tag list error is shown", async () => {
  vi.mocked(api.listTags).mockResolvedValue(ok([{ name: "Kariyer", charts: 1 }]));
  vi.mocked(api.listClientsByTag).mockResolvedValue(ok([ayse]));
  let touch: () => void = () => {};
  function Toucher() {
    touch = useSession().touch;
    return null;
  }
  renderWithStores(
    <>
      <Toucher />
      <Sidebar onAddChart={onAddChart} />
    </>,
    { opened: opened() },
  );
  const filter = await within(sidebar()).findByRole("combobox", { name: "Etiket" });
  await within(filter).findByRole("option", { name: "Kariyer (1)" });
  fireEvent.change(filter, { target: { value: "Kariyer" } });
  await waitFor(() => expect(api.listClientsByTag).toHaveBeenCalledWith("Kariyer", 200));
  vi.mocked(api.listTags).mockResolvedValue(ok([]));
  act(() => touch());
  await waitFor(() => expect(filter).toHaveValue(""));
  expect(within(sidebar()).getByRole("heading", { name: "Son açılanlar" })).toBeInTheDocument();
  vi.mocked(api.listTags).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  act(() => touch());
  expect(await within(sidebar()).findByText(/Etiketler okunamadı: .*disk/)).toBeInTheDocument();
});

test("switching to another client while the open chart is being deleted does not reopen the first client", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValue(ok(charts));
  let release!: () => void;
  vi.mocked(api.deleteChart).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  const other = opened();
  other.client = ayse;
  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
  vi.mocked(api.openClient).mockResolvedValue(ok(other));
  renderSidebar();
  fireEvent.click(await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Harita silinsin mi?" })).getByRole("button", { name: "Haritayı sil" }));
  fireEvent.click(await within(list()).findByRole("button", { name: /Ayşe Yılmaz/ }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20"));
  release();
  await act(async () => {});
  expect(api.openClient).toHaveBeenCalledOnce();
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20");
});

test("Enter that commits IME text in the tag box does not save", async () => {
  vi.mocked(api.setChartTags).mockResolvedValue(ok(["a"]));
  renderSidebar();
  const box = await within(sidebar()).findByRole("textbox", { name: "Etiketler" });
  fireEvent.change(box, { target: { value: "a" } });
  fireEvent.keyDown(box, { key: "Enter", keyCode: 229 });
  expect(api.setChartTags).not.toHaveBeenCalled();
  fireEvent.keyDown(box, { key: "Enter" });
  await waitFor(() => expect(api.setChartTags).toHaveBeenCalledOnce());
});
