import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computePrashnaFacts: vi.fn(), updatePrashna: vi.fn() } }));

import { api, type Result } from "../api";
import type { CommandError, OpenedClient, PrashnaFacts, PrashnaMeta } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { MAX_QUESTION_CHARS } from "../prashna/model";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { RISING_NAMES } from "./labels";
import { PrashnaFactsPanel, QUESTION_SAVE_MS } from "./PrashnaFactsPanel";

const PANEL: Panel = { id: "pf", kind: "prashna_facts", x: 0, y: 0, w: 7, h: 20, settings: {} };
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure: Result<never> = { status: "error", error: { code: "calc", params: { detail: "x" } } };
const QUESTION = "Deneme sorusu: yolculuk bu ay olur mu?";

function Meta() {
  const { opened } = useSession();
  return <output aria-label="kayıt">{opened?.prashna ? `${opened.prashna.focus_house ?? "-"}|${opened.prashna.question}` : "-"}</output>;
}

function renderPanel(opened: OpenedClient | null = fixture().prashna.opened) {
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Prashna olguları">
      <Meta />
      <PrashnaFactsPanel panel={PANEL} />
    </section>,
    { opened },
  );
}

const section = (name: string) => screen.getByRole("region", { name });
const terms = (root: HTMLElement) => [...root.querySelectorAll("dt")].map((dt) => `${dt.textContent}: ${dt.nextElementSibling?.textContent}`);
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const focusSelect = () => screen.getByRole("combobox", { name: "Odak evi" });
const question = () => screen.getByRole("textbox", { name: "Soru" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.computePrashnaFacts).mockImplementation(async (input, focus) => {
    const { prashna } = fixture();
    if (focus === null) return ok(prashna.facts_no_focus);
    return ok(input.lagna_number === null ? prashna.facts : prashna.facts_number);
  });
  vi.mocked(api.updatePrashna).mockImplementation(async (_chartId, edit) =>
    ok<PrashnaMeta>({ ...fixture().prashna.opened.prashna!, question: edit.question.trim(), focus_house: edit.focus_house }),
  );
});

test("a chart that is not a prashna gets a note and no command", () => {
  renderPanel(fixture().opened);
  expect(screen.getByRole("note")).toHaveTextContent("Bu harita bir prashna değil. Yeni bir prashna kurmak için ⌘⇧P.");
  expect(api.computePrashnaFacts).not.toHaveBeenCalled();
  expect(screen.queryByRole("textbox")).toBeNull();
});

test("without an open chart the panel says so", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computePrashnaFacts).not.toHaveBeenCalled();
});

