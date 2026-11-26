import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: { computeCompat: vi.fn(), chartInput: vi.fn(), chartBrief: vi.fn(), listClients: vi.fn(), searchClients: vi.fn(), listCharts: vi.fn() },
}));

import { api, type Result } from "../api";
import type { ChartRecord, Client, KutaReport, OpenedClient } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { compatInputs, initialRoles } from "../modules/useCompat";
import { chartInput } from "../modules/input";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { CompatPanel, formatKutaScore, KUTAS } from "./CompatPanel";

const FX = fixture();
const PARTNER = FX.compat.partner_uuid;
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };

/** Açık harita, eş bağıyla (varsayılan: açık harita damat rolünde). */
function linked(roles: "OpenIsGroom" | "OpenIsBride" = "OpenIsGroom", opened: OpenedClient = fixture().opened): OpenedClient {
  opened.view_state.compat_partner_uuid = PARTNER;
  opened.view_state.compat_roles = roles;
  return opened;
}

function renderPanel(opened: OpenedClient | null = linked()) {
  return renderWithStores(<CompatPanel />, { opened });
}

const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const rows = (table: HTMLElement) => within(table).getAllByRole("row").map(cells);
const facts = () => [...document.querySelectorAll(".compat-facts li")].map((li) => li.textContent);
const people = () => [...document.querySelectorAll(".compat-people div")].map((d) => d.textContent);

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.chartBrief).mockImplementation(async () => ok(fixture().compat.brief));
  vi.mocked(api.chartInput).mockImplementation(async () => ok(fixture().compat.input));
  // Rapor rollerden seçilir: açık harita damat rolündeyse `report`, değilse `report_swapped`.
  vi.mocked(api.computeCompat).mockImplementation(async (groom) => ok(groom.jd_ut === FX.opened.chart.jd_ut ? fixture().compat.report : fixture().compat.report_swapped));
});

test("without a partner the panel offers one action and requests nothing", () => {
  renderPanel(fixture().opened);
  expect(screen.getByText("Uyum için bir eş harita seçin.")).toBeInTheDocument();
  expect(screen.getAllByRole("button").map((b) => b.textContent)).toEqual(["Eş harita seç"]);
  expect(screen.queryByRole("table")).toBeNull();
  for (const f of Object.values(api)) expect(f).not.toHaveBeenCalled();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  for (const f of Object.values(api)) expect(f).not.toHaveBeenCalled();
});

test("lists the eight kutas with both classes and the total, the facts and the mutual placement", async () => {
  renderPanel();
  const scores = await screen.findByRole("table", { name: "Ashtakuta puanları" });
  // Eşin girdisi ve kimliği kimlikle okunur; rapor iki girdiyle, damat rolü önce istenir.
  expect(api.chartBrief).toHaveBeenCalledWith(PARTNER);
  expect(api.chartInput).toHaveBeenCalledWith(PARTNER);
  expect(api.computeCompat).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeCompat).mock.calls[0]).toEqual([chartInput(FX.opened), FX.compat.input]);
  expect(people()).toEqual(["1. kişi (damat rolü)Örnek Danışan · Doğum haritası", "2. kişi (gelin rolü)Eş Danışan · Doğum haritası"]);
  // Delhi (Ay Terazi, Swati) ile İstanbul (Ay Başak, Chitra): 19 / 36.
  expect(rows(scores)).toEqual([
    ["Kuta", "1. kişi (damat rolü): Örnek Danışan", "2. kişi (gelin rolü): Eş Danışan", "Puan"],
    ["Varna", "Shudra", "Vaishya", "0 / 1"],
    ["Vashya", "İnsan", "İnsan", "2 / 2"],
    ["Tara", "2 · Sampat", "9 · Parama mitra", "3 / 3"],
    ["Yoni", "Manda", "Kaplan", "1 / 4"],
    ["Graha maitri", "Venüs", "Merkür", "5 / 5"],
    ["Gana", "Deva", "Rakshasa", "0 / 6"],
    ["Bhakoot", "Ter", "Baş", "0 / 7"],
    ["Nadi", "Antya", "Madhya", "8 / 8"],
    ["Toplam", "19 / 36"],
  ]);
  expect(KUTAS).toEqual(FX.compat.report.scores.map((s) => s.kuta));
  // Olgular düz metin satırlarıdır: Bhakoot istisnası (puan 0, lordlar iki yönlü dost) ve iki Kuja dosha satırı.
  expect(facts()).toEqual([
    "Bhakoot istisnası: puan 0; iki Ay burcunun lordu aynı ya da iki yönlü dost. Puan değişmez.",
    "Kuja dosha, Örnek Danışan: lagnadan yok, Ay'dan yok, Venüs'ten var.",
    "Kuja dosha, Eş Danışan: lagnadan var, Ay'dan var, Venüs'ten var.",
  ]);
  const overlay = screen.getByRole("table", { name: "Karşılıklı yerleşim" });
  const placed = rows(overlay);
  expect(placed).toHaveLength(10);
  expect(placed[0]).toEqual(["Graha", "Örnek Danışan grahaları, Eş Danışan haritasında", "Eş Danışan grahaları, Örnek Danışan haritasında"]);
  expect(placed[1]).toEqual(["Gü", "lagnadan 8 · Ay'dan 4", "lagnadan 10 · Ay'dan 6"]);
  // Dokuz satırın tamamı rapordaki on sekiz satırdır: ilk dokuzu damat rolünün grahaları.
  FX.compat.report.overlay.slice(0, 9).forEach((row, i) => {
    const other = FX.compat.report.overlay[i + 9];
    expect(placed[i + 1].slice(1)).toEqual([`lagnadan ${row.house_from_lagna} · Ay'dan ${row.house_from_moon}`, `lagnadan ${other.house_from_lagna} · Ay'dan ${other.house_from_moon}`]);
  });
  expect(screen.getByText("Yöntem: kuta.ashtakuta_v1")).toBeInTheDocument();
  // Doğum saatleri biliniyor: yaklaşık işareti ve uyarı yok.
  expect(document.querySelector(".compat-approx")).toBeNull();
  expect(document.querySelector(".compat-warning")).toBeNull();
  expect(screen.queryByText(/hesap ayarları farklı/)).toBeNull();
});

