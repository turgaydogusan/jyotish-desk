import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Dashboard } from "../dashboard/Dashboard";
import { isEditableTarget } from "../keyboard";
import { useLayoutStore } from "../view/store";
import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
import { PrintView } from "./PrintView";
import "./shell.css";
import { TopBar } from "./TopBar";
import { useElementWidth } from "./useElementWidth";

export function Shell() {
  const { toggleNotes } = useLayoutStore();
  const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
  const [printView, setPrintView] = useState(false);
  const measureRef = useRef<HTMLDivElement>(null);
  const width = useElementWidth(measureRef);

  // Boyamadan önce uygulanır (yanlış tema görünmez); main.tsx ilk çizimden önce de uygular.
  useLayoutEffect(() => applyTheme(themePref), [themePref]);
  const changeTheme = (pref: ThemePref) => {
    saveThemePref(pref);
    setThemePref(pref);
  };

  // Cmd+Shift+N: notlar panelini etkin düzende aç/kapat (spec §3.2). Yazı alanlarında,
  // tuş tekrarında, IME bileşiminde ve baskı görünümünde (pano yokken) yok sayılır.
  useEffect(() => {
    if (printView) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.repeat || e.isComposing || isEditableTarget(e.target)) return;
      if (e.metaKey && e.shiftKey && !e.altKey && !e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        toggleNotes();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggleNotes, printView]);

  return (
    <div className="shell">
      <TopBar
        themePref={themePref}
        onThemeChange={changeTheme}
        printView={printView}
        onTogglePrint={() => setPrintView((v) => !v)}
      />
      <main className="shell-main">
        <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
      </main>
    </div>
  );
}
