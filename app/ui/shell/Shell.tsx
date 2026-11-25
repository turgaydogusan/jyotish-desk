import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Dashboard } from "../dashboard/Dashboard";
import { matchShortcut } from "../shortcuts";
import { useLayoutStore } from "../view/store";
import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
import { HelpDialog } from "./HelpDialog";
import { NoticeBar } from "./notices";
import { PrintView } from "./PrintView";
import "./shell.css";
import { TopBar } from "./TopBar";
import { useElementWidth } from "./useElementWidth";

export function Shell() {
  const { toggleNotes } = useLayoutStore();
  const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
  const [printView, setPrintView] = useState(false);
  const [help, setHelp] = useState(false);
  const measureRef = useRef<HTMLDivElement>(null);
  const width = useElementWidth(measureRef);

  // Boyamadan önce uygulanır (yanlış tema görünmez); main.tsx ilk çizimden önce de uygular.
  useLayoutEffect(() => applyTheme(themePref), [themePref]);
  const changeTheme = (pref: ThemePref) => {
    saveThemePref(pref);
    setThemePref(pref);
  };

  // Genel kısayollar (shortcuts.ts). Cmd+Shift+N notlar panelini etkin düzende açar/kapatır
  // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const command = matchShortcut(e);
      if (command?.id === "toggleNotes" && !printView) {
        e.preventDefault();
        toggleNotes();
      } else if (command?.id === "help") {
        e.preventDefault();
        setHelp(true);
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
        onHelp={() => setHelp(true)}
      />
      <NoticeBar />
      <main className="shell-main">
        <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
      </main>
      {help && <HelpDialog onClose={() => setHelp(false)} />}
    </div>
  );
}