test("swapping the roles asks again with the inputs swapped and is saved with the chart", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  fireEvent.click(screen.getByRole("button", { name: "Rolleri değiştir" }));
  await waitFor(() => expect(people()[0]).toBe("1. kişi (damat rolü)Eş Danışan · Doğum haritası"));
  await waitFor(() => expect(api.computeCompat).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.computeCompat).mock.calls[1]).toEqual([FX.compat.input, chartInput(FX.opened)]);
  // Eş yeniden okunmaz; yalnızca rapor yeniden istenir.
  expect(api.chartInput).toHaveBeenCalledTimes(1);
  const scores = await screen.findByRole("table", { name: "Ashtakuta puanları" });
  await waitFor(() => expect(rows(scores).at(-1)).toEqual(["Toplam", "21 / 36"]));
  expect(rows(scores)[1]).toEqual(["Varna", "Vaishya", "Shudra", "1 / 1"]);
  expect(rows(scores)[0]).toEqual(["Kuta", "1. kişi (damat rolü): Eş Danışan", "2. kişi (gelin rolü): Örnek Danışan", "Puan"]);
  expect(facts()[1]).toBe("Kuja dosha, Eş Danışan: lagnadan var, Ay'dan var, Venüs'ten var.");
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState).toMatchObject({ compat_partner_uuid: PARTNER, compat_roles: "OpenIsBride" }));
});

test("the roles stored with the chart decide who is the groom", async () => {
  renderPanel(linked("OpenIsBride"));
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(vi.mocked(api.computeCompat).mock.calls[0]).toEqual([FX.compat.input, chartInput(FX.opened)]);
  expect(people()).toEqual(["1. kişi (damat rolü)Eş Danışan · Doğum haritası", "2. kişi (gelin rolü)Örnek Danışan · Doğum haritası"]);
});

test("a partner without a birth time: the report is marked approximate and the Moon changes are named", async () => {
  vi.mocked(api.chartInput).mockImplementation(async () => ok({ ...fixture().compat.input, time_known: false }));
  vi.mocked(api.computeCompat).mockImplementation(async () => ok(fixture().compat.report_time_unknown));
  renderPanel();
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  // Girdi `time_known: false` ile gider: lagnaya bağlı değerleri Rust boş döner (K2).
  expect(vi.mocked(api.computeCompat).mock.calls[0][1].time_known).toBe(false);
  const note = document.querySelector(".compat-approx") as HTMLElement;
  expect(note).toHaveTextContent(/^≈yaklaşık Doğum saati bilinmeyen haritada Ay öğle konumundadır/);
  expect(people()[1]).toBe("2. kişi (gelin rolü)≈yaklaşık Eş Danışan · Doğum haritası");
  expect(people()[0]).toBe("1. kişi (damat rolü)Örnek Danışan · Doğum haritası");
  // Ay o gün hem nakshatra hem burç değiştiriyor: iki uyarı, eşin adıyla, tek satırda.
  const warnings = [...document.querySelectorAll(".compat-warning")].map((w) => w.textContent);
  expect(warnings).toEqual([
    "Eş Danışan: Ay o gün nakshatra değiştiriyor; tara, yoni, gana ve nadi doğum saatine göre değişir. Eş Danışan: Ay o gün burç değiştiriyor; varna, vashya, graha maitri ve Bhakoot doğum saatine göre değişir.",
  ]);
  // Eşin lagnası yok: açık haritanın grahaları ondan sayılamaz; ters yön sayılır.
  const placed = rows(screen.getByRole("table", { name: "Karşılıklı yerleşim" }));
  expect(placed[1]).toEqual(["Gü", "lagnadan — · Ay'dan 4", "lagnadan 10 · Ay'dan 6"]);
  expect(facts().at(-1)).toBe("Kuja dosha, Eş Danışan: lagnadan bilinmiyor, Ay'dan var, Venüs'ten var.");
});

