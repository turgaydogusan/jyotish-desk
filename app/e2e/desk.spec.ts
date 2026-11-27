import { expect, test } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

test("birth entry: empty state, atlas place by keyboard, preview, save opens the chart", async ({ page }) => {
  await mockIpc(page, { session: false });
  await page.goto("/");
  await page.getByRole("region", { name: "Açık harita yok" }).getByRole("button", { name: "Yeni harita" }).click();
  const form = page.getByRole("dialog", { name: "Yeni harita" });
  await expect(form.getByLabel("Danışan adı", { exact: true })).toBeFocused();
  await form.getByLabel("Danışan adı", { exact: true }).fill("Örnek Danışan");
  await form.getByLabel("Tarih", { exact: true }).fill("2000-01-01");
  await form.getByLabel("Saat", { exact: true }).fill("17:30");
  const place = form.getByRole("combobox", { name: "Doğum yeri" });
  await place.fill("new delhi");
  await expect(form.getByRole("listbox", { name: "Bulunan yerler" }).getByRole("option").first()).toContainText("New Delhi");
  await place.press("Enter");
  await expect(form.getByLabel("Bölge adı", { exact: true })).toHaveValue("Asia/Kolkata");
  await expect(form.getByRole("region", { name: "Önizleme" })).toContainText("UTC 2000-01-01 12:00:00 · IST +05:30");
  await form.getByRole("button", { name: "Haritayı kaydet" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
  const created = (await ipcCalls(page, "create_client")) as { input: { chart: { birth: { tz: unknown } } } }[];
  expect(created[0].input.chart.birth.tz).toEqual({ Iana: "Asia/Kolkata" });
});

test("classic layout with Cmd+5: D1, panchang and D9 above; Vimshottari, chara dasha and karakas below", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit5");
  await expect(page.getByRole("combobox", { name: "Düzen" })).toHaveValue("builtin:classic");
  await expect(page.getByRole("combobox", { name: "Düzen" }).locator("option:checked")).toHaveText("Klasik");
  await expect(page.getByRole("region", { name: "Harita D1", exact: true }).getByRole("group", { name: "D1 haritası" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Panchang", exact: true })).toBeVisible();
  await expect(page.getByRole("region", { name: "Harita D9", exact: true }).getByRole("group", { name: "D9 haritası" })).toBeVisible();
  // İki dasa ağacı bölgesi sistemin adıyla ayrışır: aynı adı taşıyan iki bölge yoktur.
  await expect(page.getByRole("region", { name: "Dasa ağacı" })).toHaveCount(2);
  const vimshottari = page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true });
  const chara = page.getByRole("region", { name: "Dasa ağacı Chara dasha", exact: true });
  await expect(vimshottari.getByRole("combobox", { name: "Sistem" }).locator("option:checked")).toHaveText("Vimshottari");
  await expect(chara.getByRole("combobox", { name: "Sistem" }).locator("option:checked")).toHaveText("Chara dasha");
  await expect(chara.getByText("Yön: geri")).toBeVisible();
  // Yerleşim: D1 solda 7 × 18; alt sıra üç eşit sütun.
  const box = (name: string) =>
    page.getByRole("region", { name, exact: true }).evaluate((el) => ["x", "y", "w", "h"].map((k) => Number((el as HTMLElement).dataset[k])));
  expect(await box("Harita D1")).toEqual([0, 0, 7, 18]);
  expect(await box("Panchang")).toEqual([7, 0, 5, 8]);
  expect(await box("Harita D9")).toEqual([7, 8, 5, 10]);
  expect(await box("Dasa ağacı Vimshottari")).toEqual([0, 18, 4, 12]);
  expect(await box("Dasa ağacı Chara dasha")).toEqual([4, 18, 4, 12]);
  expect(await box("Chara karaka")).toEqual([8, 18, 4, 12]);
  await expect(page.getByRole("region", { name: "Chara karaka" })).toBeVisible();
  // Klasik düzende chara dasha açıktır: tek bir hesap istenir.
  await expect.poll(async () => (await ipcCalls(page, "compute_chara_dasha")).length).toBe(1);
});

test("layout switch with Cmd+2 and a keyboard panel move that is saved with the chart", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  await expect(page.getByRole("combobox", { name: "Düzen" }).locator("option:checked")).toHaveText("Araştırma");
  await expect(page.getByRole("region", { name: "Graha tablosu" })).toBeVisible();
  const table = page.getByRole("region", { name: "Graha tablosu" });
  const before = Number(await table.getAttribute("data-y"));
  await table.focus();
  await page.keyboard.press("Alt+ArrowDown");
  await expect(table).toHaveAttribute("data-y", String(before + 1));
  await expect.poll(async () => (await ipcCalls(page, "save_view_state")).length).toBeGreaterThan(0);
  const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { chartId: number };
  expect(saved.chartId).toBe(1);
});

test("hovering a graha row highlights it in the chart; the style selector redraws", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  const row = page.getByRole("region", { name: "Graha tablosu" }).locator('tr[data-graha="Saturn"]');
  await row.hover();
  await expect(page.locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);
  const chart = page.getByRole("region", { name: "Harita D1", exact: true });
  await chart.getByRole("combobox", { name: "Biçim" }).selectOption("south");
  await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", "south");
});

test("dasha tree by keyboard loads level 4 on demand", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  const tree = page.getByRole("tree", { name: "Dasa ağacı" });
  await expect(tree).toBeVisible();
  await tree.getByRole("treeitem").first().focus();
  // Sağ ok: kapalıysa açar, açıksa ilk alt döneme iner (1 → 2 → 3. seviye).
  for (const key of ["ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight"]) await page.keyboard.press(key);
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "3");
  await expect(tree.getByRole("treeitem", { name: /^Rahu–Rahu–Rahu–Rahu, / })).toBeVisible();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "4");
  await expect.poll(async () => (await ipcCalls(page, "dasha_children")).length).toBe(1);
});

test("spec §8: Cmd+K, a name and Enter open that client's chart with no form or confirmation", async ({ page }) => {
  await mockIpc(page, { session: true });
  // Yerel pencere (confirm, alert, prompt) açılırsa kaydedilir ve test sonunda düşer.
  const nativeDialogs: string[] = [];
  page.on("dialog", (d) => {
    nativeDialogs.push(d.type());
    void d.dismiss();
  });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+KeyK");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await expect(palette.getByRole("combobox")).toBeFocused();
  await page.keyboard.type("ayşe");
  await expect(palette.getByRole("option").first()).toContainText("Ayşe Yılmaz");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
  // Harita açıkken ne form ne onay penceresi vardır (spec §8).
  await expect(page.locator("form")).toHaveCount(0);
  await expect(page.getByRole("alertdialog")).toHaveCount(0);
  expect(nativeDialogs).toEqual([]);
  expect(await ipcCalls(page, "open_client")).toEqual([{ clientId: 2, chartId: null }]);
});

test("the sidebar opens a recent client, a new note is saved as it is typed, and Cmd+\\ from the note hides the sidebar", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  const sidebar = page.getByRole("complementary", { name: "Danışanlar" });
  await sidebar.getByRole("button", { name: "Ayşe Yılmaz" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
  await expect(sidebar.getByRole("button", { name: "Ayşe Yılmaz" })).toHaveAttribute("aria-current", "true");
  const notes = page.getByRole("region", { name: "Notlar" });
  await notes.getByRole("button", { name: "Yeni not" }).click();
  const box = notes.getByRole("textbox");
  await expect(box).toBeFocused();
  await page.keyboard.type("Satürn dönüşü konuşuldu");
  await page.keyboard.press("Tab");
  await expect.poll(async () => ((await ipcCalls(page, "update_note")).at(-1) as { body?: string } | undefined)?.body).toBe("Satürn dönüşü konuşuldu");
  // ⌘ kısayolları not kutusunun içinden de çalışır (Plan 10 incelemesi MF1).
  await box.focus();
  await page.keyboard.press("Meta+Backslash");
  await expect(sidebar).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Danışanlar" })).toHaveAttribute("aria-expanded", "false");
});

test("Cmd+, edits the open chart's settings; the change is saved for that chart and the difference badge appears", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Comma");
  const dialog = page.getByRole("dialog", { name: "Hesap ayarları" });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("combobox", { name: "Ayanamsa" }).selectOption({ label: "Raman" });
  await expect.poll(async () => (await ipcCalls(page, "save_calc_settings")).length).toBe(1);
  const [call] = (await ipcCalls(page, "save_calc_settings")) as { chartId: number; settings: { ayanamsa: unknown } }[];
  expect(call.chartId).toBe(1);
  expect(call.settings.ayanamsa).toBe("Raman");
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole("button", { name: /Ayar farkı/ })).toBeVisible();
});

test("the transit layout shows Now; a chosen moment is applied and Now returns to the present", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit3");
  const table = page.getByRole("region", { name: "Transit tablosu" });
  await expect(table).toBeVisible();
  const now = table.getByRole("button", { name: "Şimdi" });
  await expect(now).toHaveAttribute("aria-pressed", "true");
  await table.getByLabel("Tarih").fill("2030-01-01");
  await table.getByLabel("Saat").fill("12:00");
  await table.getByRole("button", { name: "Uygula" }).click();
  await expect(now).toHaveAttribute("aria-pressed", "false");
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain("2030-01-01");
  await now.click();
  await expect(now).toHaveAttribute("aria-pressed", "true");
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain('"Now"');
});

