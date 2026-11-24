import "@testing-library/jest-dom/vitest";
import { i18n } from "./i18n";

// Her test Türkçe ve boş depolamayla başlar.
afterEach(async () => {
  localStorage.clear();
  document.documentElement.lang = "tr";
  await i18n.changeLanguage("tr");
});