test("an open chart without a birth time sends time_known false and marks its own name", async () => {
  vi.mocked(api.computeCompat).mockImplementation(async () => ok(fixture().compat.report_time_unknown));
  renderPanel(linked("OpenIsBride", unknownTimeOpened()));
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(vi.mocked(api.computeCompat).mock.calls[0].map((i) => i.time_known)).toEqual([true, false]);
  expect(people()).toEqual(["1. kişi (damat rolü)Eş Danışan · Doğum haritası", "2. kişi (gelin rolü)≈yaklaşık Örnek Danışan · Doğum haritası"]);
});

test("different calculation settings of the two charts are pointed out", async () => {
  vi.mocked(api.chartInput).mockImplementation(async () => {
    const input = fixture().compat.input;
    return ok({ ...input, settings: { ...input.settings, ayanamsa: "Raman" as const } });
  });
  renderPanel();
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(screen.getByText("İki haritanın hesap ayarları farklı; her harita kendi ayarıyla hesaplandı.")).toBeInTheDocument();
});

test("a deleted partner chart clears the link and the panel returns to the empty state", async () => {
  vi.mocked(api.chartBrief).mockResolvedValue(ok(null));
  vi.mocked(api.chartInput).mockResolvedValue(ok(null));
  const { persistence } = renderPanel();
  expect(await screen.findByText("Uyum için bir eş harita seçin.")).toBeInTheDocument();
  expect(screen.getAllByRole("button").map((b) => b.textContent)).toEqual(["Eş harita seç"]);
  expect(api.computeCompat).not.toHaveBeenCalled();
  // Bağ görünüm kaydından da çıkar: silinen eş geç gelen bir kayıtla geri yazılmaz.
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState).toMatchObject({ compat_partner_uuid: null, compat_roles: "OpenIsGroom" }));
});

test("removing the partner clears the link and the report", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  fireEvent.click(screen.getByRole("button", { name: "Eşi çıkar" }));
  expect(await screen.findByText("Uyum için bir eş harita seçin.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.compat_partner_uuid).toBeNull());
});

test("an error is shown in the panel with the people and the actions; retry asks again", async () => {
  vi.mocked(api.computeCompat).mockResolvedValueOnce(failure);
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(people()).toHaveLength(2);
  expect(screen.getByRole("button", { name: "Rolleri değiştir" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(api.computeCompat).toHaveBeenCalledTimes(2);
});

test("a failure to read the partner is an error, not a missing partner: the link stays", async () => {
  vi.mocked(api.chartInput).mockResolvedValueOnce(failure);
  const { persistence } = renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(api.computeCompat).not.toHaveBeenCalled();
  expect(persistence.saved).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(api.chartInput).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeCompat).mockResolvedValueOnce(failure);
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Uyum">
      <CompatPanel />
    </section>,
    { opened: linked() },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Uyum" }));
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<KutaReport>) => void;
    vi.mocked(api.computeCompat).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    await act(async () => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    // Yüklenirken boş durum gösterilmez.
    expect(screen.queryByText("Uyum için bir eş harita seçin.")).toBeNull();
    await act(async () => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok(fixture().compat.report)));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(2);
  } finally {
    vi.useRealTimers();
  }
});

test("the tables sit in keyboard-focusable scroll groups named by their captions", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  for (const name of ["Ashtakuta puanları", "Karşılıklı yerleşim"]) {
    const group = screen.getByRole("group", { name });
    expect(group).toHaveAttribute("tabindex", "0");
    expect(within(group).getByRole("table", { name })).toBeInTheDocument();
  }
  expect(screen.getByRole("list", { name: "Ek olgular" })).toBeInTheDocument();
});

test("shared highlight: a graha of the placement table is pinned and its row is marked", async () => {
  renderPanel();
  const overlay = await screen.findByRole("table", { name: "Karşılıklı yerleşim" });
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.graha);
  const mars = within(overlay).getByRole("button", { name: "Mars" });
  expect(mars).toHaveTextContent("Ma");
  fireEvent.click(mars);
  expect(mars).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["Mars"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});