test("a tag written on the open chart appears in the tag filter and finds that client", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const sidebar = page.getByRole("complementary", { name: "Danışanlar" });
  const box = sidebar.getByRole("textbox", { name: "Etiketler" });
  await box.fill("Kariyer");
  await box.press("Enter");
  const filter = sidebar.getByRole("combobox", { name: "Etiket" });
  await expect(filter.locator("option", { hasText: "Kariyer (1)" })).toHaveCount(1);
  await filter.selectOption({ label: "Kariyer (1)" });
  const list = sidebar.getByRole("region", { name: "“Kariyer” etiketli" });
  await expect(list.getByRole("button")).toHaveCount(1);
  await expect(list.getByRole("button")).toContainText("Örnek Danışan");
  expect((await ipcCalls(page, "list_clients_by_tag")).at(-1)).toEqual({ tag: "Kariyer", limit: 200 });
});

test("Cmd+P opens the print preview; under print media only the sheets remain; Cmd+P again asks for the print dialog", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+p");
  const toolbar = page.getByRole("group", { name: "Baskı seçenekleri" });
  const sheets = page.locator("section.print-page");
  await expect(toolbar).toBeVisible();
  await expect(sheets).toHaveCount(4);
  await expect(page).toHaveTitle("Örnek Danışan – Doğum haritası");
  expect(await ipcCalls(page, "print_window")).toHaveLength(0);

  await page.emulateMedia({ media: "print" });
  await expect(toolbar).toBeHidden();
  await expect(page.locator("header.topbar")).toBeHidden();
  await expect(sheets.first()).toHaveCSS("break-after", "page");
  await expect(sheets.last()).toHaveCSS("break-after", "auto");
  // 15 mm iç dolgu = kâğıdın kenar boşluğu (96 dpi'de 56,69 px).
  await expect(sheets.first()).toHaveCSS("padding-top", /^56\.6\d*px$/);
  const retro = (row: string) => page.locator(`[data-page="grahas"] [data-row="${row}"] td`).nth(5);
  await expect(retro("Saturn")).toHaveText("R");
  await expect(retro("Rahu")).toHaveText("");
  await expect(retro("Ketu")).toHaveText("");
  await page.emulateMedia({ media: "screen" });

  await page.keyboard.press("Meta+p");
  await expect.poll(() => ipcCalls(page, "print_window")).toHaveLength(1);
  await toolbar.getByRole("checkbox", { name: "Vimshottari dasa" }).uncheck();
  await expect(sheets).toHaveCount(3);
  await page.getByRole("button", { name: "Panoya dön" }).click();
  await expect(page.getByRole("region", { name: "Harita D1", exact: true })).toBeVisible();
  await expect(page).toHaveTitle("Jyotish Desk");
});

test("print sheets are paper-white in the dark theme and, at their longest, each fits one A4 and one Letter page", async ({ page }) => {
  // Saati bilinmeyen harita: kapakta uyarı, harita ve dasa sayfalarında not basılır.
  await mockIpc(page, { session: true, unknownTime: true });
  await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "dark"));
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+p");
  const sheets = page.locator("section.print-page");
  await expect(sheets).toHaveCount(4);
  await expect(page.locator('[data-page="charts"] .print-note')).toContainText("Doğum saati bilinmiyor");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  for (const sheet of await sheets.all()) {
    await expect(sheet).toHaveCSS("background-color", "rgb(255, 255, 255)");
    await expect(sheet).toHaveCSS("color", "rgb(44, 44, 42)");
  }
  await expect(page.locator('[data-page="charts"] .chart-cell').first()).toHaveCSS("stroke", "rgb(83, 74, 183)");

  // WKWebView yazdırırken içeriği yaklaşık 1,067 kat büyütür (scripts/print-probe.sh ölçümü):
  // kâğıdın CSS karşılığı A4'te 744 × 1053 px, Letter'da 765 × 990 px. En dar genişlik ve en
  // kısa yükseklik birlikte denenir.
  await page.setViewportSize({ width: 744, height: 990 });
  await page.emulateMedia({ media: "print" });
  for (const sheet of await sheets.all()) {
    const box = (await sheet.boundingBox())!;
    expect(box.width).toBeLessThanOrEqual(744);
    expect(box.height).toBeGreaterThan(200);
    expect(box.height).toBeLessThanOrEqual(990);
    // Hiçbir tablo ya da çizim kâğıdın iç genişliğinden taşmaz.
    expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
  }
});

test("updates: nothing is checked until asked; About checks, shows the notes and installs the consented version", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.getByRole("button", { name: "Hakkında" }).click();
  const about = page.getByRole("dialog", { name: "Jyotish Desk hakkında" });
  await expect(about.getByText("Sürüm 0.9.0")).toBeVisible();
  await expect(about.getByText("https://github.com/turgaydogusan/jyotish-desk")).toBeVisible();
  await expect(about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" })).not.toBeChecked();
  expect(await ipcCalls(page, "check_for_update")).toHaveLength(0);

  await about.getByRole("button", { name: "Şimdi denetle" }).click();
  await expect(about.getByRole("status")).toHaveText("Yeni sürüm: 0.9.1 (2026-12-01).");
  await expect(about.getByLabel("0.9.1 sürüm notları")).toHaveText("- Baskı düzeltmesi");
  expect(await ipcCalls(page, "install_update")).toHaveLength(0);
  await about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" }).click();
  await expect(about.getByRole("status")).toContainText("0.9.1 kuruldu.");
  expect(await ipcCalls(page, "install_update")).toEqual([{ version: "0.9.1" }]);
  await expect(about.getByRole("button", { name: "Kapat" })).toBeFocused();

  // Tercih açılınca bir sonraki açılış denetler ve bildirim bandında haber verir.
  await about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" }).check();
  await page.reload();
  await expect(page.getByRole("region", { name: "Bildirimler" })).toContainText("Yeni sürüm var: 0.9.1.");
  expect(await ipcCalls(page, "check_for_update")).toHaveLength(1);
});

test("module panels: the palette adds Aspects, Panchang and Ashtakavarga; aspects are computed once and share the highlight", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async (query: string, name: RegExp) => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill(query);
    await palette.getByRole("option", { name }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  // Modül sonucu panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await ipcCalls(page, "compute_drishti")).toHaveLength(0);

  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
  const drishti = page.getByRole("region", { name: "Bakışlar" });
  await expect(drishti).toBeFocused();
  await expect(drishti).toBeInViewport();
  await expect(drishti.getByRole("table")).toBeVisible();
  const calls = (await ipcCalls(page, "compute_drishti")) as { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } }[];
  expect(calls).toHaveLength(1);
  expect(calls[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
  // Jüpiter Koç'tan Yay'daki Güneş'e tam bakar; virupa görünümü sayıyı gösterir.
  const jupiter = drishti.locator('tbody tr[data-graha="Jupiter"]');
  await expect(jupiter.locator("td").first()).toContainText("●");
  await drishti.getByRole("combobox", { name: "Değer" }).selectOption("virupa");
  await expect(jupiter.locator("td").first()).toHaveText("52.4");
  expect(await ipcCalls(page, "compute_drishti")).toHaveLength(1);

  await toggle("panchang", /Panchang panelini aç\/kapat/);
  const panchang = page.getByRole("region", { name: "Panchang" });
  await expect(panchang.locator('tr[data-row="tithi"]')).toContainText("Krishna 11 · Ekadashi");
  await expect(panchang.locator('tr[data-row="tithi"]')).toContainText("2000-01-02 13:19");
  await toggle("ashtakavarga", /Ashtakavarga panelini aç\/kapat/);
  const ashtakavarga = page.getByRole("region", { name: "Ashtakavarga" });
  await expect(ashtakavarga.locator('tr[data-row="sav"]')).toContainText("337");
  await expect(ashtakavarga).toBeFocused();

  // Ortak vurgu: bakış tablosundaki Jüpiter satırı haritada ve ashtakavarga satırında da vurgulanır.
  await jupiter.getByRole("button", { name: "Jüpiter" }).hover();
  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(ashtakavarga.locator('tr[data-graha="Jupiter"]')).toHaveClass(/is-highlighted/);

  // Paneller düzenle birlikte kaydedilir.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.map((p) => p.kind);
    })
    .toEqual(["chart", "chart", "dasha_strip", "notes", "drishti", "panchang", "ashtakavarga"]);

  // Aynı komut paneli kaldırır.
  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
  await expect(drishti).toHaveCount(0);
  await expect(panchang).toBeVisible();
  // Odak gövdeye düşmez; diğer panellerin eklenip kaldırılması bakışları yeniden istemez.
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
  expect(await ipcCalls(page, "compute_drishti")).toHaveLength(1);
});

