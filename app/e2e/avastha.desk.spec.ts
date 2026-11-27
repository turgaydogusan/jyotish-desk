// Avastha akışları (Plan 24). Ayrı dosyadır: `desk.spec.ts` paralel planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

type AvasthaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };

test("avasthas: a palette panel with the four series and the Shayanadi terms; the graha table's Baladi column", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("avastha");
    await palette.getByRole("option", { name: /Avasthalar panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  const calls = async () => (await ipcCalls(page, "compute_avasthas")) as AvasthaCall[];
  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await toggle();
  const panel = page.getByRole("region", { name: "Avasthalar" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  await expect(panel.getByRole("table")).toHaveCount(2);
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });

  // Delhi fikstürü: Satürn Koç'ta (düşüş burcu); düşmanı Ay karşıdan bakar, Jüpiter yanındadır.
  const states = panel.getByRole("table", { name: /^Avasthalar: yedi grahanın/ });
  await expect(states.locator("tbody tr")).toHaveCount(7);
  const saturn = states.locator('tbody tr[data-graha="Saturn"]');
  await expect(saturn.locator('td[data-series="baladi"]')).toHaveText("Yuva");
  await expect(saturn.locator('td[data-series="jagradadi"]')).toHaveText("Sushupti");
  await expect(saturn.getByRole("listitem")).toHaveText(["Kshudhita (Ay)", "Mudita (Jüpiter)"]);
  await expect(saturn.locator('td[data-series="shayanadi"]')).toHaveText("Gamana");
  // Hiçbir Lajjitadi durumunda olmayan graha çizgi ve sözcük taşır.
  await expect(states.locator('tbody tr[data-graha="Moon"] td[data-series="lajjitadi"]')).toHaveText("—hiçbiri");

  // Shayanadi terimleri Rust'tan gelir: Satürn 2 × 7 × 5 + 15 + 25 + 3 = 113, artan 5.
  const terms = panel.getByRole("table", { name: "Shayanadi terimleri" });
  await expect(terms.locator('tbody tr[data-terms-of="Saturn"]').locator("th, td")).toHaveText(["Satürn", "2", "7", "5", "113", "5"]);
  await expect(panel.getByText("Ortak terimler: doğum nakshatrası 15 (Swati), ghati 25, lagna 3 (İki).")).toBeVisible();
  await expect(panel.getByText(/^Doğrulanmamış tablo:/)).toBeVisible();
  await expect(panel.getByText("Yöntem: avastha.bphs_v1")).toBeVisible();
  // Tabloda denetim az olabilir: kaydırma kabı klavye odağı alır.
  const scroll = panel.getByRole("group", { name: "Shayanadi terimleri" });
  await scroll.focus();
  await expect(scroll).toBeFocused();

  // Ortak vurgu: satırdaki graha haritadaki grahayı ve iki tablodaki satırını vurgular.
  await saturn.getByRole("button", { name: "Satürn" }).hover();
  await expect(page.locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(panel.locator("tr.is-highlighted")).toHaveCount(2);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await toggle();
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);

  // Graha tablosunun Baladi sütunu aynı komutu okur ve ayarı düzenle birlikte kaydedilir.
  await page.keyboard.press("Meta+Digit2");
  const table = page.getByRole("region", { name: "Graha tablosu" });
  await expect(table.getByRole("columnheader", { name: "Baladi" })).toHaveCount(0);
  const before = (await calls()).length;
  await table.locator("summary", { hasText: "Sütunlar" }).click();
  await table.getByRole("checkbox", { name: "Baladi" }).check();
  await expect(table.getByRole("columnheader", { name: "Baladi" })).toBeVisible();
  await expect(table.locator('td[data-baladi="Saturn"]')).toHaveText("Yuva");
  await expect(table.locator('td[data-baladi="Rahu"]')).toHaveText("");
  expect((await calls()).length).toBe(before + 1);
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: { baladi?: boolean } }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "graha_table")
        .map((p) => p.settings.baladi);
    })
    .toEqual([true]);
});

test("avasthas with an unknown birth time: Lajjita and Shayanadi are not given and the Moon's row is approximate", async ({ page }) => {
  await mockIpc(page, { session: true, unknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("avastha");
  await palette.getByRole("option", { name: /Avasthalar panelini aç\/kapat/ }).click();
  const panel = page.getByRole("region", { name: "Avasthalar" });
  await expect(panel.getByRole("table")).toHaveCount(1);
  const calls = (await ipcCalls(page, "compute_avasthas")) as AvasthaCall[];
  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
  await expect(panel.getByText(/^Doğum saati bilinmiyor: Lajjita değerlendirilmedi ve Shayanadi hesaplanmadı/)).toBeVisible();
  await expect(panel.locator('td[data-series="shayanadi"]')).toHaveText(Array.from({ length: 7 }, () => "—hesaplanmadı"));
  await expect(panel.locator('tr[data-graha="Moon"] td[data-series="baladi"]')).toHaveText("≈yaklaşık Vriddha");
  await expect(panel.locator('tr[data-graha="Sun"] td[data-series="baladi"]')).toHaveText("Yuva");
  await expect(panel.getByRole("listitem").filter({ hasText: "Lajjita" })).toHaveCount(0);
  await expect(panel.getByText(/^Ortak terimler/)).toHaveCount(0);
});
