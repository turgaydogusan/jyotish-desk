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
      // Kısayol dinleyicisi masa çizilince bağlanır; başlık görünmeden basılan tuş kaybolur.
      await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
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

// Baskı sayfaları (Plan 12): önizlemedeki dört A4 kâğıt. Koyu tema seçilidir; kâğıt yine de
// beyazdır (`.paper`). Kuzey biçimi varsayılandır.
for (const sheet of ["cover", "charts", "dasha", "grahas"] as const) {
  test(`print sheet ${sheet}`, async ({ page }) => {
    await mockIpc(page, { session: true });
    await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "dark"));
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
    await page.keyboard.press("Meta+p");
    await expect(page.locator("section.print-page")).toHaveCount(4);
    await expect(page.locator(`[data-page="${sheet}"]`)).toHaveScreenshot(`print-${sheet}.png`);
  });
}

// Chalit görünümü (yol haritası Plan 25): Placidus ayarlı Delhi haritası, üç biçim, açık tema.
// Merkür, Jüpiter ve Venüs burç evinden bir önceki bhavanın hücresinde, `*` işaretiyle.
for (const style of ["north", "south", "circle"] as const) {
  test(`chalit ${style}`, async ({ page }) => {
    await mockIpc(page, { session: true, placidus: true });
    await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "light"));
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
    await page.keyboard.press("Meta+Backslash");
    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
    const chart = page.getByRole("region", { name: "Harita D1", exact: true });
    await chart.getByRole("combobox", { name: "Biçim" }).selectOption(style);
    await chart.getByRole("checkbox", { name: "Bhava chalit" }).check();
    await expect(chart.locator("svg.chart [data-shifted]")).toHaveCount(3);
    await expect(chart.locator("svg.chart")).toHaveScreenshot(`chalit-${style}.png`);
  });
}

// Sudarshana chakra (yol haritası Plan 25): gerçek Delhi haritası, açık ve koyu tema.
for (const theme of ["light", "dark"] as const) {
  test(`sudarshana ${theme}`, async ({ page }) => {
    await mockIpc(page, { session: true });
    await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
    await page.keyboard.press("Meta+Backslash");
    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("sudarshana");
    await palette.getByRole("option", { name: /Sudarshana chakra panelini aç\/kapat/ }).click();
    const svg = page.getByRole("region", { name: "Sudarshana chakra" }).locator("svg.sudarshana");
    await expect(svg.locator(".chart-planet")).toHaveCount(27);
    await svg.scrollIntoViewIfNeeded();
    await expect(svg).toHaveScreenshot(`sudarshana-${theme}.png`);
  });
}
