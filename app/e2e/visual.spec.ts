// Görsel regresyon (yalnızca yerel: `pnpm e2e:visual`). Üç harita biçimi × açık/koyu tema,
// gerçek Delhi D1. Temel görüntüler `e2e/visual.spec.ts-snapshots/` altında, darwin son ekli;
// başka font yığınında (Linux CI) piksel farkı beklenir, bu yüzden CI'da koşmaz.
import { expect, test } from "@playwright/test";
import { mockIpc } from "./ipc";

for (const theme of ["light", "dark"] as const) {
  for (const style of ["north", "south", "circle"] as const) {
    test(`D1 ${style} ${theme}`, async ({ page }) => {
      await mockIpc(page, { session: true });
      await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
      await page.goto("/");
      // Kenar çubuğu kapalı: temel görüntüler haritanın kendisidir, pano genişliğine bağlı değil.
      await page.keyboard.press("Meta+Backslash");
      await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
      const chart = page.getByRole("region", { name: "Harita D1", exact: true });
      await chart.getByRole("combobox", { name: "Biçim" }).selectOption(style);
      await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", style);
      await expect(chart.locator("svg.chart")).toHaveScreenshot(`d1-${style}-${theme}.png`);
    });
  }
}
