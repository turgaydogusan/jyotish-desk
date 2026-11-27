import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { useState } from "react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: { chartSearch: vi.fn(), chartSearchCancel: vi.fn(), tagCharts: vi.fn(), listSavedSearches: vi.fn(), saveSearch: vi.fn(), deleteSavedSearch: vi.fn() },
}));

import { api, type Result } from "../api";
import type { ChartQuery, ChartSearchPage } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { fixture } from "../test/fixture";
import { ChartSearch } from "./ChartSearch";
import { EMPTY_QUERY } from "./model";

const FX = fixture().search;
const onClose = vi.fn();
const onOpenChart = vi.fn();
const onTagged = vi.fn();
let current: ChartQuery;

function Harness({ initial }: { initial: ChartQuery }) {
  const [query, setQuery] = useState(initial);
  current = query;
  return <ChartSearch query={query} onQueryChange={setQuery} onClose={onClose} onOpenChart={onOpenChart} onTagged={onTagged} />;
}
const open = (initial: ChartQuery = FX.query) => render(<Harness initial={initial} />);
const dialog = () => screen.getByRole("dialog", { name: "Haritalarda ara" });
const button = (name: string | RegExp) => within(dialog()).getByRole("button", { name });
const summary = () => dialog().querySelector("[data-summary]")?.textContent;
const settle = () => act(async () => {});
const ok = (page: ChartSearchPage): Result<ChartSearchPage> => ({ status: "ok", data: page });
function deferred() {
  let resolve!: (r: Result<ChartSearchPage>) => void;
  const promise = new Promise<Result<ChartSearchPage>>((r) => (resolve = r));
  return { promise, resolve };
}
const first = (total: number): ChartSearchPage => ({
  ...FX.page,
  scanned: 500,
  total,
  done: false,
  next: { after_chart_id: 500, scanned: 500, skipped_time_unknown: 0, failed: 0, total, now_jd_ut: 2461372 },
});

beforeEach(() => {
  for (const f of [onClose, onOpenChart, onTagged, ...Object.values(api)]) vi.mocked(f).mockReset();
  vi.mocked(api.listSavedSearches).mockResolvedValue({ status: "ok", data: [FX.saved] });
  vi.mocked(api.chartSearchCancel).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.chartSearch).mockResolvedValue(ok(FX.page));
});

test("without conditions the only action adds an example and the search says why it cannot run", async () => {
  open(EMPTY_QUERY);
  await settle();
  expect(within(dialog()).getByText(/Henüz koşul yok/)).toBeInTheDocument();
  const run = button("Ara");
  expect(run).toHaveAttribute("aria-disabled", "true");
  expect(run).toHaveAccessibleDescription("Aramak için en az bir koşul ekleyin.");
  fireEvent.click(run);
  expect(api.chartSearch).not.toHaveBeenCalled();
  // Odak pencere açılınca tek eylemdedir.
  expect(button("Örnek koşul ekle")).toHaveFocus();
  fireEvent.click(button("Örnek koşul ekle"));
  const row = within(dialog()).getByRole("group", { name: "Koşul 1" });
  expect(within(row).getByLabelText("Tür")).toHaveDisplayValue("Graha burçta");
  expect(within(row).getByLabelText("Graha")).toHaveDisplayValue("Ay");
  expect(within(row).getByRole("checkbox", { name: "Yen" })).toBeChecked();
  expect(button("Ara")).not.toHaveAttribute("aria-disabled");
  // Koşul varken düğme sıradan bir koşul ekler ve eksiğini satırında söyler.
  fireEvent.click(button("Koşul ekle"));
  expect(within(within(dialog()).getByRole("group", { name: "Koşul 2" })).getByText("En az bir burç seçin.")).toBeInTheDocument();
  expect(button("Ara")).toHaveAccessibleDescription("Koşul 2 eksik: seçeneklerini tamamlayın.");
});