test("jaimini panels: karakas, arudha padas and argala come from one command; chart marks, rashi drishti and the karaka scheme setting", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async (query: string, name: RegExp) => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill(query);
    await palette.getByRole("option", { name }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type JaiminiCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number; advanced: { karaka_scheme: string; jaimini_co_lords: boolean } } } };
  const jaiminiCalls = async () => (await ipcCalls(page, "compute_jaimini")) as JaiminiCall[];
  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await jaiminiCalls()).toHaveLength(0);

  await toggle("karaka", /Chara karaka panelini aç\/kapat/);
  const karakas = page.getByRole("region", { name: "Chara karaka" });
  await expect(karakas).toBeFocused();
  await expect(karakas).toBeInViewport();
  await expect(karakas.locator('tr[data-role="Atma"]')).toContainText("Ay");
  await expect(karakas.locator('tr[data-role="Amatya"]')).toContainText("11°11′ (etkin 18°48′)");
  await expect(karakas.locator(".jaimini-karakamsha")).toContainText("Karakamsha: Bal");
  const first = await jaiminiCalls();
  expect(first).toHaveLength(1);
  expect(first[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2, advanced: { karaka_scheme: "Eight", jaimini_co_lords: true } } });

  await toggle("arudha", /Arudha padalar panelini aç\/kapat/);
  const arudha = page.getByRole("region", { name: "Arudha padalar" });
  await expect(arudha.locator('tr[data-pada="1"]')).toContainText("A1 · AL");
  await expect(arudha.locator('tr[data-pada="1"]')).toContainText("Bal*");
  await expect(arudha.locator('tr[data-pada="12"]')).toContainText("A12 · UL");

  await toggle("argala", /Argala panelini aç\/kapat/);
  const argala = page.getByRole("region", { name: "Argala" });
  await expect(argala.locator('tr[data-pair="5"]')).toContainText("çekişmeli");
  // Referans ortak seçimi izler: haritada Ay'a tıklanınca Ay'ın burcu (Terazi) referans olur.
  await argala.getByRole("combobox", { name: "Referans" }).selectOption("selected");
  await expect(argala.getByText(/^Bir burç seçin:/)).toBeVisible();
  await page.locator('svg [data-graha="Moon"]').first().click();
  await expect(argala.getByText("Referans: Ter")).toBeVisible();
  // Ortak vurgu: sabitlenen Ay karaka tablosunda da vurgulanır.
  await expect(karakas.locator('tr[data-role="Atma"]')).toHaveClass(/is-highlighted/);
  await page.keyboard.press("Escape");

  // Harita panelinde arudha işaretleri: on iki pada burçlarına yazılır.
  const d1 = page.getByRole("region", { name: "Harita D1" });
  await d1.getByRole("checkbox", { name: "Arudha padalar" }).check();
  await expect(d1.locator(".chart-mark")).toHaveCount(12);
  await expect(d1.locator('[data-mark="AL"]')).toHaveAttribute("data-house", "10");
  // D9 panelinde işaret kutusu kapalıdır: odaklanır, nedeni görünür, tıklamak bir şey yazmaz.
  const d9 = page.getByRole("region", { name: "Harita D9" });
  const d9Marks = d9.getByRole("checkbox", { name: "Arudha padalar" });
  await expect(d9Marks).toHaveAttribute("aria-disabled", "true");
  await expect(d9.getByText("Arudha padalar yalnızca D1'de yazılır.")).toBeVisible();
  await d9Marks.focus();
  await page.keyboard.press("Space");
  await expect(d9Marks).toBeFocused();
  await expect(d9Marks).not.toBeChecked();
  await expect(d9.locator(".chart-mark")).toHaveCount(0);

  // Bakışlar panelinde rashi türü aynı raporu okur; graha bakışı istenmez.
  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
  const drishti = page.getByRole("region", { name: "Bakışlar" });
  await drishti.getByRole("combobox", { name: "Tür" }).selectOption("rashi");
  await expect(drishti.locator("tbody tr")).toHaveCount(9);
  await expect(drishti.locator('tbody tr[data-graha="Ketu"] td').nth(5)).toContainText("●");

  // Her tüketici raporu bir kez ister: üç panel, harita işaretleri ve rashi matrisi.
  await expect.poll(async () => (await jaiminiCalls()).length).toBe(5);

  // Ayar: karaka şeması yedili yapılınca harita yeniden hesaplanır ve rapor yeni ayarla istenir.
  await page.keyboard.press("Meta+Comma");
  const settings = page.getByRole("dialog", { name: "Hesap ayarları" });
  await settings.getByLabel("Chara karaka şeması").selectOption("Seven");
  await expect.poll(async () => (await ipcCalls(page, "save_calc_settings")).length).toBe(1);
  const [saved] = (await ipcCalls(page, "save_calc_settings")) as { settings: { advanced: unknown } }[];
  expect(saved.settings.advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
  await settings.getByRole("button", { name: "Kapat" }).click();
  await expect.poll(async () => (await jaiminiCalls()).length).toBe(10);
  const last = (await jaiminiCalls()).at(-1)!;
  expect(last.input.settings.advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
  // Fark rozeti ileri modül ayarını da adlandırır.
  await expect(page.getByText(/Ayar farkı/).first()).toBeVisible();

  // Paneller ve ayarları düzenle birlikte kaydedilir.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.map((p) => [p.kind, p.settings.arudha_marks ?? p.settings.reference ?? p.settings.kind ?? null]);
    })
    .toEqual([
      ["chart", true],
      ["chart", null],
      ["dasha_strip", null],
      ["notes", null],
      ["jaimini_karakas", null],
      ["arudha", null],
      ["argala", "selected"],
      ["drishti", "rashi"],
    ]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await toggle("karaka", /Chara karaka panelini aç\/kapat/);
  await expect(karakas).toHaveCount(0);
  await expect(arudha).toBeVisible();
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("chara dasha: the system selector in the dasha tree and strip; each system keeps its own open path; the strip marks the period Rust names", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  const panel = page.getByRole("region", { name: "Dasa ağacı" });
  const tree = panel.getByRole("tree", { name: "Dasa ağacı" });
  await expect(tree.getByRole("treeitem")).toHaveCount(9);
  // Chara dasha seçilmeden istenmez (ileri modüller spec'i K1).
  expect(await ipcCalls(page, "compute_chara_dasha")).toHaveLength(0);

  const system = panel.getByRole("combobox", { name: "Sistem" });
  await expect(system.locator("option:checked")).toHaveText("Vimshottari");
  await system.selectOption("chara");
  await expect(panel.getByText("Yön: geri")).toBeVisible();
  await expect(panel.getByText("Yıllar: İki 6 · Boğ 6 · Koç 10 · Bal 11 · Kov 10 · Oğl 9 · Yay 4 · Akr 3 · Ter 1 · Baş 9 · Asl 8 · Yen 9")).toBeVisible();
  await expect(tree.getByRole("treeitem")).toHaveCount(20);
  await expect(tree.getByRole("treeitem").first()).toHaveAccessibleName("İki, 2000-01-01 – 2006-01-01");
  await expect(tree.getByRole("treeitem", { name: /^2\. İki, / })).toBeVisible();
  const computed = (await ipcCalls(page, "compute_chara_dasha")) as { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } }[];
  expect(computed).toHaveLength(1);
  expect(computed[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });

  // Klavye: sağ ok açar, sonra ilk alt döneme iner; 3. seviye istenerek yüklenir ve son seviyedir.
  await tree.getByRole("treeitem").first().focus();
  for (const key of ["ArrowRight", "ArrowRight", "ArrowRight"]) await page.keyboard.press(key);
  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ–Koç, / })).toBeVisible();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "3");
  await expect(page.locator(":focus")).not.toHaveAttribute("aria-expanded");
  const children = (await ipcCalls(page, "chara_dasha_children")) as { path: string[]; cycle: number }[];
  expect(children.map((c) => [c.path, c.cycle])).toEqual([[["Gemini", "Taurus"], 1]]);

  // Açık dal sistem başına ayrı kaydedilir; sistem panelin ayarıdır.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as
        | { viewState: { dasha_expanded_path: string[]; chara_expanded_path: unknown; panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } }
        | undefined;
      const layout = saved?.viewState.panel_layout;
      const settings = layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.find((p) => p.kind === "dasha_tree")?.settings;
      return [saved?.viewState.chara_expanded_path, saved?.viewState.dasha_expanded_path, settings?.system];
    })
    .toEqual([{ cycle: 1, signs: ["Gemini", "Taurus"] }, [], "chara"]);
  // Vimshottari'ye dönünce onun (kapalı) ağacı gelir; chara dashaya dönünce açık dal yerindedir.
  await system.selectOption("vimshottari");
  await expect(tree.getByRole("treeitem")).toHaveCount(9);
  await system.selectOption("chara");
  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ, / })).toHaveAttribute("aria-expanded", "true");
  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ–Koç, / })).toBeVisible();

  // Dasa şeridi: güncel dönem Rust'tan gelir (`chara_dasha_at`); tıklama transit anını taşır.
  await page.keyboard.press("Meta+Digit3");
  const strip = page.getByRole("region", { name: "Dasa şeridi" });
  await expect(strip.getByRole("list").getByRole("button")).toHaveCount(9);
  expect(await ipcCalls(page, "chara_dasha_at")).toHaveLength(0);
  await strip.getByRole("combobox", { name: "Sistem" }).selectOption("chara");
  const items = strip.getByRole("list").getByRole("button");
  await expect(items).toHaveCount(12);
  await expect(items.first()).toContainText("Bal–Kov");
  const current = strip.locator('[aria-current="true"]');
  await expect(current).toHaveCount(1);
  await expect(current).toHaveAttribute("data-sign", "Virgo");
  await expect(current).toContainText("transit anında");
  const asked = (await ipcCalls(page, "chara_dasha_at")) as { jdUt: number; input: { time_known: boolean } }[];
  expect(asked).toHaveLength(1);
  expect(asked[0].jdUt).toBeCloseTo(2461369.9166666665, 6);
  await strip.getByRole("combobox", { name: "Seviye" }).selectOption("maha");
  await expect(items).toHaveCount(20);
  await expect(strip.locator('[aria-current="true"]')).toHaveAttribute("data-sign", "Pisces");
  await items.first().click();
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain('"Jd":2451545');
  // The strip stays and the chosen period keeps the focus (the mocked transit view keeps its
  // moment, so the path is not asked again here; CharaDashaStrip.test.tsx covers the re-request).
  await items.nth(1).focus();
  await page.keyboard.press("Enter");
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain('"Jd":2453736.5');
  await expect(items).toHaveCount(20);
  await expect(items.nth(1)).toBeFocused();
  await expect(strip.getByRole("list")).not.toHaveAttribute("aria-busy");
});

