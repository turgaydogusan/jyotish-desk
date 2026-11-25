import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { boot } from "./boot";
import "./i18n";
import "./theme/tokens.css";
import { applyTheme, loadThemePref } from "./theme/theme";

// İlk çizimden önce yalnızca `data-theme` yazılır (yanlış tema parlaması olmaz). "system"
// dinleyicisi hemen kaldırılır: sistem değişikliklerini Shell'in kendi applyTheme'i izler,
// böylece iki dinleyici birikmez.
const disposeBootTheme = applyTheme(loadThemePref());
disposeBootTheme();

// Düzen ve son oturum store'dan okunur (Rust tarafında ≤ 50 ms); ilk çizim kayıtlı düzen
// ve açık haritayla yapılır, boş pano parlaması olmaz. Hatalar açılışı engellemez.
void boot().then((initial) => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App boot={initial} />
    </StrictMode>,
  );
});