test("a search lists the matching charts and a result opens its chart", async () => {
  open();
  await settle();
  expect(within(dialog()).getByText(/Koşulları kurup/)).toBeInTheDocument();
  fireEvent.click(button("Ara"));
  await settle();
  expect(vi.mocked(api.chartSearch).mock.calls[0][0]).toEqual(FX.query);
  expect(vi.mocked(api.chartSearch).mock.calls[0][2]).toBeNull();
  expect(summary()).toBe("1 eşleşme / 2 harita tarandı");
  const hit = button("Örnek Danışan, etiketsiz harita: haritayı aç");
  expect(hit).toHaveTextContent("Ay Ter 5. ev; değil: Gü 1/10. ev");
  fireEvent.click(hit);
  expect(onOpenChart).toHaveBeenCalledWith(FX.page.hits[0].client_id, FX.page.hits[0].chart_id);
});

test("while scanning the progress is honest, and stopping keeps the matches and the focus", async () => {
  const [one, two] = [deferred(), deferred()];
  vi.mocked(api.chartSearch).mockReset().mockReturnValueOnce(one.promise).mockReturnValueOnce(two.promise);
  open();
  await settle();
  fireEvent.click(button("Ara"));
  // İlk sayfadan önce sayı uydurulmaz.
  expect(summary()).toBe("Aranıyor…");
  expect(within(dialog()).queryByRole("progressbar")).toBeNull();
  expect(button("Ara")).toHaveAttribute("aria-disabled", "true");
  await act(async () => one.resolve(ok(first(1200))));
  expect(summary()).toBe("1 eşleşme · 500 / 1200 harita tarandı");
  expect(within(dialog()).getByRole("progressbar", { name: "Taranan haritalar" })).toHaveAttribute("value", "500");
  expect(button(/haritayı aç/)).toBeInTheDocument();
  const stop = button("Durdur");
  stop.focus();
  fireEvent.click(stop);
  expect(summary()).toBe("Durduruldu: 1 eşleşme · 500 / 1200 harita tarandı");
  expect(api.chartSearchCancel).toHaveBeenCalledWith(vi.mocked(api.chartSearch).mock.calls[0][1]);
  // "Durdur" gider; odak "Ara"ya geçer, gövdeye düşmez.
  expect(within(dialog()).queryByRole("button", { name: "Durdur" })).toBeNull();
  expect(button("Ara")).toHaveFocus();
  // Durdurulan aramanın geç gelen sayfası listeyi değiştirmez.
  await act(async () => two.resolve(ok(FX.page)));
  expect(summary()).toBe("Durduruldu: 1 eşleşme · 500 / 1200 harita tarandı");
});

test("no matches, an empty scope and the skipped charts are said in words", async () => {
  vi.mocked(api.chartSearch).mockResolvedValueOnce(ok({ ...FX.page, hits: [], scanned: 40, total: 40, skipped_time_unknown: 3, failed: 2 }));
  open();
  await settle();
  fireEvent.click(button("Ara"));
  await settle();
  expect(summary()).toBe("0 eşleşme / 40 harita tarandı");
  expect(within(dialog()).getByText(/Eşleşme yok \(40 harita tarandı\)\. Koşulları gevşetin/)).toBeInTheDocument();
  expect(within(dialog()).getByText("Saati bilinmediği için değerlendirilemeyen: 3")).toBeInTheDocument();
  expect(within(dialog()).getByText("Hesaplanamayan: 2")).toBeInTheDocument();
  expect(within(dialog()).queryByLabelText("Sonuçları etiketle")).toBeNull();
  vi.mocked(api.chartSearch).mockResolvedValueOnce(ok({ ...FX.page, hits: [], scanned: 0, total: 0 }));
  fireEvent.click(button("Ara"));
  await settle();
  expect(within(dialog()).getByText("Kapsamda harita yok.")).toBeInTheDocument();
});

