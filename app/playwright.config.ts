import { defineConfig, devices } from "@playwright/test";

// Uçtan uca testler Vite geliştirme sunucusundaki arayüzü WebKit'te (Tauri'nin macOS
// web görünümüyle aynı motor) çalıştırır; Tauri IPC `e2e/ipc.ts` ile taklit edilir.
// "e2e" CI'da koşar. "visual" ekran görüntüsü karşılaştırmasıdır: font yığınına bağlı
// olduğundan yalnızca yerelde (`pnpm e2e:visual`) koşar.
export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    ...devices["Desktop Safari"],
    viewport: { width: 1440, height: 900 },
    baseURL: "http://localhost:1421",
    locale: "tr-TR",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "e2e", testMatch: /desk\.spec\.ts$/ },
    { name: "visual", testMatch: /visual\.spec\.ts$/, expect: { toHaveScreenshot: { maxDiffPixelRatio: 0.002 } } },
  ],
  webServer: {
    command: "pnpm exec vite --port 1421 --strictPort",
    url: "http://localhost:1421",
    reuseExistingServer: !process.env.CI,
  },
});
