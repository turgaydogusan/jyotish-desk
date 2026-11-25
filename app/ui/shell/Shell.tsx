import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { BirthForm } from "../birth/BirthForm";
import { Sidebar } from "../clients/Sidebar";
import { Dashboard } from "../dashboard/Dashboard";
import { useSession } from "../session/store";
import { matchShortcut } from "../shortcuts";
import { useLayoutStore } from "../view/store";
import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
import { EmptyState } from "./EmptyState";
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
  const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
  const [sidebar, setSidebar] = useState(true);
  const { opened } = useSession();
  const measureRef = useRef<HTMLDivElement>(null);
  const width = useElementWidth(measureRef);

  // Boyamadan önce uygulanır (yanlış tema görünmez); main.tsx ilk çizimden önce de uygular.
  useLayoutEffect(() => applyTheme(themePref), [themePref]);
  const changeTheme = (pref: ThemePref) => {
    saveThemePref(pref);
    setThemePref(pref);
  };

  // Genel kısayollar (shortcuts.ts). Cmd+Shift+N notlar panelini etkin düzende açar/kapatır
  // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar. Dinleyici bir
  // kez kurulur ve güncel durumu `latest`'ten okur: kayıttan hemen sonra basılan ⌘E, effect
  // yeniden kurulmadan önce de açık haritayı görür.
  const latest = useRef({ toggleNotes, printView, opened });
  latest.current = { toggleNotes, printView, opened };
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const command = matchShortcut(e);
      const now = latest.current;
      if (command?.id === "toggleNotes" && !now.printView) {
        e.preventDefault();
        now.toggleNotes();
      } else if (command?.id === "help") {
        e.preventDefault();
        setHelp(true);
      } else if (command?.id === "newChart") {
        e.preventDefault();
        setForm("new");
      } else if (command?.id === "editChart" && now.opened) {
        e.preventDefault();
        setForm("edit");
      } else if (command?.id === "sidebar" && !now.printView) {
        e.preventDefault();
        setSidebar((v) => !v);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="shell">
      <TopBar
        themePref={themePref}
        onThemeChange={changeTheme}
        printView={printView}
        onTogglePrint={() => setPrintView((v) => !v)}
        onHelp={() => setHelp(true)}
        onNewChart={() => setForm("new")}
        onEditChart={opened ? () => setForm("edit") : undefined}
        sidebarOpen={sidebar}
        onToggleSidebar={() => setSidebar((v) => !v)}
      />
      <NoticeBar />
      <div className="shell-body">
        {sidebar && !printView && <Sidebar onAddChart={() => setForm("add")} />}
        <main className="shell-main">
          <div ref={measureRef}>{printView ? <PrintView /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
        </main>
      </div>
      {help && <HelpDialog onClose={() => setHelp(false)} />}
      {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
    </div>
  );
}
