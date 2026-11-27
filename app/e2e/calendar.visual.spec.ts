// Transit takviminin görsel temelleri (yol haritası Plan 29; yalnızca yerel: `pnpm e2e:visual`).
// Ayrı dosyadır: `visual.spec.ts` paralel planlarla paylaşılır; "visual" projesi adı
// `visual.spec.ts` ile biten her dosyayı koşar. Fikstürün ayı (Kasım 2026, Delhi), açık ve koyu
// tema: ay ızgarası (bugün işaretli) ve bütün olaylarla dolu ızgara.
import { expect, test, type Page } from "@playwright/test";
import { mockIpc } from "./ipc";

async function openCalendar(page: Page, theme: "light" | "dark") {
  await mockIpc(page, { session: true });
  await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  // Kenar çubuğu kapalı: temel görüntüler pano genişliğine bağlı değil.
  await page.keyboard.press("Meta+Backslash");
  await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("takvim");
  await palette.getByRole("option", { name: /Transit takvimi panelini aç\/kapat/ }).click();
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  const grid = panel.locator("table.calendar-grid");
  await expect(grid.locator("td[data-date]")).toHaveCount(30);
  return { panel, grid };
}

for (const theme of ["light", "dark"] as const) {
  test(`calendar ${theme}`, async ({ page }) => {
    const { grid } = await openCalendar(page, theme);
    await grid.scrollIntoViewIfNeeded();
    await expect(grid).toHaveScreenshot(`calendar-${theme}.png`);
  });
}

test("calendar full light", async ({ page }) => {
  const { panel, grid } = await openCalendar(page, "light");
  await panel.locator("summary", { hasText: /^Olay türleri/ }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await panel.getByRole("checkbox", { name: /^Ay'ın olayları da/ }).check();
  await expect(grid.locator("li.calendar-more").first()).toBeVisible();
  // Seçili gün: dolunay günü.
  await grid.locator('td[data-date="2026-11-24"]').click();
  await expect(grid.locator('td[aria-selected="true"]')).toHaveAttribute("data-date", "2026-11-24");
  await grid.scrollIntoViewIfNeeded();
  await expect(grid).toHaveScreenshot("calendar-full-light.png");
});