test("draws the facts of the question chart: lagna, lords and their relation, houses, the moon, the moment and the yogas", async () => {
  renderPanel();
  expect(question()).toHaveValue(QUESTION);
  expect(focusSelect()).toHaveDisplayValue("7. ev");
  await screen.findByRole("region", { name: "Lagna" });
  const { opened } = fixture().prashna;
  expect(api.computePrashnaFacts).toHaveBeenCalledTimes(1);
  expect(api.computePrashnaFacts).toHaveBeenCalledWith(
    { jd_ut: opened.chart.jd_ut, lat: opened.chart.geo_lat, lon: opened.chart.geo_lon, settings: opened.chart.meta.settings, time_known: true, lagna_number: null },
    7,
  );
  // Lagna: burç ve derece, tür, doğuş biçimi (klasik ad ve çevrilen ek).
  const lagna = terms(section("Lagna"));
  expect(lagna[0]).toMatch(/^Burç: Koç \d+°\d{2}′$/);
  expect(lagna.slice(1)).toEqual(["Burç türü: yürük", "Doğuş biçimi: Prishthodaya · sırtla doğan"]);
  // Lordlar: lagna lordu Mars Aslan'da 5. evde; odak lordu Venüs Terazi'de 7. evde (burçlar
  // öteki tablolardaki gibi kısaltmayla yazılır).
  const lords = within(section("Lordlar")).getByRole("table");
  expect(lords.querySelector("caption")).toHaveTextContent("Lagna lordu ve odak evinin lordu");
  expect(bodyRows(lords)).toEqual([
    ["Lagna lordu", "Mars", "Asl", "5", "Çok dost", "—"],
    ["Odak lordu", "Venüs", "Ter", "7", "Mulatrikona", "—"],
  ]);
  expect(within(section("Lordlar")).getByRole("list", { name: "İki lordun bağı" })).toHaveTextContent("hiçbiri");
  expect(section("Lordlar")).toHaveTextContent("Odak lordu lagna lordundan 3. burçta; lagna lordu odak lordundan 11. burçta.");
  // Kendralar ve odak evi: sınıf sözle yazılır (renk anlam taşımaz).
  const houses = within(section("Kendralar ve odak evi")).getByRole("table");
  expect(bodyRows(houses)).toEqual([
    ["1. ev", "Koç", "boş"],
    ["4. ev", "Yen", "boş"],
    ["7. ev", "Ter", "Merkür (iyicil), Venüs (iyicil)"],
    ["10. ev", "Oğl", "boş"],
    ["7. ev · odak", "Ter", "Merkür (iyicil), Venüs (iyicil)"],
  ]);
  // Ay ve anın panchangı (v1 haritasından).
  expect(terms(section("Ay"))).toEqual(["Burç: Boğ · 2. ev", "Nakshatra: Rohini · pada 4", "Tithi: Krishna 1 · Pratipada", "Burcuna bakanlar: Güneş, Satürn"]);
  expect(terms(section("Anın panchangı")).map((l) => l.split(":")[0])).toEqual(["Tithi", "Vara", "Nakshatra", "Nitya yoga", "Karana"]);
  expect(terms(section("Anın panchangı"))[0]).toBe("Tithi: Krishna 1 · Pratipada");
  // Yoga listesi (Alt proje 3.5'in kataloğu) ve kapanış satırları; hüküm sözü yoktur.
  const yogas = section("Yogalar");
  expect(yogas.querySelectorAll("tr[data-yoga]")).toHaveLength(fixture().prashna.facts.yogas.hits.length);
  expect(screen.getByText("Bu panel olguları sayar; sorunun yanıtını ya da bir hükmü söylemez.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: prashna.parashari_v1")).toBeInTheDocument();
  const text = document.body.textContent ?? "";
  for (const verdict of ["uğurlu", "uğursuz", "olumlu", "olumsuz", "evet", "hayır"]) expect(text.toLocaleLowerCase("tr")).not.toContain(verdict);
  // Geniş tablolar klavyeyle kaydırılır: adlı ve odaklanır kaplar.
  for (const name of ["Lordlar", "Kendralar ve odak evi"]) {
    const scroll = screen.getByRole("group", { name });
    expect(scroll).toHaveAttribute("tabindex", "0");
  }
});

test("a number prashna says so: the badge, the navamsa middle and the number in the module input", async () => {
  renderPanel(fixture().prashna.opened_number);
  await screen.findByRole("region", { name: "Lagna" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][0].lagna_number).toBe(54);
  expect(screen.getByText("sayı: 54")).toBeInTheDocument();
  expect(terms(section("Lagna"))[0]).toBe("Burç: Baş 28°20′ · 54. navamsanın ortası");
  // Lagna sayıdan gelir: lordlar ve evler Başak'tan sayılır.
  const lords = bodyRows(within(section("Lordlar")).getByRole("table"));
  expect(lords.map((r) => r[0])).toEqual(["Lagna lordu", "Odak lordu"]);
  expect(lords[0][1]).toBe("Merkür");
  expect(lords[1][1]).toBe("Jüpiter");
});

test("the focus house is changed from the panel: saved, written to the session and the facts are read again in place", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  let release: (r: Result<PrashnaFacts>) => void = () => {};
  vi.mocked(api.computePrashnaFacts).mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
  focusSelect().focus();
  fireEvent.change(focusSelect(), { target: { value: "10" } });
  await waitFor(() => expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`10|${QUESTION}`));
  expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: QUESTION, focus_house: 10 });
  await waitFor(() => expect(api.computePrashnaFacts).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[1][1]).toBe(10);
  // Yeni olgular okunurken eskiler ve odak yerinde durur; panel meşgul olduğunu söyler.
  expect(screen.getByRole("region", { name: "Lordlar" })).toBeInTheDocument();
  expect(document.querySelector(".prashna-facts")).toHaveAttribute("aria-busy", "true");
  expect(focusSelect()).toHaveFocus();
  expect(focusSelect()).toHaveDisplayValue("10. ev");
  const next = fixture().prashna.facts;
  next.focus!.house = 10;
  await act(async () => release(ok(next)));
  expect(document.querySelector(".prashna-facts")).not.toHaveAttribute("aria-busy");
  expect(bodyRows(within(section("Kendralar ve odak evi")).getByRole("table")).at(-1)![0]).toBe("10. ev · odak");
  expect(focusSelect()).toHaveFocus();
});