test("changing the form after a search marks the list as belonging to the earlier search", async () => {
  open();
  await settle();
  fireEvent.click(button("Ara"));
  await settle();
  expect(within(dialog()).queryByRole("note")).toBeNull();
  fireEvent.click(within(dialog()).getByRole("checkbox", { name: "Prashna haritalarını dışla" }));
  expect(current.scope.exclude_prashna).toBe(false);
  expect(within(dialog()).getByRole("note")).toHaveTextContent("Koşullar değişti; liste önceki aramaya ait.");
  // Eski listenin sonucu yerinde durur ama etiketlenemez.
  expect(button(/haritayı aç/)).toBeInTheDocument();
  expect(button("Etiketle")).toHaveAttribute("aria-disabled", "true");
  // Arama sürerken form değişirse arama durdurulur.
  const pending = deferred();
  vi.mocked(api.chartSearch).mockReturnValueOnce(pending.promise);
  fireEvent.click(button("Ara"));
  fireEvent.click(within(dialog()).getByRole("checkbox", { name: "Prashna haritalarını dışla" }));
  await settle();
  expect(api.chartSearchCancel).toHaveBeenCalledTimes(1);
  expect(summary()).toBe("Arama ilk sonuçtan önce durduruldu.");
});

test("an error is shown with a retry that keeps the focus in the surface", async () => {
  vi.mocked(api.chartSearch).mockResolvedValueOnce({ status: "error", error: { code: "invalid_search_input", params: { field: "signs" } } });
  open();
  await settle();
  fireEvent.click(button("Ara"));
  await settle();
  expect(within(dialog()).getByRole("alert")).toHaveTextContent("Arama isteği geçersiz: signs.");
  const retry = button("Yeniden dene");
  retry.focus();
  fireEvent.click(retry);
  expect(button("Ara")).toHaveFocus();
  await settle();
  expect(api.chartSearch).toHaveBeenCalledTimes(2);
  expect(within(dialog()).queryByRole("alert")).toBeNull();
  expect(summary()).toBe("1 eşleşme / 2 harita tarandı");
});

test("tagging the results asks first, naming the tag and the number of charts", async () => {
  vi.mocked(api.tagCharts).mockResolvedValue({ status: "ok", data: 1 });
  open();
  await settle();
  fireEvent.click(button("Ara"));
  await settle();
  const apply = button("Etiketle");
  expect(apply).toHaveAccessibleDescription("Bir etiket yazın.");
  fireEvent.click(apply);
  expect(screen.queryByRole("dialog", { name: "Sonuçları etiketle" })).toBeNull();
  fireEvent.change(within(dialog()).getByLabelText("Sonuçları etiketle"), { target: { value: "bir, iki" } });
  expect(button("Etiketle")).toHaveAccessibleDescription("Tek bir etiket yazın (virgülsüz).");
  fireEvent.change(within(dialog()).getByLabelText("Sonuçları etiketle"), { target: { value: " Ay Terazi " } });
  fireEvent.click(button("Etiketle"));
  const confirm = screen.getByRole("dialog", { name: "Sonuçları etiketle" });
  expect(confirm).toHaveTextContent("“Ay Terazi” etiketi 1 haritaya eklenecek. Haritaların öteki etiketleri durur.");
  // Odak "Vazgeç"te başlar; vazgeçmek yazmaz.
  expect(within(confirm).getByRole("button", { name: "Vazgeç" })).toHaveFocus();
  fireEvent.click(within(confirm).getByRole("button", { name: "Vazgeç" }));
  expect(api.tagCharts).not.toHaveBeenCalled();
  fireEvent.click(button("Etiketle"));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Sonuçları etiketle" })).getByRole("button", { name: "1 haritayı etiketle" }));
  await settle();
  expect(api.tagCharts).toHaveBeenCalledWith({ chart_ids: [FX.page.hits[0].chart_id], tag: "Ay Terazi" });
  expect(within(dialog()).getByText("“Ay Terazi” etiketi 1 haritaya eklendi.")).toBeInTheDocument();
  expect(onTagged).toHaveBeenCalledTimes(1);
});

