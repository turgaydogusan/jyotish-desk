// Varshaphala akışları (Plan 27). Ayrı dosyadır: `desk.spec.ts` paralel planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

type VarshaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } }; age: number | null; place: { lat: number; lon: number; tz: unknown } };

async function togglePanel(page: Page) {
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("varshaphala");
  await palette.getByRole("option", { name: /Varshaphala panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
}

test("varshaphala: the annual chart, the muntha, year stepping, save as an event chart and the mudda dasha tree", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "compute_varsha")) as VarshaCall[];
  // Araştırma düzeni: dasa ağacı burada durur. Rapor panel açılmadan istenmez (K1).
  await page.keyboard.press("Meta+Digit2");
  await expect(page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true })).toBeVisible();
  expect(await calls()).toHaveLength(0);

  await togglePanel(page);
  const panel = page.getByRole("region", { name: "Varshaphala" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  const facts = panel.getByRole("table", { name: "Varshaphala: yılın olguları" });
  await expect(facts).toBeVisible();
  // Yaş gönderilmez: süren yılı Rust bulur. Yer açık haritanın doğum yeridir.
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0]).toMatchObject({ input: { jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } }, age: null, place: { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } } });

  // Delhi fikstürü: 26 yıl tamamlanmış; muntha Aslan, yıl lagnası Oğlak.
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("27. yıl (tamamlanan yıl 26)");
  await expect(facts.locator('tr[data-fact="start"] td')).toHaveText("2026-01-01 09:28 (UTC+05:30)");
  await expect(facts.locator('tr[data-fact="muntha"] td')).toHaveText("Asl · yıllık haritada 8. ev · doğum haritasında 3. ev · lordu Güneş");
  await expect(facts.locator('tr[data-fact="lagna"] td')).toHaveText("Oğl 22°04′ · lordu Satürn");
  await expect(panel.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue("26");
  const chart = panel.getByRole("group", { name: "Yıllık harita, 27. yıl" });
  await expect(chart.locator("[data-graha]")).toHaveCount(9);
  await expect(chart.locator(".chart-mark")).toHaveText("Mu");
  const positions = panel.getByRole("table", { name: "Yıllık harita konumları" });
  await expect(positions.locator("tbody tr")).toHaveCount(10);
  await expect(positions.locator('tbody tr[data-graha="Sun"]').locator("th, td")).toHaveText(["Güneş", "Yay", "16°30′", "Purva Ashadha", "12"]);
  await expect(panel.getByText("Yöntem: varsha.sidereal_return_v1")).toBeVisible();
  // Tabloda denetim az olabilir: kaydırma kabı klavye odağı alır.
  const scroll = panel.getByRole("group", { name: "Varshaphala: yılın olguları" });
  await scroll.focus();
  await expect(scroll).toBeFocused();

  // Ortak vurgu: yıllık haritanın tablosundaki graha, doğum haritasındaki grahayı da vurgular.
  await positions.getByRole("button", { name: "Satürn" }).hover();
  await expect(page.getByRole("region", { name: "Harita D1", exact: true }).locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);

  // Sonraki yıl: 27 yaşı istenir; muntha bir burç ilerler.
  await panel.getByRole("button", { name: "Sonraki yıl" }).click();
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("28. yıl (tamamlanan yıl 27)");
  await expect(facts.locator('tr[data-fact="muntha"] td')).toHaveText("Baş · yıllık haritada 5. ev · doğum haritasında 4. ev · lordu Merkür");
  await expect(panel.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeVisible();
  expect((await calls()).map((c) => c.age)).toEqual([null, 27]);

  // Olay haritası olarak kaydetme: kaynak haritanın kimliği, yaş ve yer seçimi gider; an ve yer
  // arayüzden gitmez. Harita açılmaz: başlık aynı danışanda ve aynı haritada durur.
  await panel.getByRole("button", { name: "Olay haritası olarak kaydet" }).click();
  await expect(panel.getByRole("status")).toHaveText("Olay haritası kaydedildi: Varshaphala 2027-01-01");
  expect(await ipcCalls(page, "save_varsha_chart")).toEqual([{ chartId: 1, age: 27, home: false }]);
  await expect(panel.getByRole("button", { name: "Olay haritası olarak kaydet" })).toHaveAttribute("aria-disabled", "true");
  await expect(panel.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeVisible();

  // Dasa ağacında mudda dasa: aynı yılı gösterir (27 yaşı: ilk lord Rahu).
  const tree = page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true });
  await tree.getByRole("combobox", { name: "Sistem" }).selectOption("mudda");
  const mudda = page.getByRole("region", { name: "Dasa ağacı Mudda dasa", exact: true });
  const items = mudda.getByRole("tree", { name: "Mudda dasa" }).getByRole("treeitem");
  await expect(items).toHaveCount(9);
  await expect(items.first()).toHaveAccessibleName("Rahu, 2027-01-01 15:41 – 2027-02-25 10:35");
  await expect(mudda.getByText("İlk lord Rahu: doğum nakshatrası 15 (Swati), tamamlanan yıl 27, artan 4.")).toBeVisible();
  await expect(mudda.getByText(/^Doğrulanmamış tablo:/)).toBeVisible();
  // Bir dönem dokuz alt dönemini açar (istek yapılmaz).
  const before = (await calls()).length;
  await items.first().locator(".dasha-row").click();
  await expect(mudda.getByRole("treeitem")).toHaveCount(18);
  await expect(mudda.getByRole("treeitem").nth(1)).toHaveAccessibleName("Rahu–Rahu, 2027-01-01 15:41 – 2027-01-09 20:55");
  expect((await calls()).length).toBe(before);
  // Ağaçtaki "Bu yıl" Varshaphala panelini de süren yıla döndürür.
  await mudda.getByRole("button", { name: "Bu yıl" }).click();
  await expect(panel.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue("26");
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("27. yıl (tamamlanan yıl 26)");
  await expect(mudda.getByRole("treeitem").first()).toHaveAccessibleName("Mars, 2026-01-01 09:28 – 2026-01-22 16:50");
  // Sistem seçimi düzenle birlikte kaydedilir.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: { system?: string } }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "dasha_tree")
        .map((p) => p.settings.system);
    })
    .toEqual(["mudda"]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await togglePanel(page);
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("varshaphala with an unknown birth time: no annual chart and no muntha; the mudda dasha is approximate", async ({ page }) => {
  await mockIpc(page, { session: true, unknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  await togglePanel(page);
  const panel = page.getByRole("region", { name: "Varshaphala" });
  await expect(panel.getByText(/^Doğum saati bilinmiyor: yıllık harita ve muntha hesaplanmadı/)).toBeVisible();
  const calls = (await ipcCalls(page, "compute_varsha")) as VarshaCall[];
  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
  await expect(panel.getByRole("table")).toHaveCount(1);
  await expect(panel.locator("tr[data-fact]")).toHaveCount(4);
  await expect(panel.locator('tr[data-fact="muntha"]')).toHaveCount(0);
  await expect(panel.locator("svg.chart")).toHaveCount(0);
  await expect(panel.getByRole("button", { name: "Olay haritası olarak kaydet" })).toHaveCount(0);

  const tree = page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true });
  await tree.getByRole("combobox", { name: "Sistem" }).selectOption("mudda");
  const mudda = page.getByRole("region", { name: "Dasa ağacı Mudda dasa", exact: true });
  await expect(mudda.getByRole("treeitem")).toHaveCount(9);
  await expect(mudda.getByText("Doğum saati bilinmiyor: dönüş anı, dönem tarihleri ve ilk lord yaklaşıktır (≈).")).toBeVisible();
});