const client = (id: number, name: string, gender: Client["gender"]): Client => ({ ...FX.opened.client, id, name, gender, uuid: `00000000-0000-4000-8000-0000000000${id}0` });
const chartOf = (c: Client, id: number, uuid: string): ChartRecord => ({ ...FX.opened.chart_record, id, client_id: c.id, uuid });

test("choosing a partner from the picker links the chart; the first roles follow the two genders", async () => {
  const mehmet = client(7, "Mehmet Kaya", "Male");
  vi.mocked(api.listClients).mockResolvedValue(ok([mehmet]));
  vi.mocked(api.listCharts).mockResolvedValue(ok([chartOf(mehmet, 70, PARTNER)]));
  const opened = fixture().opened;
  opened.client.gender = "Female";
  const { persistence } = renderPanel(opened);
  fireEvent.click(screen.getByRole("button", { name: "Eş harita seç" }));
  const dialog = screen.getByRole("dialog", { name: "Eş harita seç" });
  fireEvent.click(await within(dialog).findByRole("option", { name: "Mehmet Kaya" }));
  // Tek haritalı danışan: harita doğrudan eş olur ve pencere kapanır.
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.listCharts).toHaveBeenCalledWith(7);
  await screen.findByRole("table", { name: "Ashtakuta puanları" });
  // Açık haritanın danışanı kadın, eşinki erkek: açık harita gelin rolünü alır.
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState).toMatchObject({ compat_partner_uuid: PARTNER, compat_roles: "OpenIsBride" }));
  expect(vi.mocked(api.computeCompat).mock.calls[0]).toEqual([FX.compat.input, chartInput(FX.opened)]);
  // "Eşi değiştir" aynı seçiciyi açar.
  fireEvent.click(screen.getByRole("button", { name: "Eşi değiştir" }));
  expect(screen.getByRole("dialog", { name: "Eş harita seç" })).toBeInTheDocument();
});

test("first roles: the open chart is the groom unless its client is a woman and the partner a man", () => {
  expect(initialRoles("Female", "Male")).toBe("OpenIsBride");
  for (const [open, partner] of [
    ["Male", "Female"],
    ["Male", "Male"],
    ["Female", "Female"],
    [null, "Male"],
    ["Female", null],
    ["Other", "Male"],
    ["Female", "Other"],
    [null, null],
  ] as const) {
    expect(initialRoles(open, partner)).toBe("OpenIsGroom");
  }
  const [a, b] = [chartInput(FX.opened), FX.compat.input];
  expect(compatInputs(a, b, "OpenIsGroom")).toEqual([a, b]);
  expect(compatInputs(a, b, "OpenIsBride")).toEqual([b, a]);
});

test("scores are written as whole numbers and halves", () => {
  expect([0, 0.5, 1, 1.5, 2.5, 19, 36].map(formatKutaScore)).toEqual(["0", "½", "1", "1½", "2½", "19", "36"]);
  expect(formatKutaScore(null)).toBe("—");
});

test("every kuta and every class id of the Rust catalog is named in three languages; classical names are the same", () => {
  expect(Object.keys(FX.kuta_classes).sort()).toEqual([...KUTAS].sort());
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.compat.kutaName)).toEqual([...KUTAS]);
    const classes = lang.compat.class as Record<string, Record<string, string>>;
    for (const kuta of KUTAS) {
      const ids = FX.kuta_classes[kuta];
      // Graha maitrinin sınıfı lordun, Bhakoot'un sınıfı burcun adıdır: var olan anahtarlardan okunur.
      if (kuta === "GrahaMaitri") expect(ids.every((id) => id in lang.grahaName)).toBe(true);
      else if (kuta === "Bhakoot") expect(ids).toHaveLength(Object.keys(lang.sign).length);
      else expect(Object.keys(classes[kuta])).toEqual(ids);
    }
    // Katalogda olmayan bir sınıf grubu durmaz.
    expect(Object.keys(classes).sort()).toEqual(KUTAS.filter((k) => k !== "GrahaMaitri" && k !== "Bhakoot").sort());
  }
  // Klasik adlar üç dilde aynıdır (K11); vashya grupları ve yoni hayvanları çevrilir.
  expect(en.compat.kutaName).toEqual(tr.compat.kutaName);
  expect(fr.compat.kutaName).toEqual(tr.compat.kutaName);
  for (const kuta of ["Varna", "Tara", "Gana", "Nadi"] as const) {
    expect(en.compat.class[kuta]).toEqual(tr.compat.class[kuta]);
    expect(fr.compat.class[kuta]).toEqual(tr.compat.class[kuta]);
  }
});
