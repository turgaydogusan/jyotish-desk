import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./i18n";
import "./theme/tokens.css";
import { applyTheme, loadThemePref } from "./theme/theme";

applyTheme(loadThemePref()); // ilk çizimden önce: yanlış tema parlaması olmaz

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