test("shadbala and bhava bala: two palette panels on one command; units, items view and the shared highlight", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async (query: string, name: RegExp) => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill(query);
    await palette.getByRole("option", { name }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type ShadbalaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };
  const calls = async () => (await ipcCalls(page, "compute_shadbala")) as ShadbalaCall[];
  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await toggle("shadbala", /Shadbala panelini aç\/kapat/);
  const shadbala = page.getByRole("region", { name: "Shadbala" });
  await expect(shadbala).toBeFocused();
  await expect(shadbala).toBeInViewport();
  const summary = shadbala.getByRole("table", { name: /^Shadbala: yedi grahanın altı gücü/ });
  await expect(summary).toBeVisible();
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
  // Jüpiter en güçlü grahadır (sıra 1): toplam 533,1 virupa, oran 1,37.
  const jupiter = summary.locator('tbody tr[data-graha="Jupiter"]');
  await expect(jupiter.locator("td")).toHaveText(["233.8", "35.0", "185.8", "36.4", "34.3", "7.8", "533.1", "390.0", "1.37", "1"]);
  await expect(shadbala.getByRole("list", { name: "Zaman lordları" }).getByRole("listitem")).toHaveText([
    "Yıl lordu: Merkür",
    "Ay lordu: Jüpiter",
    "Gün lordu: Satürn",
    "Saat lordu: Güneş",
    "Tribhaga lordu: Satürn",
  ]);
  // Birim ve ayrıntı: yeniden hesap istenmez.
  await shadbala.getByRole("combobox", { name: "Birim" }).selectOption("rupa");
  await expect(jupiter.locator("td").nth(6)).toHaveText("8.89");
  await shadbala.getByRole("combobox", { name: "Ayrıntı" }).selectOption("items");
  await expect(shadbala.getByRole("table", { name: "Sthana bala kalemleri" })).toBeVisible();
  // Kalem tablolarında odaklanabilir denetim yoktur: kaydırma kabı klavye odağı alır.
  const sthanaScroll = shadbala.getByRole("group", { name: "Sthana bala kalemleri" });
  await sthanaScroll.focus();
  await expect(sthanaScroll).toBeFocused();
  await expect(shadbala.getByRole("table", { name: "Zaman gücü kalemleri" }).locator('tbody tr[data-graha="Saturn"] td').nth(5)).toHaveText("0.75");
  expect(await calls()).toHaveLength(1);

  // Bhava bala kendi isteğini yapar (her panel bir kez): aynı girdi, aynı rapor.
  await toggle("bhava", /Bhava bala panelini aç\/kapat/);
  const bhava = page.getByRole("region", { name: "Bhava bala" });
  await expect(bhava).toBeFocused();
  const seventh = bhava.locator('tbody tr[data-house="7"]');
  await expect(seventh.locator("td")).toHaveText(["Jüpiter", "533.1", "30.0", "55.9", "619.0", "1"]);
  expect(await calls()).toHaveLength(2);
  expect((await calls())[1].input).toEqual((await calls())[0].input);

  // Ortak vurgu: Bhava bala'daki lord, Shadbala satırını ve haritadaki grahayı vurgular.
  await seventh.getByRole("button", { name: "Jüpiter" }).hover();
  await expect(jupiter).toHaveClass(/is-highlighted/);
  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(bhava.locator('tbody tr[data-house="10"]')).toHaveClass(/is-highlighted/);

  // Paneller ve ayarları düzenle birlikte kaydedilir.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "shadbala" || p.kind === "bhava_bala")
        .map((p) => [p.kind, p.settings]);
    })
    .toEqual([
      ["shadbala", { unit: "rupa", detail: "items" }],
      ["bhava_bala", {}],
    ]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez; diğer panel yeniden istemez.
  await toggle("shadbala", /Shadbala panelini aç\/kapat/);
  await expect(shadbala).toHaveCount(0);
  await expect(bhava).toBeVisible();
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
  expect(await calls()).toHaveLength(2);
});