test("saved searches load their conditions, save under a name and delete", async () => {
  vi.mocked(api.saveSearch).mockImplementation(async (r) => ({ status: "ok", data: { id: 9, name: r.name, query: r.query, updated_at: "t" } }));
  vi.mocked(api.deleteSavedSearch).mockResolvedValue({ status: "ok", data: null });
  open(EMPTY_QUERY);
  // Liste okunmadan saklanamaz.
  expect(button("Aramayı sakla")).toHaveAccessibleDescription("Kayıtlı aramalar okunuyor.");
  await settle();
  expect(button("Aramayı sakla")).toHaveAccessibleDescription("Saklamak için koşulları tamamlayın.");
  const select = within(dialog()).getByLabelText("Kayıtlı arama");
  fireEvent.change(select, { target: { value: String(FX.saved.id) } });
  expect(current).toEqual(FX.saved.query);
  expect(within(dialog()).getByLabelText("Arama adı")).toHaveValue(FX.saved.name);
  expect(within(dialog()).getAllByRole("group", { name: /^Koşul \d$/ })).toHaveLength(2);
  // Yeni adla saklama: istek yalnızca adı ve sorguyu taşır.
  fireEvent.change(within(dialog()).getByLabelText("Arama adı"), { target: { value: "  Yeni arama " } });
  fireEvent.click(button("Aramayı sakla"));
  await settle();
  expect(api.saveSearch).toHaveBeenCalledWith({ name: "Yeni arama", query: FX.saved.query });
  expect(select).toHaveDisplayValue("Yeni arama");
  expect(within(select).getAllByRole("option")).toHaveLength(3);
  // Silme: seçili kayıt gider, odak seçiciye geçer.
  const remove = button("Aramayı sil");
  remove.focus();
  fireEvent.click(remove);
  await settle();
  expect(api.deleteSavedSearch).toHaveBeenCalledWith(9);
  expect(select).toHaveFocus();
  expect(within(select).getAllByRole("option")).toHaveLength(2);
  expect(button("Aramayı sil")).toHaveAccessibleDescription("Silmek için kayıtlı bir arama seçin.");
});

test("a row changes its kind, offers no retro for the nodes and leaves the focus on the add button when removed", async () => {
  open();
  await settle();
  const row = () => within(dialog()).getByRole("group", { name: "Koşul 1" });
  fireEvent.change(within(row()).getByLabelText("Tür"), { target: { value: "GrahaState" } });
  expect(current.all_of[0].cond).toEqual({ GrahaState: { graha: "Mars", flags: [], dignities: [] } });
  fireEvent.click(within(row()).getByRole("checkbox", { name: "R" }));
  fireEvent.change(within(row()).getByLabelText("Graha"), { target: { value: "Rahu" } });
  // Düğümde retro kutusu yoktur ve işaretli retro kalkar.
  expect(within(row()).queryByRole("checkbox", { name: "R" })).toBeNull();
  expect(current.all_of[0].cond).toEqual({ GrahaState: { graha: "Rahu", flags: [], dignities: [] } });
  fireEvent.click(within(row()).getByRole("checkbox", { name: "değil" }));
  expect(current.all_of[0].negate).toBe(true);
  const remove = within(row()).getByRole("button", { name: "Koşulu sil" });
  remove.focus();
  fireEvent.click(remove);
  expect(current.all_of).toHaveLength(1);
  expect(button("Koşul ekle")).toHaveFocus();
});

test("the surface, its shortcut and its palette command share one name in every language", () => {
  for (const lang of [tr, en, fr]) {
    expect(lang.shortcuts.chartSearch).toBe(lang.search.title);
    expect(lang.palette.command.chartSearch).toBe(lang.search.title);
  }
  // Fransızca metinler tipografik kesme işareti kullanır.
  expect(JSON.stringify(fr.search)).not.toContain("'");
});