test("with no focus house only the lagna lord is listed and the panel says why", async () => {
  const opened = fixture().prashna.opened;
  opened.prashna = { ...opened.prashna!, focus_house: null };
  renderPanel(opened);
  await screen.findByRole("region", { name: "Lordlar" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][1]).toBeNull();
  expect(focusSelect()).toHaveDisplayValue("Seçilmedi");
  expect(bodyRows(within(section("Lordlar")).getByRole("table")).map((r) => r[0])).toEqual(["Lagna lordu"]);
  expect(section("Lordlar")).toHaveTextContent("Odak evi seçilmedi: odak evinin lordu ve iki lordun bağı gösterilmez.");
  expect(within(section("Lordlar")).queryByRole("list")).toBeNull();
  expect(bodyRows(within(section("Kendralar ve odak evi")).getByRole("table"))).toHaveLength(4);
});

test("the relation lists every link that holds, and says when both houses have the same lord", async () => {
  const both = fixture().prashna.facts;
  both.focus!.relation = { same_graha: false, links: ["Exchange", "MutualAspect"], focus_lord_from_lagna_lord: 7, lagna_lord_from_focus_lord: 7 };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(both));
  const first = renderPanel();
  const list = await screen.findByRole("list", { name: "İki lordun bağı" });
  expect(within(list).getAllByRole("listitem").map((li) => li.textContent)).toEqual(["değişim", "karşılıklı bakış"]);
  first.unmount();
  const same = fixture().prashna.facts;
  same.focus!.lord = { ...same.lagna!.lord };
  same.focus!.relation = { same_graha: true, links: [], focus_lord_from_lagna_lord: 1, lagna_lord_from_focus_lord: 1 };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(same));
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  expect(section("Lordlar")).toHaveTextContent("7. evin lordu da Mars: iki evin lordu aynı graha.");
  expect(within(section("Lordlar")).queryByRole("list")).toBeNull();
});

test("a retrograde, combust lord is marked in words", async () => {
  const facts = fixture().prashna.facts;
  facts.lagna!.lord = { ...facts.lagna!.lord, retrograde: true, combust: true, dignity: "Debilitated" };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(facts));
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  expect(bodyRows(within(section("Lordlar")).getByRole("table"))[0].slice(4)).toEqual(["Düşük", "retro, yanık"]);
});

test("the question is saved a moment after typing stops; a question that is too long is not sent", async () => {
  vi.useFakeTimers();
  try {
    renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Yeni soru" } });
    expect(question()).toHaveAccessibleDescription("9 / 2000 karakter · yazdıkça kaydedilir");
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS - 1));
    expect(api.updatePrashna).not.toHaveBeenCalled();
    fireEvent.change(question(), { target: { value: "Yeni soru mu" } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    expect(api.updatePrashna).toHaveBeenCalledTimes(1);
    expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Yeni soru mu", focus_house: 7 });
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent("7|Yeni soru mu");
    expect(question()).toHaveAccessibleDescription("Kaydedildi");
    // Soru olguları etkilemez: yeniden hesap yok, metin ve odak yerinde.
    expect(api.computePrashnaFacts).toHaveBeenCalledTimes(1);
    expect(question()).toHaveValue("Yeni soru mu");
    // Çok uzun soru gönderilmez ve bunu satır söyler.
    fireEvent.change(question(), { target: { value: "ş".repeat(MAX_QUESTION_CHARS + 1) } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS * 2));
    expect(api.updatePrashna).toHaveBeenCalledTimes(1);
    expect(question()).toHaveAttribute("aria-invalid", "true");
    expect(question()).toHaveAccessibleDescription("2001 / 2000 karakter: soru çok uzun, kaydedilmedi.");
  } finally {
    vi.useRealTimers();
  }
});

