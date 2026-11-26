import { expect, test } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

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
  // D9 panelinde işaret kutusu kapalıdır.
  await expect(page.getByRole("region", { name: "Harita D9" }).getByRole("checkbox", { name: "Arudha padalar" })).toBeDisabled();

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
