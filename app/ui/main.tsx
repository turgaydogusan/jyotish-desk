import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./i18n";
import "./theme/tokens.css";
import { applyTheme, loadThemePref } from "./theme/theme";
import { loadLayout } from "./view/persistence";

// İlk çizimden önce yalnızca `data-theme` yazılır (yanlış tema parlaması olmaz). "system"
// dinleyicisi hemen kaldırılır: sistem değişikliklerini Shell'in kendi applyTheme'i izler,
// böylece iki dinleyici birikmez.
const disposeBootTheme = applyTheme(loadThemePref());
disposeBootTheme();

// Düzen store'dan okunur (≈ milisaniyeler); ilk çizim kayıtlı düzenle yapılır, yanlış düzen
// parlaması olmaz. Okuma hatası açılışı engellemez: hazır düzenler kullanılır.
void loadLayout().then((initialLayout) => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App initialLayout={initialLayout} />
    </StrictMode>,
  );
});
