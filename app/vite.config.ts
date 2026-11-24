import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Tauri dev sunucusu sabit portta bekler; port doluysa hata verilir.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", target: "safari16" },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["ui/setupTests.ts"],
    include: ["ui/**/*.test.{ts,tsx}"],
    // Vitest CSS içe aktarımlarını varsayılan olarak boşaltır; `?raw` ile okunan CSS
    // (tema ve renk testleri) ham metin olarak gelsin diye bu içe aktarımlar dahil edilir.
    css: { include: [/\.css\?raw$/] },
  },
});
