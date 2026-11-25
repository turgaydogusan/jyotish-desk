import { AboutDialog } from "../about/AboutDialog";
import { SettingsDialog } from "../settings/SettingsDialog";
import { startupUpdateCheck } from "../update/startup";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { BirthForm } from "../birth/BirthForm";
import { Sidebar } from "../clients/Sidebar";
import type { PaletteItem } from "../palette/items";
import { Palette } from "../palette/Palette";
import { Dashboard } from "../dashboard/Dashboard";
import { useSession } from "../session/store";
import { matchShortcut } from "../shortcuts";
import { useLayoutStore } from "../view/store";
import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
import { EmptyState } from "./EmptyState";
import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";
import { HelpDialog } from "./HelpDialog";
import { NoticeBar, useNotices } from "./notices";
import { loadSidebarOpen, saveSidebarOpen } from "./sidebarPref";
import { PrintView } from "../print/PrintView";
import "./shell.css";
import { TopBar } from "./TopBar";
import { useElementWidth } from "./useElementWidth";

export function Shell() {
  const layout = useLayoutStore();
  const { toggleNotes } = layout;
  const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
  const [printRequested, setPrintRequested] = useState(false);
  const [printError, setPrintError] = useState<CommandError | null>(null);
  const [help, setHelp] = useState(false);
  const [settings, setSettings] = useState(false);
  const [about, setAbout] = useState(false);
  const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
  const [sidebar, setSidebar] = useState(() => loadSidebarOpen());
  // Açık/kapalı durumu bir sonraki açılışta da geçerlidir (Plan 12 D11).
  useEffect(() => saveSidebarOpen(sidebar), [sidebar]);
  const [palette, setPalette] = useState(false);
  const toggleSidebar = () => {
    leaveSidebar();
    setSidebar((v) => !v);
  };
  const session = useSession();
  const { opened } = session;
  // Baskı önizlemesi yalnızca açık haritayla anlamlıdır; harita kapanırsa pano geri gelir.
  const printView = printRequested && opened !== null;
  // Harita önizlemenin ardında giderse (silme önizleme açılırken sürüyorduysa) istek de biter:
  // sonra açılan harita panoda açılır, önizlemede değil.
  useEffect(() => {
    if (opened === null) setPrintRequested(false);
  }, [opened]);
  const togglePrint = () => {
    setPrintError(null);
    setPrintRequested(!printView);
  };
  // Yazdırma penceresini Rust açar (C2′: arayüz yol vermez; PDF o pencerenin seçeneğidir).
  const printNow = async () => {
    setPrintError(null);
    const r = await api.printWindow();
    if (r.status === "error") setPrintError(r.error);
  };
  const { push } = useNotices();
  // Açılışta güncelleme denetimi yalnızca kullanıcı tercihi açtıysa yapılır (Plan 13 D1) ve
  // bir kez: StrictMode'un ikinci effect koşusu yeni istek göndermez.
  const updateChecked = useRef(false);
  useEffect(() => {
    if (updateChecked.current) return;
    updateChecked.current = true;
    void startupUpdateCheck(push);
  }, [push]);
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
  const latest = useRef({ toggleNotes, printView, opened, togglePrint, printNow });
  latest.current = { toggleNotes, printView, opened, togglePrint, printNow };
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const command = matchShortcut(e);
      const now = latest.current;
      if (command?.id === "toggleNotes" && !now.printView) {
        e.preventDefault();
        leaveNotes();
        now.toggleNotes();
      } else if (command?.id === "help") {
        e.preventDefault();
        setHelp(true);
      } else if (command?.id === "settings") {
        e.preventDefault();
        setSettings(true);
      } else if (command?.id === "newChart") {
        e.preventDefault();
        setForm("new");
      } else if (command?.id === "editChart" && now.opened) {
        e.preventDefault();
        setForm("edit");
      } else if (command?.id === "sidebar" && !now.printView) {
        e.preventDefault();
        leaveSidebar();
        setSidebar((v) => !v);
      } else if (command?.id === "palette") {
        e.preventDefault();
        setPalette(true);
      } else if (command?.id === "print") {
        // ⌘P her zaman tüketilir: web görünümü panoyu kendi başına yazdırmaz.
        e.preventDefault();
        if (!now.opened) return;
        if (now.printView) void now.printNow();
        else now.togglePrint();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const openClient = (clientId: number, chartId: number | null = null) =>
    void session.openClient(clientId, chartId).then((error) => error && push({ id: "client", type: "error", messageKey: "notice.openFailed", error }));

  // Paletten seçilen öğe. Danışan seçimi doğrudan açar (form ya da onay yok, spec §8).
  const runPalette = (item: PaletteItem, meta: boolean) => {
    switch (item.kind) {
      case "client":
        if (meta) setSidebar(true);
        return openClient(item.clientId);
      case "chart":
        return openClient(item.clientId, item.chartId);
      case "layout":
        return layout.select(item.layoutId);
      case "varga": {
        const panel = layout.active.panels.find((p) => p.kind === "chart");
        return panel && layout.setPanelSettings(panel.id, { varga: item.varga });
      }
      case "command":
        if (item.command === "newChart" || item.command === "addChart" || item.command === "editChart") {
          return setForm(item.command === "newChart" ? "new" : item.command === "addChart" ? "add" : "edit");
        }
        // Kısayollarındaki gibi: baskı görünümünde (pano ve kenar çubuğu yokken) yok sayılır.
        if (item.command === "settings") return setSettings(true);
        if (item.command === "print") return printView ? undefined : togglePrint();
        if (item.command === "toggleNotes") {
          if (printView) return;
          leaveNotes();
          return toggleNotes();
        }
        if (item.command === "sidebar") return printView ? undefined : toggleSidebar();
        if (item.command === "about") return setAbout(true);
        return setHelp(true);
    }
  };

  return (
    <div className="shell">
      <TopBar
        themePref={themePref}
        onThemeChange={changeTheme}
        printView={printView}
        onTogglePrint={opened ? togglePrint : undefined}
        onHelp={() => setHelp(true)}
        onNewChart={() => setForm("new")}
        onEditChart={opened ? () => setForm("edit") : undefined}
        sidebarOpen={sidebar}
        onToggleSidebar={toggleSidebar}
        onPalette={() => setPalette(true)}
        onSettings={() => setSettings(true)}
        onAbout={() => setAbout(true)}
      />
      <NoticeBar />
      <div className="shell-body">
        {sidebar && !printView && <Sidebar onAddChart={() => setForm("add")} />}
        <main id={MAIN_ID} className="shell-main" tabIndex={-1}>
          <div ref={measureRef}>{printView ? <PrintView error={printError} onPrint={() => void printNow()} /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
        </main>
      </div>
      {help && <HelpDialog onClose={() => setHelp(false)} />}
      {palette && <Palette onClose={() => setPalette(false)} onRun={runPalette} />}
      {settings && <SettingsDialog onClose={() => setSettings(false)} />}
      {about && <AboutDialog onClose={() => setAbout(false)} />}
      {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
    </div>
  );
}