test("a pending question is sent when the panel goes away; a failed save is shown and can be retried", async () => {
  vi.useFakeTimers();
  try {
    const first = renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Yarım" } });
    first.unmount();
    await act(async () => vi.advanceTimersByTime(0));
    expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Yarım", focus_house: 7 });
    vi.mocked(api.updatePrashna).mockClear();

    const error: CommandError = { code: "store_unavailable", params: {} };
    vi.mocked(api.updatePrashna).mockResolvedValueOnce({ status: "error", error });
    renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Başka" } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Kaydedilemedi: Danışan veritabanı kullanılamıyor.");
    // Yazılan metin durur; kayıt değişmedi.
    expect(question()).toHaveValue("Başka");
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`7|${QUESTION}`);
    fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
    await act(async () => vi.advanceTimersByTime(0));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent("7|Başka");
  } finally {
    vi.useRealTimers();
  }
});

test("a focus house chosen while a question save is still due is not undone by that save", async () => {
  vi.useFakeTimers();
  try {
    const chartId = fixture().prashna.opened.chart_record.id;
    // İlk kayıt yanıtı bekletilir: zamanlayıcı ateşlediğinde oturumdaki odak evi hâlâ eskidir.
    let release: () => void = () => {};
    vi.mocked(api.updatePrashna).mockImplementationOnce(
      (_chartId, edit) =>
        new Promise((resolve) => {
          release = () => resolve(ok<PrashnaMeta>({ ...fixture().prashna.opened.prashna!, question: edit.question.trim(), focus_house: edit.focus_house }));
        }),
    );
    renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Taslak" } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS / 2));
    fireEvent.change(focusSelect(), { target: { value: "10" } });
    expect(focusSelect()).toHaveDisplayValue("10. ev");
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    await act(async () => release());
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    // Her kayıt seçilen evi taşır; sıradaki son kayıt eski evi geri yazmaz.
    for (const call of vi.mocked(api.updatePrashna).mock.calls) expect(call).toEqual([chartId, { question: "Taslak", focus_house: 10 }]);
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent("10|Taslak");
    expect(focusSelect()).toHaveDisplayValue("10. ev");
  } finally {
    vi.useRealTimers();
  }
});

test("a focus house change with a draft that is too long saves the house with the stored question", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  const long = "ş".repeat(MAX_QUESTION_CHARS + 1);
  fireEvent.change(question(), { target: { value: long } });
  fireEvent.change(focusSelect(), { target: { value: "10" } });
  await waitFor(() => expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`10|${QUESTION}`));
  expect(api.updatePrashna).toHaveBeenCalledTimes(1);
  expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: QUESTION, focus_house: 10 });
  // Taslak ve uyarısı yerinde durur.
  expect(question()).toHaveValue(long);
  expect(question()).toHaveAccessibleDescription("2001 / 2000 karakter: soru çok uzun, kaydedilmedi.");
  expect(screen.queryByRole("alert")).toBeNull();
});

test("a failed focus house save keeps the chosen house and retry sends it again", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  vi.mocked(api.updatePrashna).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  fireEvent.change(focusSelect(), { target: { value: "10" } });
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Kaydedilemedi: Danışan veritabanı kullanılamıyor.");
  // Seçim geri atlamaz; kayıt eski evdedir ve satır bunu söyler.
  expect(focusSelect()).toHaveDisplayValue("10. ev");
  expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`7|${QUESTION}`);
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`10|${QUESTION}`));
  expect(api.updatePrashna).toHaveBeenLastCalledWith(fixture().prashna.opened.chart_record.id, { question: QUESTION, focus_house: 10 });
  expect(screen.queryByRole("alert")).toBeNull();
});

test("the occupants cell is named by its own content: the graha buttons stay reachable by name", async () => {
  renderPanel();
  const table = within(await screen.findByRole("region", { name: "Kendralar ve odak evi" })).getByRole("table");
  expect(table.querySelector("td[aria-label]")).toBeNull();
  expect(within(table).getAllByRole("button").length).toBeGreaterThan(4);
});

