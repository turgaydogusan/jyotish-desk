import "@testing-library/jest-dom/vitest";
import { i18n } from "./i18n";

// jsdom `scrollIntoView`'u uygulamaz; etkin seçeneğini görünür tutan bileşenler (yer arama)
// onu çağırır. Çağrıyı denetleyen test `vi.spyOn` ile izler.
Element.prototype.scrollIntoView = () => {};

// Her test Türkçe ve boş depolamayla başlar.
afterEach(async () => {
  localStorage.clear();
  document.documentElement.lang = "tr";
  await i18n.changeLanguage("tr");
});
