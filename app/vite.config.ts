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
  },
});