test("an error keeps the question and the focus house; retry reads again and shows nothing from before the error", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  // Odak evi değişir, okuma düşer: hata panelin içindedir; soru ve seçim yerinde durur.
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(failure);
  fireEvent.change(focusSelect(), { target: { value: "4" } });
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  expect(screen.getByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(question()).toHaveValue(QUESTION);
  expect(focusSelect()).toHaveDisplayValue("4. ev");
  expect(screen.queryByRole("region", { name: "Lordlar" })).toBeNull();
  // Yeniden deneme boş başlar: hatadan önceki olgular geri gelmez.
  let release: (r: Result<PrashnaFacts>) => void = () => {};
  vi.mocked(api.computePrashnaFacts).mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
  retry.focus();
  fireEvent.click(retry);
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  expect(screen.queryByRole("region", { name: "Lordlar" })).toBeNull();
  const next = fixture().prashna.facts;
  next.focus!.house = 4;
  await act(async () => release(ok(next)));
  expect(screen.getByRole("region", { name: "Lordlar" })).toBeInTheDocument();
  // Odaktaki düğme kalkınca odak gövdeye düşmez: panelin içinde durur.
  expect(document.activeElement).not.toBe(document.body);
});

test("a chart whose time is unknown shows no lagna facts and says why; the moon and the yogas remain", async () => {
  const facts = fixture().prashna.facts;
  facts.lagna = null;
  facts.focus = null;
  facts.moon.house = null;
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(facts));
  const opened = fixture().prashna.opened;
  opened.chart_record.birth.time = null;
  opened.chart_record.birth.time_certainty = "Unknown";
  renderPanel(opened);
  await screen.findByRole("region", { name: "Ay" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByRole("note")).toHaveTextContent("Bu haritanın saati bilinmiyor: lagnaya bağlı olgular (lagna, lordlar, evler) gösterilmez.");
  for (const name of ["Lagna", "Lordlar", "Kendralar ve odak evi"]) expect(screen.queryByRole("region", { name })).toBeNull();
  expect(terms(section("Ay"))[0]).toBe("Burç: Boğ");
  expect(section("Yogalar")).toBeInTheDocument();
});

test("the lagna lord is a shared highlight: pinning it marks its row", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  const table = within(section("Lordlar")).getByRole("table");
  const mars = within(table).getByRole("button", { name: "Mars" });
  expect(mars).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(mars);
  expect(mars).toHaveAttribute("aria-pressed", "true");
  expect(table.querySelector('tr[data-lord="lagna"]')).toHaveClass("is-highlighted");
  expect(table.querySelector('tr[data-lord="focus"]')).not.toHaveClass("is-highlighted");
});

test("another prashna starts with its own question, not the draft of the previous one", async () => {
  function Switch() {
    const session = useSession();
    return (
      <button type="button" onClick={() => session.open({ ...fixture().prashna.opened_number, prashna: { ...fixture().prashna.opened_number.prashna!, question: "İkinci soru" } })}>
        öteki
      </button>
    );
  }
  renderWithStores(
    <>
      <Switch />
      <PrashnaFactsPanel panel={PANEL} />
    </>,
    { opened: fixture().prashna.opened },
  );
  await screen.findByRole("region", { name: "Lagna" });
  fireEvent.change(question(), { target: { value: "Taslak" } });
  fireEvent.click(screen.getByRole("button", { name: "öteki" }));
  await waitFor(() => expect(question()).toHaveValue("İkinci soru"));
  // Önceki haritanın taslağı kendi haritasına kaydedilir.
  expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Taslak", focus_house: 7 });
});

test("every rising mode, link, kind and nature has a name in three languages; classical names are the same in each", () => {
  const { prashna_ids } = fixture();
  expect(Object.keys(RISING_NAMES).sort()).toEqual([...prashna_ids.rising].sort());
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.prashna.rising).sort()).toEqual([...prashna_ids.rising].sort());
    expect(Object.keys(lang.prashna.link).sort()).toEqual([...prashna_ids.links].sort());
    expect(Object.keys(lang.prashna.kind).sort()).toEqual(["Dual", "Fixed", "Movable"]);
    expect(Object.keys(lang.prashna.nature).sort()).toEqual(["Benefic", "Malefic"]);
    expect(lang.panel.prashna_facts).toContain(lang === fr ? "prashna" : lang === en ? "Prashna" : "Prashna");
    expect(lang.layout.prashna).toBe("Prashna");
  }
  // Fransızca metinler tipografik kesme işareti kullanır.
  expect(JSON.stringify(fr.prashna)).not.toContain("'");
});
