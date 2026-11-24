import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./i18n";
import "./theme/tokens.css";
import { applyTheme, loadThemePref } from "./theme/theme";

// İlk çizimden önce yalnızca `data-theme` yazılır (yanlış tema parlaması olmaz). "system"
// dinleyicisi hemen kaldırılır: sistem değişikliklerini Shell'in kendi applyTheme'i izler,
// böylece iki dinleyici birikmez.
const disposeBootTheme = applyTheme(loadThemePref());
disposeBootTheme();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