test("yogas: a palette panel with the grouped list; the group filter and the shared highlight", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("yoga");
    await palette.getByRole("option", { name: /Yogalar panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type YogasCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };
  const calls = async () => (await ipcCalls(page, "compute_yogas")) as YogasCall[];
  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await toggle();
  const yogas = page.getByRole("region", { name: "Yogalar" });
  await expect(yogas).toBeFocused();
  await expect(yogas).toBeInViewport();
  await expect(yogas.getByRole("table")).toHaveCount(9);
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
  // Delhi fikstürü: 9. ve 10. lord (Satürn ve Jüpiter) Koç'ta birlikte.
  const raja = yogas.getByRole("table", { name: "Raja yogaları" });
  const dharma = raja.locator('tbody tr[data-yoga="raja.dharma_karmadhipati"]');
  await expect(dharma.locator("th, td")).toHaveText(["Dharma-Karmadhipati", "Jü Sa", "9 10", "birlikte", "—"]);
  await expect(yogas.getByRole("table", { name: "Dosha" }).locator("tbody tr").locator("th, td")).toHaveText(["Kuja dosha", "Ma", "9", "—", "Venüs'ten"]);
  await expect(yogas.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeVisible();
  // İlişki ve koşul sütunlarında denetim yoktur: kaydırma kabı klavye odağı alır.
  const list = yogas.getByRole("group", { name: "Oluşan yogalar" });
  await list.focus();
  await expect(list).toBeFocused();

  // Ortak vurgu: satırdaki graha haritadaki grahayı ve onu içeren bütün yoga satırlarını vurgular.
  await dharma.getByRole("button", { name: "Jüpiter" }).hover();
  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(yogas.locator("tr.is-highlighted")).toHaveCount(4);
  await expect(yogas.locator('tr[data-yoga="lunar.gajakesari"]')).toHaveClass(/is-highlighted/);

  // Grup süzgeci listeyi daraltır; yeniden hesap istenmez ve ayar düzenle birlikte kaydedilir.
  await yogas.getByRole("combobox", { name: "Grup" }).selectOption("Raja");
  await expect(yogas.getByRole("table")).toHaveCount(1);
  await expect(raja.locator("tbody tr")).toHaveCount(2);
  await yogas.getByRole("combobox", { name: "Grup" }).selectOption("NabhasaAshraya");
  await expect(yogas.getByText("Bu grupta oluşan yoga yok: Nabhasa: ashraya yogaları.")).toBeVisible();
  expect(await calls()).toHaveLength(1);
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "yogas")
        .map((p) => p.settings);
    })
    .toEqual([{ group: "NabhasaAshraya" }]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await toggle();
  await expect(yogas).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("yogas with an unknown birth time: lagna rules are skipped and the rest is marked approximate", async ({ page }) => {
  await mockIpc(page, { session: true, unknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("yoga");
  await palette.getByRole("option", { name: /Yogalar panelini aç\/kapat/ }).click();
  const yogas = page.getByRole("region", { name: "Yogalar" });
  await expect(yogas.getByRole("table")).toHaveCount(4);
  const calls = (await ipcCalls(page, "compute_yogas")) as { input: { time_known: boolean } }[];
  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
  await expect(yogas.getByText(/^Doğum saati bilinmiyor: lagnaya bağlı 17 kural değerlendirilmedi\./)).toBeVisible();
  await expect(yogas.getByText("Değerlendirilen kural: 24. Oluşan kayıt: 6.")).toBeVisible();
  // Her satır "≈" işaretini taşır; evler çizgidir.
  await expect(yogas.locator("tbody tr")).toHaveCount(6);
  await expect(yogas.locator('tbody tr th span[aria-hidden="true"]')).toHaveText(["≈", "≈", "≈", "≈", "≈", "≈"]);
  await expect(yogas.locator('tr[data-yoga="dosha.kuja"]').locator("th, td")).toHaveText(["≈yaklaşık Kuja dosha", "Ma", "—", "—", "Venüs'ten"]);
});

test("compat: a palette panel; a partner from the picker, the kuta table, swapped roles and the partner ring", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("uyum");
    await palette.getByRole("option", { name: /Uyum panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type Input = { jd_ut: number; time_known: boolean };
  const compatCalls = async () => (await ipcCalls(page, "compute_compat")) as { groom: Input; bride: Input }[];
  type Saved = { viewState: { compat_partner_uuid: string | null; compat_roles: string; panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } };
  const lastSaved = async () => (await ipcCalls(page, "save_view_state")).at(-1) as Saved | undefined;
  const PARTNER_CHART = "00000000-0000-4000-8000-000000000004";

  await toggle();
  const compat = page.getByRole("region", { name: "Uyum" });
  await expect(compat).toBeFocused();
  await expect(compat).toBeInViewport();
  // Eş seçilmemiş: tek eylemli boş durum; hiçbir uyum komutu çağrılmaz (ileri modüller spec'i K1).
  await expect(compat.getByText("Uyum için bir eş harita seçin.")).toBeVisible();
  await expect(compat.getByRole("button")).toHaveText(["Eş harita seç"]);
  for (const cmd of ["compute_compat", "chart_input", "chart_brief"]) expect(await ipcCalls(page, cmd)).toHaveLength(0);

  // Eş seçici: klavyeyle arama; tek haritalı danışan doğrudan eş olur.
  await compat.getByRole("button", { name: "Eş harita seç" }).click();
  const picker = page.getByRole("dialog", { name: "Eş harita seç" });
  await picker.getByRole("combobox", { name: "Danışan ara" }).fill("ayşe");
  await expect(picker.getByRole("option")).toHaveText(["Ayşe Yılmaz"]);
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toHaveCount(0);

  const scores = compat.getByRole("table", { name: "Ashtakuta puanları" });
  await expect(scores.locator("tbody tr")).toHaveCount(8);
  await expect(scores.locator('tr[data-kuta="Tara"]').locator("th, td")).toHaveText(["Tara", "2 · Sampat", "9 · Parama mitra", "3 / 3"]);
  await expect(scores.locator("tfoot tr").locator("th, td")).toHaveText(["Toplam", "19 / 36"]);
  await expect(compat.locator(".compat-people div")).toHaveText(["1. kişi (damat rolü)Örnek Danışan · Doğum haritası", "2. kişi (gelin rolü)Ayşe Yılmaz · Doğum haritası"]);
  await expect(compat.locator(".compat-facts li")).toHaveCount(3);
  await expect(compat.getByRole("table", { name: "Karşılıklı yerleşim" }).locator("tbody tr")).toHaveCount(9);
  // Eşin girdisi kimlikle okunur; rapor bir kez, damat rolü açık harita olarak istenir.
  expect(await ipcCalls(page, "chart_input")).toEqual([{ chartUuid: PARTNER_CHART }]);
  expect(await ipcCalls(page, "chart_brief")).toEqual([{ chartUuid: PARTNER_CHART }]);
  const first = await compatCalls();
  expect(first).toHaveLength(1);
  expect(first[0].groom).toMatchObject({ jd_ut: 2451545, time_known: true });
  expect(first[0].bride).toMatchObject({ jd_ut: FIXTURE.compat.input.jd_ut, time_known: true });
  // Eş bağı haritanın görünüm kaydına yazılır.
  await expect.poll(async () => [(await lastSaved())?.viewState.compat_partner_uuid, (await lastSaved())?.viewState.compat_roles]).toEqual([PARTNER_CHART, "OpenIsGroom"]);

  // Ortak vurgu: yerleşim tablosundaki graha haritadaki grahayı vurgular.
  await compat.getByRole("table", { name: "Karşılıklı yerleşim" }).getByRole("button", { name: "Jüpiter" }).click();
  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(compat.locator('tr[data-graha="Jupiter"]')).toHaveClass(/is-highlighted/);
  // Esc sabitlemeyi kaldırır: satır ve haritadaki graha vurgusuz kalır.
  await page.keyboard.press("Escape");
  await expect(compat.locator('tr[data-graha="Jupiter"]')).not.toHaveClass(/is-highlighted/);
  await expect(page.locator(".chart-planet--highlight")).toHaveCount(0);

  // Roller değişir: rapor iki girdi yer değiştirerek yeniden istenir; eş yeniden okunmaz.
  await compat.getByRole("button", { name: "Rolleri değiştir" }).click();
  await expect(scores.locator("tfoot tr").locator("th, td")).toHaveText(["Toplam", "21 / 36"]);
  await expect(compat.locator(".compat-people div").first()).toHaveText("1. kişi (damat rolü)Ayşe Yılmaz · Doğum haritası");
  const second = await compatCalls();
  expect(second).toHaveLength(2);
  expect(second[1].groom.jd_ut).toBe(FIXTURE.compat.input.jd_ut);
  expect(await ipcCalls(page, "chart_input")).toHaveLength(1);
  await expect.poll(async () => (await lastSaved())?.viewState.compat_roles).toBe("OpenIsBride");

  // Harita panelinde eş halkası: eşin dokuz grahası D1'in dış halkasına yazılır.
  const d1 = page.getByRole("region", { name: /^Harita/ }).first();
  await d1.getByRole("checkbox", { name: "Eş halkası" }).check();
  await expect(d1.locator(".chart-transit")).toHaveCount(9);
  await expect(d1.getByText(/^Eş \(Ayşe Yılmaz\): /)).toHaveCount(1);
  // İki halka aynı anda açık olamaz.
  await d1.getByRole("checkbox", { name: "Transit halkası" }).check();
  await expect(d1.getByRole("checkbox", { name: "Eş halkası" })).not.toBeChecked();
  await expect(d1.getByText(/^Eş \(/)).toHaveCount(0);

  // Panel düzenle birlikte kaydedilir; aynı komut paneli kaldırır ve odak gövdeye düşmez.
  await expect
    .poll(async () => {
      const layout = (await lastSaved())?.viewState.panel_layout;
      return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.map((p) => p.kind);
    })
    .toContain("compat");
  await toggle();
  await expect(compat).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("compat with a partner whose birth time is unknown: the report is marked approximate and the Moon changes are named", async ({ page }) => {
  await mockIpc(page, { session: true, partnerUnknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("uyum");
  await palette.getByRole("option", { name: /Uyum panelini aç\/kapat/ }).click();
  const compat = page.getByRole("region", { name: "Uyum" });
  await compat.getByRole("button", { name: "Eş harita seç" }).click();
  // Boş aramada son açılan danışanlar listelenir; fareyle seçim.
  await page.getByRole("dialog", { name: "Eş harita seç" }).getByRole("option", { name: "Ayşe Yılmaz" }).click();
  await expect(compat.getByRole("table", { name: "Ashtakuta puanları" })).toBeVisible();
  // Girdi `time_known: false` ile gider: lagnaya bağlı değerleri Rust boş döner (K2).
  const calls = (await ipcCalls(page, "compute_compat")) as { bride: { time_known: boolean } }[];
  expect(calls.map((c) => c.bride.time_known)).toEqual([false]);
  await expect(compat.locator(".compat-approx")).toHaveText(/^≈yaklaşık Doğum saati bilinmeyen haritada Ay öğle konumundadır/);
  await expect(compat.locator(".compat-people div").nth(1)).toHaveText("2. kişi (gelin rolü)≈yaklaşık Ayşe Yılmaz · Doğum haritası");
  await expect(compat.locator(".compat-warning")).toHaveText(/^Ayşe Yılmaz: Ay o gün nakshatra değiştiriyor;.*Ayşe Yılmaz: Ay o gün burç değiştiriyor;/);
  // Eşin lagnası yok: açık haritanın grahaları ondan sayılamaz.
  const sun = compat.getByRole("table", { name: "Karşılıklı yerleşim" }).locator('tr[data-graha="Sun"]');
  await expect(sun.locator("th, td")).toHaveText(["Gü", "lagnadan — · Ay'dan 4", "lagnadan 10 · Ay'dan 6"]);
  await expect(compat.locator('.compat-facts li[data-fact="kuja-Bride"]')).toHaveText("Kuja dosha, Ayşe Yılmaz: lagnadan bilinmiyor, Ay'dan var, Venüs'ten var.");
});

test("muhurta day: a palette panel; the current place from the dialog, the day strip, the limbs and the hora table", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("gün panchang");
    await palette.getByRole("option", { name: /Gün panchangı panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type DayCall = { at: unknown; place: { lat: number; lon: number; tz: unknown }; natal: unknown };
  const dayCalls = async () => (await ipcCalls(page, "day_panchang")) as DayCall[];
  type Saved = { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } };
  const savedPanel = async () => {
    const layout = ((await ipcCalls(page, "save_view_state")).at(-1) as Saved | undefined)?.viewState.panel_layout;
    return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.find((p) => p.kind === "muhurta_day");
  };

  await toggle();
  const day = page.getByRole("region", { name: "Gün panchangı" });
  await expect(day).toBeFocused();
  await expect(day).toBeInViewport();
  // Bulunulan yer seçilmemiş: panel yer ister, gün hesaplanmaz (ileri modüller spec'i K1).
  await expect(day.getByRole("note")).toContainText("Bulunulan yer seçilmedi.");
  expect(await dayCalls()).toHaveLength(0);

  // Bulunulan yer penceresi: açık haritanın doğum yeri önerilir; seçim tercihlere yazılır.
  // Klavyeyle açılır (WebKit tıklanan düğmeye odak vermez): pencere kapanınca odak düğmeye döner.
  await day.getByRole("button", { name: "Bulunulan yeri seç" }).focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", { name: "Bulunulan yer" });
  const useBirthPlace = dialog.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ });
  await expect(useBirthPlace).toBeVisible();
  // Pencere açıkken de, yer seçilene dek gün istenmez.
  expect(await dayCalls()).toHaveLength(0);
  await useBirthPlace.click();
  await expect(dialog.locator(".home-place-current")).toHaveText("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
  // Tercihlerin tamamı yazılır (kayıt bütün kaydı değiştirir): diğer tercihler yerinde durur.
  const prefs = (await ipcCalls(page, "set_app_prefs")).at(-1) as { prefs: unknown };
  expect(prefs.prefs).toEqual({ ...FIXTURE.prefs.prefs, home_place: FIXTURE.muhurta.home_place });
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(day.getByRole("button", { name: "Bulunulan yeri değiştir" })).toBeFocused();

  // Gün: şerit, uzuvlar, kişiye bağlı etkenler ve hora tablosu; komuta yerin adı gitmez.
  await expect(day.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeVisible();
  const strip = day.getByRole("list", { name: "Gündüzün sekiz dilimi" });
  await expect(strip.getByRole("listitem")).toHaveCount(8);
  await expect(strip.locator("li.is-marked")).toHaveText(["2Yama08:13–09:31", "4Guli10:49–12:07", "5Rahu12:07–13:26"]);
  await expect(day.getByText(/^Abhijit 11:47–12:28/)).toBeVisible();
  await expect(day.getByRole("table", { name: "Beş uzuv ve değişim saatleri" }).locator("tbody tr")).toHaveCount(12);
  await expect(day.getByRole("table", { name: "Açık haritanın Ay'ına göre" }).locator("tbody tr")).toHaveCount(3);
  const horas = day.getByRole("table", { name: "Hora tablosu" });
  await expect(horas.locator("tbody tr")).toHaveCount(24);
  const calls = await dayCalls();
  expect(calls).toHaveLength(1);
  expect(calls[0].at).toBe("Now");
  expect(calls[0].place).toEqual(FIXTURE.muhurta.place);
  expect(calls[0].natal).toEqual(FIXTURE.muhurta.natal);

  // Ortak vurgu: hora lordu haritadaki grahayı vurgular.
  await horas.getByRole("button", { name: "Jüpiter" }).first().click();
  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(horas.locator("tr.is-highlighted")).toHaveCount(3);
  await page.keyboard.press("Escape");

  // Yer seçimi ve tarih bağı panelin ayarıdır: düzenle birlikte kaydedilir.
  await day.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await day.getByRole("combobox", { name: "Tarih bağı" }).selectOption("today");
  await expect(day.getByRole("form", { name: "Transit anı" })).toHaveCount(0);
  await expect.poll(async () => (await savedPanel())?.settings).toEqual({ place: "chart", link: "today" });

  // Aynı komut paneli kaldırır ve odak gövdeye düşmez.
  await toggle();
  await expect(day).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("muhurta search: the filter form, paged windows, a saved filter, and a row that moves the day panel to that moment", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const open = async (query: string, name: RegExp) => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill(query);
    await palette.getByRole("option", { name }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type SearchCall = { query: { from_date: string | null; to_date: string | null; place: unknown; criteria: Record<string, unknown>; natal: unknown }; cursor: { cursor: { chunk: number } } | null };
  const searchCalls = async () => (await ipcCalls(page, "muhurta_search")) as SearchCall[];

  await open("gün panchang", /Gün panchangı panelini aç\/kapat/);
  const day = page.getByRole("region", { name: "Gün panchangı" });
  await day.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await expect(day.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeVisible();
  await open("muhurta ara", /Muhurta araması panelini aç\/kapat/);
  const search = page.getByRole("region", { name: "Muhurta araması" });
  await expect(search).toBeFocused();
  // Panel açılınca arama yapılmaz; kayıtlı süzgeçler bir kez okunur.
  expect(await searchCalls()).toHaveLength(0);
  expect(await ipcCalls(page, "load_muhurta_filters")).toHaveLength(1);
  await expect(search.getByText(/^Süzgeci kurup “Ara”ya basın\./)).toBeVisible();

  // Süzgeç: yer, gün, gündüz, dışlanan dilim; koşullar görünür etiketli kutulardır.
  await search.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await search.getByRole("checkbox", { name: "Perşembe" }).check();
  await search.getByRole("checkbox", { name: "Yalnızca gündüz" }).check();
  await search.getByRole("checkbox", { name: "Rahu dilimi" }).check();
  await search.getByText("Tithiler · kısıt yok").click();
  await search.getByRole("checkbox", { name: "S2 Dwitiya" }).check();
  await expect(search.getByText("Tithiler · 1 seçili")).toBeVisible();
  await search.getByRole("checkbox", { name: "S2 Dwitiya" }).uncheck();

  await search.getByRole("button", { name: "Ara" }).click();
  await expect(search.locator("[data-summary]")).toHaveText("30 pencere · 2026-11-26 – 2027-01-31");
  const results = search.getByRole("table", { name: "Bulunan pencereler" });
  await expect(results.locator("tbody tr")).toHaveCount(30);
  await expect(results.locator("tbody tr").first().locator("th, td")).toHaveText(["2026-11-26 08:14", "2026-11-26 09:32", "1 sa 17 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Akr"]);
  // Üç dilim, her biri bir öncekinin imleciyle; tarihler boş (bugünden başlar), yerin adı gitmez.
  const calls = await searchCalls();
  expect(calls.map((c) => c.cursor?.cursor.chunk ?? null)).toEqual([null, 1, 2]);
  expect(calls[0].query).toMatchObject({ from_date: null, to_date: null, place: FIXTURE.muhurta.place, natal: FIXTURE.muhurta.natal });
  expect(calls[0].query.criteria).toMatchObject({ weekdays: ["Thursday"], daytime_only: true, exclude: ["RahuSegment"], tithis: [] });

  // Süzgeç adlandırılıp kaydedilir: kayıt yalnızca ad ve koşul taşır.
  await search.getByRole("textbox", { name: "Süzgeç adı" }).fill("Perşembe gündüzü");
  await search.getByRole("button", { name: "Süzgeci kaydet" }).click();
  await expect(search.getByRole("combobox", { name: "Kayıtlı süzgeç" })).toHaveValue("Perşembe gündüzü");
  const savedFilters = (await ipcCalls(page, "save_muhurta_filters")).at(-1) as { filters: Record<string, unknown>[] };
  expect(savedFilters.filters).toHaveLength(1);
  expect(Object.keys(savedFilters.filters[0]).sort()).toEqual(["criteria", "name"]);

  // Satırın başlangıcı transit tarihini o ana götürür: gün paneli o anı ister.
  const before = (await ipcCalls(page, "day_panchang")).length;
  const first = results.locator("tbody tr").first().getByRole("button");
  await first.focus();
  await page.keyboard.press("Enter");
  await expect.poll(async () => (await ipcCalls(page, "day_panchang")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "day_panchang")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: FIXTURE.muhurta.pages[0].windows[0].start_jd_ut });
  // Sonuç listesi yerinde durur ve odak satırdadır.
  await expect(results.locator("tbody tr")).toHaveCount(30);
  await expect(first).toBeFocused();
});

test("prashna: Cmd+Shift+P casts a number prashna for the open client; the Prashna layout, the facts, the focus house and the question", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Shift+p");
  const sheet = page.getByRole("dialog", { name: "Yeni prashna" });
  await expect(sheet.getByRole("radio", { name: "Şimdi" })).toBeFocused();
  // Bulunulan yer seçilmemiş: form yer ister, haritanın yerine kendiliğinden düşmez ve komut çağırmaz.
  await expect(sheet.locator(".prashna-place")).toContainText("Bulunulan yer seçilmemiş.");
  await sheet.getByRole("button", { name: "Prashna kur" }).click();
  await expect(sheet.getByRole("combobox", { name: "Yer" })).toBeFocused();
  await expect(sheet.getByText("Bir yer seçin.")).toBeVisible();
  expect(await ipcCalls(page, "create_prashna")).toHaveLength(0);
  await sheet.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await expect(sheet.locator(".prashna-place")).toHaveText("New Delhi, Delhi, India · Asia/Kolkata");
  await sheet.getByRole("textbox", { name: "Sayı (isteğe bağlı)" }).fill("54");
  await sheet.getByRole("textbox", { name: "Soru" }).fill("Deneme sorusu");
  await sheet.getByRole("combobox", { name: "Odak evi" }).selectOption("7");
  await sheet.getByRole("button", { name: "Prashna kur" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // İstek: an "şimdi" (Rust okur), yer haritanın yeri, sayı ve soru yazıldığı gibi; açık danışana.
  type Request = { request: { client_id: number | null; at: unknown; place: { place: { name: string }; tz: unknown }; number: number | null; question: string; focus_house: number | null } };
  const [cast] = (await ipcCalls(page, "create_prashna")) as Request[];
  expect(cast.request).toMatchObject({ client_id: 1, at: "Now", number: 54, question: "Deneme sorusu", focus_house: 7 });
  expect(cast.request.place).toEqual(FIXTURE.muhurta.home_place);

  // Harita açılır ve Prashna düzenine geçilir: D1, olgular, panchang ve notlar.
  await expect(page.getByRole("combobox", { name: "Düzen" })).toHaveValue("builtin:prashna");
  await expect(page.locator(".topbar-chart").first()).toHaveText("Prashna 2026-11-25 15:30");
  await expect(page.locator(".topbar-number")).toContainText("sayı: 54");
  const facts = page.getByRole("region", { name: "Prashna olguları" });
  await expect(page.getByRole("region", { name: "Panchang", exact: true })).toBeVisible();
  await expect(page.getByRole("region", { name: "Notlar" })).toBeVisible();
  await expect(facts.getByRole("textbox", { name: "Soru" })).toHaveValue("Deneme sorusu");
  // Lagna sayıdan gelir (54 → Başak 28°20′, navamsa ortası); modül girdisi sayıyı taşır.
  await expect(facts.getByRole("region", { name: "Lagna" })).toContainText("Baş 28°20′ · 54. navamsanın ortası");
  await expect(facts.getByRole("region", { name: "Lagna" })).toContainText("Shirshodaya · başla doğan");
  type FactsCall = { input: { jd_ut: number; time_known: boolean; lagna_number: number | null }; focusHouse: number | null };
  const factCalls = async () => (await ipcCalls(page, "compute_prashna_facts")) as FactsCall[];
  expect(await factCalls()).toHaveLength(1);
  expect((await factCalls())[0]).toMatchObject({ input: { jd_ut: FIXTURE.prashna.opened_number.chart.jd_ut, time_known: true, lagna_number: 54 }, focusHouse: 7 });
  const lords = facts.getByRole("table", { name: "Lagna lordu ve odak evinin lordu" });
  await expect(lords.locator("tbody tr")).toHaveCount(2);
  await expect(lords.locator("tbody tr").first().locator("th, td").first()).toHaveText("Lagna lordu");
  await expect(facts.getByRole("region", { name: "Ay" })).toContainText("Rohini");
  await expect(facts.getByText("Bu panel olguları sayar; sorunun yanıtını ya da bir hükmü söylemez.")).toBeVisible();

  // Odak evi panelden değişir: kaydedilir, olgular yeniden okunur, odak seçicide durur.
  const focus = facts.getByRole("combobox", { name: "Odak evi" });
  await focus.focus();
  await focus.selectOption("10");
  await expect.poll(async () => (await factCalls()).length).toBe(2);
  expect((await factCalls())[1].focusHouse).toBe(10);
  await expect(facts.locator('tr[data-focus="true"] th')).toHaveText("10. ev · odak");
  await expect(focus).toBeFocused();
  // Soru yazdıkça kaydedilir; olgular yeniden hesaplanmaz.
  await facts.getByRole("textbox", { name: "Soru" }).fill("Deneme sorusu, ikinci yazım");
  await expect(facts.getByText("Kaydedildi")).toBeVisible();
  type Edit = { chartId: number; edit: { question: string; focus_house: number | null } };
  const edits = (await ipcCalls(page, "update_prashna")) as Edit[];
  expect(edits.at(-1)).toEqual({ chartId: FIXTURE.prashna.opened_number.chart_record.id, edit: { question: "Deneme sorusu, ikinci yazım", focus_house: 10 } });
  expect(await factCalls()).toHaveLength(2);
  // Lagna lordu ortak vurgudur: haritadaki graha da işaretlenir.
  await lords.getByRole("button", { name: "Merkür" }).click();
  await expect(lords.locator('tr[data-lord="lagna"]')).toHaveClass(/is-highlighted/);
});

test("prashna without an open client: the palette command, a searched place, a typed moment; the chart opens under the built-in client", async ({ page }) => {
  await mockIpc(page, { session: false });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("prashna");
  await palette.getByRole("option", { name: /Yeni prashna/ }).click();
  const sheet = page.getByRole("dialog", { name: "Yeni prashna" });
  // Açık danışan yok: danışan kutusu ve "açık haritanın yeri" seçeneği yoktur.
  await expect(sheet.getByRole("checkbox")).toHaveCount(0);
  await expect(sheet.getByText(/Açık danışan yok: prashna “Prashna” adlı yerleşik danışanın altında toplanır\./)).toBeVisible();
  await expect(sheet.getByRole("combobox", { name: "Yer" }).locator("option")).toHaveText(["Bulunulan yer", "Başka bir yer ara"]);
  await sheet.getByRole("combobox", { name: "Yer" }).selectOption("search");
  await sheet.getByRole("combobox", { name: "Yer ara" }).fill("new delhi");
  await expect(sheet.getByRole("listbox").getByRole("option").first()).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(sheet.locator(".prashna-place")).toContainText("Asia/Kolkata");
  // Girilen an sorunun yerinin duvar saatidir; çözümü Rust yapar.
  await sheet.getByRole("radio", { name: "Başka bir an" }).check();
  await sheet.getByLabel("Tarih").fill("2026-11-25");
  await sheet.getByLabel("Saat").fill("15:30");
  await sheet.getByRole("button", { name: "Prashna kur" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const [cast] = (await ipcCalls(page, "create_prashna")) as { request: { client_id: number | null; at: unknown; number: number | null; focus_house: number | null } }[];
  expect(cast.request).toMatchObject({ client_id: null, at: { Local: { date: "2026-11-25", time: "15:30" } }, number: null, focus_house: null });

  // Yerleşik danışan açılır; zaman prashnasında sayı rozeti yoktur ve odak evi seçilmemiştir.
  await expect(page.getByRole("heading", { level: 1, name: "Prashna" })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Düzen" })).toHaveValue("builtin:prashna");
  await expect(page.locator(".topbar-number")).toHaveCount(0);
  const facts = page.getByRole("region", { name: "Prashna olguları" });
  await expect(facts.getByRole("combobox", { name: "Odak evi" })).toHaveValue("");
  await expect(facts.getByText("Odak evi seçilmedi: odak evinin lordu ve iki lordun bağı gösterilmez.")).toBeVisible();
  const calls = (await ipcCalls(page, "compute_prashna_facts")) as { input: { lagna_number: number | null }; focusHouse: number | null }[];
  expect(calls).toEqual([expect.objectContaining({ focusHouse: null })]);
  expect(calls[0].input.lagna_number).toBeNull();
  // Olgular paneli paletten kapatılıp açılabilir (K12); kapatınca odak gövdeye düşmez.
  await page.keyboard.press("Meta+k");
  await page.getByRole("dialog", { name: "Komut paleti" }).getByRole("combobox").fill("prashna olgu");
  await page.getByRole("dialog", { name: "Komut paleti" }).getByRole("option", { name: /Prashna olguları panelini aç\/kapat/ }).click();
  await expect(page.getByRole("region", { name: "Prashna olguları" })).toHaveCount(0);
  await expect(page.locator("body")).not.toBeFocused();
});

test("chart search: Cmd+Shift+F, an example condition, the result list, a saved search, tagging the results and opening a chart", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  type SearchCall = { query: { all_of: { cond: Record<string, unknown>; negate: boolean }[]; scope: { tags: string[]; kind: string | null; exclude_prashna: boolean } }; searchId: number; cursor: unknown };
  const searchCalls = async () => (await ipcCalls(page, "chart_search")) as SearchCall[];

  await page.keyboard.press("Meta+Shift+f");
  const sheet = page.getByRole("dialog", { name: "Haritalarda ara" });
  // Koşul yokken tek eylem örnek koşuldur ve arama nedenini söyleyerek kullanılamaz.
  await expect(sheet.getByRole("button", { name: "Örnek koşul ekle" })).toBeFocused();
  await expect(sheet.getByRole("button", { name: "Ara", exact: true })).toHaveAttribute("aria-disabled", "true");
  expect(await ipcCalls(page, "list_saved_searches")).toHaveLength(1);
  await page.keyboard.press("Enter");
  const row = sheet.getByRole("group", { name: "Koşul 1" });
  await expect(row.getByLabel("Tür")).toHaveValue("GrahaInSign");
  // Koşul klavyeyle kurulur: Ay Terazi'de; ikinci koşul "değil" ile.
  await row.getByText("Burçlar · 1 seçili").click();
  await row.getByRole("checkbox", { name: "Yen" }).uncheck();
  await row.getByRole("checkbox", { name: "Ter" }).check();
  await sheet.getByRole("button", { name: "Koşul ekle" }).click();
  const second = sheet.getByRole("group", { name: "Koşul 2" });
  await second.getByLabel("Tür").selectOption("GrahaInHouse");
  await expect(sheet.getByRole("button", { name: "Ara", exact: true })).toHaveAttribute("aria-disabled", "true");
  await second.getByText("Evler · seçilmedi").click();
  await second.getByRole("checkbox", { name: "1", exact: true }).check();
  await second.getByRole("checkbox", { name: "10", exact: true }).check();
  await second.getByRole("checkbox", { name: "değil" }).check();
  expect(await searchCalls()).toHaveLength(0);

  await sheet.getByRole("button", { name: "Ara", exact: true }).click();
  await expect(sheet.locator("[data-summary]")).toHaveText("1 eşleşme / 2 harita tarandı");
  const hit = sheet.getByRole("button", { name: "Örnek Danışan, etiketsiz harita: haritayı aç" });
  await expect(hit).toContainText("Ay Ter 5. ev; değil: Gü 1/10. ev");
  const calls = await searchCalls();
  expect(calls).toHaveLength(1);
  expect(calls[0].cursor).toBeNull();
  expect(calls[0].query).toEqual(FIXTURE.search.query);

  // Arama saklanır: kayıt yalnızca ad ve sorgu taşır.
  await sheet.getByRole("textbox", { name: "Arama adı" }).fill("Ay Terazi");
  await sheet.getByRole("button", { name: "Aramayı sakla" }).click();
  await expect(sheet.getByRole("combobox", { name: "Kayıtlı arama" }).locator("option:checked")).toHaveText("Ay Terazi");
  const saved = (await ipcCalls(page, "save_search")).at(-1) as { request: Record<string, unknown> };
  expect(Object.keys(saved.request).sort()).toEqual(["name", "query"]);

  // Sonuçları etiketle: onay etiketi ve harita sayısını söyler; odak "Vazgeç"te başlar.
  await sheet.getByRole("textbox", { name: "Sonuçları etiketle" }).fill("Ay Terazi");
  await sheet.getByRole("button", { name: "Etiketle", exact: true }).click();
  const confirm = page.getByRole("dialog", { name: "Sonuçları etiketle" });
  await expect(confirm).toContainText("“Ay Terazi” etiketi 1 haritaya eklenecek.");
  await expect(confirm.getByRole("button", { name: "Vazgeç" })).toBeFocused();
  expect(await ipcCalls(page, "tag_charts")).toHaveLength(0);
  await confirm.getByRole("button", { name: "1 haritayı etiketle" }).click();
  await expect(sheet.getByText("“Ay Terazi” etiketi 1 haritaya eklendi.")).toBeVisible();
  expect(await ipcCalls(page, "tag_charts")).toEqual([{ request: { chart_ids: [FIXTURE.search.page.hits[0].chart_id], tag: "Ay Terazi" } }]);

  // Kapsam değişince liste önceki aramaya aittir; yeni arama prashna haritalarını da tarar.
  await sheet.getByRole("checkbox", { name: "Prashna haritalarını dışla" }).uncheck();
  await expect(sheet.getByRole("note")).toContainText("Koşullar değişti; liste önceki aramaya ait.");
  await sheet.getByRole("button", { name: "Ara", exact: true }).click();
  await expect(sheet.locator("[data-summary]")).toHaveText(`${FIXTURE.search.page_all_charts.hits.length} eşleşme / 4 harita tarandı`);
  await expect(sheet.getByRole("note")).toHaveCount(0);

  // Enter sonucu açar: yüzey kapanır, harita açılır.
  await sheet.getByRole("button", { name: "Örnek Danışan, etiketsiz harita: haritayı aç" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(await ipcCalls(page, "open_client")).toEqual([{ clientId: 1, chartId: FIXTURE.search.page.hits[0].chart_id }]);
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();

  // Paletten yeniden açılınca koşullar yerindedir, sonuçlar değil; Esc kapatır ve odak gövdeye düşmez.
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("haritalarda");
  await palette.getByRole("option", { name: /Haritalarda ara/ }).click();
  await expect(sheet.getByRole("group", { name: /^Koşul \d$/ })).toHaveCount(2);
  await expect(sheet.getByText(/^Koşulları kurup “Ara”ya basın\./)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("event search: the panel from the palette, the event list in the chart's time zone, and a row that moves the transit date", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("olay ara");
    await palette.getByRole("option", { name: /Olay araması panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  type EventCall = { spec: { from_date: string | null; to_date: string | null; place: unknown; kinds: string[]; grahas: string[]; natal: unknown }; searchId: number };
  const eventCalls = async () => (await ipcCalls(page, "event_search")) as EventCall[];

  await toggle();
  const panel = page.getByRole("region", { name: "Olay araması" });
  await expect(panel).toBeFocused();
  // Panel açılınca arama yapılmaz; yer açık haritanın yeridir.
  expect(await eventCalls()).toHaveLength(0);
  await expect(panel.getByRole("combobox", { name: "Yer" })).toHaveValue("chart");
  await expect(panel.getByText(/^Türleri, grahaları ve aralığı seçip/)).toBeVisible();
  await panel.getByRole("checkbox", { name: "Yeni ay" }).check();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş" }).check();
  await panel.getByLabel("Başlangıç tarihi").fill("2026-11-26");
  await panel.getByLabel("Bitiş tarihi").fill("2027-03-31");
  await panel.getByRole("button", { name: "Ara", exact: true }).click();
  const total = FIXTURE.search.events.events.length;
  await expect(panel.locator("[data-summary]")).toHaveText(`${total} olay · 2026-11-26 – 2027-03-31`);
  const table = panel.getByRole("table", { name: "Olaylar" });
  await expect(table.locator("tbody tr")).toHaveCount(total);
  await expect(table.locator("tbody tr").first().locator("td")).toHaveText(["Yeni ay", "Akr · Jyeshtha"]);
  // Saat açık haritanın yerinin dilimindedir: 2026-12-09 yeni ayı 00:51 UT'dir; Delhi'de
  // (+05:30) 06:21.
  expect(FIXTURE.search.events.events[0].offset_secs).toBe(19_800);
  await expect(table.locator("tbody tr").first().getByRole("button")).toHaveText("2026-12-09 06:21");
  const calls = await eventCalls();
  expect(calls).toHaveLength(1);
  expect(calls[0].spec).toMatchObject({
    from_date: "2026-11-26",
    to_date: "2027-03-31",
    place: FIXTURE.muhurta.place,
    kinds: ["SignIngress", "StationRetrograde", "StationDirect", "NewMoon", "TransitToNatal"],
    grahas: ["Jupiter", "Saturn", "Rahu"],
  });
  // Doğum verisi yalnızca açık harita isteyen tür seçiliyken gider.
  expect(calls[0].spec.natal).toMatchObject({ jd_ut: FIXTURE.opened.chart.jd_ut, time_known: true });

  // Satırdaki an transit tarihini o ana götürür; liste ve odak yerinde durur.
  const before = (await ipcCalls(page, "transits_at")).length;
  const first = table.locator("tbody tr").first().getByRole("button");
  await first.focus();
  await page.keyboard.press("Enter");
  await expect.poll(async () => (await ipcCalls(page, "transits_at")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "transits_at")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: FIXTURE.search.events.events[0].event.jd_ut });
  await expect(table.locator("tbody tr")).toHaveCount(total);
  await expect(first).toBeFocused();

  // Panel aynı komutla kalkar; odak gövdeye düşmez.
  await toggle();
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("natal extras: panchang additions, navatara with the transit taras and vimshopaka bala from the palette", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async (query: string, name: RegExp) => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill(query);
    await palette.getByRole("option", { name }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  // Üç rapor da panel açılmadan istenmez (K1).
  for (const cmd of ["compute_panchang_extras", "compute_navatara", "compute_vimshopaka"]) expect(await ipcCalls(page, cmd)).toHaveLength(0);

  // Panchang paneli: beş uzvun altında panchang ekleri.
  await toggle("panchang", /Panchang panelini aç\/kapat/);
  const panchang = page.getByRole("region", { name: "Panchang", exact: true });
  const extras = panchang.getByRole("table", { name: "Panchang ekleri" });
  await expect(extras.locator('tr[data-extra="yogi_point"] td')).toHaveText("Ter 9°19′ · Swati · pada 1");
  await expect(extras.locator('tr[data-extra="yogi"] td')).toHaveText("Rahu");
  await expect(extras.locator('tr[data-extra="avayogi"] td')).toHaveText("Venüs · Purva Ashadha");
  await expect(extras.locator('tr[data-extra="dagdha"] td')).toHaveText("Yay, Bal");
  await expect(panchang.getByText(/^Doğrulanmamış tablo: dagdha rasi tablosu/)).toBeVisible();
  expect(await ipcCalls(page, "compute_panchang_extras")).toHaveLength(1);

  // Navatara: doğum tablosu ve transit anının taraları; an Rust'a transit deposundan gider.
  await toggle("navatara", /Navatara panelini aç\/kapat/);
  const navatara = page.getByRole("region", { name: "Navatara" });
  await expect(navatara).toBeFocused();
  const natal = navatara.getByRole("table", { name: /^Navatara: doğum nakshatrası Swati/ });
  await expect(natal.locator('tbody tr[data-tara="1"] td')).toHaveText(["Swati (Ay)", "Shatabhisha", "Ardra", "Rahu", "elverişli kümede değil"]);
  const transit = navatara.getByRole("table", { name: /^Transit grahalarının tarası,/ });
  await expect(transit.locator('tbody tr[data-graha="Moon"] td')).toHaveText(["Rohini · pada 4", "8 · Mitra", "2", "elverişli kümede"]);
  const navataraCalls = (await ipcCalls(page, "compute_navatara")) as { transitJdUt: number | null }[];
  expect(navataraCalls.at(-1)?.transitJdUt).toBe(FIXTURE.transit.snapshot.jd_ut);
  // Ortak vurgu: tablodaki Ay, haritadaki Ay'ı vurgular.
  await natal.locator('tbody tr[data-tara="1"]').getByRole("button", { name: "Ay" }).hover();
  await expect(page.locator('svg [data-graha="Moon"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(natal.locator('tbody tr[data-tara="8"]')).toHaveClass(/is-highlighted/);

  // Vimshopaka bala: özet ve varga görünümü; ayar düzenle birlikte saklanır.
  await toggle("vimshopaka", /Vimshopaka bala panelini aç\/kapat/);
  const vimshopaka = page.getByRole("region", { name: "Vimshopaka bala" });
  await expect(vimshopaka).toBeFocused();
  const summary = vimshopaka.getByRole("table", { name: /^Vimshopaka bala: yedi grahanın dört varga kümesindeki değeri/ });
  await expect(summary.locator('tbody tr[data-graha="Jupiter"] td')).toHaveText(["17.200", "17.200", "14.150", "13.700"]);
  await vimshopaka.getByRole("combobox", { name: "Ayrıntı" }).selectOption("vargas");
  const vargas = vimshopaka.getByRole("table", { name: "On altı vargada burç ve basamak değeri" });
  await expect(vargas.locator('tbody tr[data-graha="Sun"] td[data-varga="D9"]')).toHaveText("Asl 20");
  await expect(vimshopaka.getByText(/^Doğrulanmamış tablo: küme ağırlıkları/)).toBeVisible();
  expect(await ipcCalls(page, "compute_vimshopaka")).toHaveLength(1);
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "navatara" || p.kind === "vimshopaka")
        .map((p) => [p.kind, p.settings]);
    })
    .toEqual([
      ["navatara", {}],
      ["vimshopaka", { detail: "vargas" }],
    ]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await toggle("navatara", /Navatara panelini aç\/kapat/);
  await expect(navatara).toHaveCount(0);
  await expect(vimshopaka).toBeVisible();
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});
