// Varshaphala görsel regresyonu (Plan 27; yalnızca yerel: `pnpm e2e:visual`). Ayrı dosyadır:
// `visual.spec.ts` paralel planlarla paylaşılır; "visual" projesi adı `visual.spec.ts` ile biten
// her dosyayı koşar. Temel görüntüler `e2e/varsha.visual.spec.ts-snapshots/` altındadır (darwin
// son ekli) ve yerelde üretilir: `pnpm --dir app exec playwright test --project=visual
// varsha.visual --update-snapshots`.
import { expect, test } from "@playwright/test";
import { mockIpc } from "./ipc";

// Yıllık harita (gerçek Delhi haritasının 27. yılı), muntha işaretiyle; açık ve koyu tema.
for (const theme of ["light", "dark"] as const) {
  test(`varshaphala annual chart ${theme}`, async ({ page }) => {
    await mockIpc(page, { session: true });
    await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
    // Kenar çubuğu kapalı: temel görüntü haritanın kendisidir, pano genişliğine bağlı değil.
    await page.keyboard.press("Meta+Backslash");
    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("varshaphala");
    await palette.getByRole("option", { name: /Varshaphala panelini aç\/kapat/ }).click();
    const svg = page.getByRole("region", { name: "Varshaphala" }).locator("svg.chart");
    await expect(svg.locator("[data-graha]")).toHaveCount(9);
    await expect(svg.locator(".chart-mark")).toHaveText("Mu");
    await svg.scrollIntoViewIfNeeded();
    await expect(svg).toHaveScreenshot(`varsha-${theme}.png`);
  });
}
