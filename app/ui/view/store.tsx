// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolları tek saf reducer'dan geçer;
// ≈ 500 ms debounce ile kayıt (spec §4), pencere gizlenince ya da kapanınca hemen kayıt,
// Cmd+1…9 ile sıradaki düzene geçiş (UI spec §3.2). Paneller düzene `useLayoutStore`,
// dasha yollarına `useViewStore` ile ulaşır; birbirini çağırmaz.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
import type { Graha, SignDashaPath, ViewState } from "../bindings";
import type { Layout, LayoutState, ModulePanelKind, Panel } from "../layout/model";
import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
import { toViewState, type ViewModel } from "./model";
import { matchShortcut } from "../shortcuts";
import type { ViewPersistence } from "./persistence";
import { viewReducer, type ViewAction } from "./reducer";

export const SAVE_DEBOUNCE_MS = 500;

export interface LayoutStore {
  state: LayoutState;
  active: Layout;
  dispatch: Dispatch<LayoutAction>;
  select(id: string): void;
  saveAs(name: string): void;
  rename(id: string, name: string): void;
  duplicate(sourceId: string, name: string): void;
  remove(id: string): void;
  resetBuiltin(id: string): void;
  updatePanels(panels: Panel[]): void;
  toggleNotes(): void;
  /** İleri modül panelini etkin düzene ekler; düzende varsa kaldırır. */
  togglePanel(kind: ModulePanelKind): void;
  /** Etkin düzendeki bir panelin ayarlarına `patch`'i ekler (varga, biçim, sütunlar). */
  setPanelSettings(panelId: string, patch: Record<string, unknown>): void;
}

export interface ViewStore {
  model: ViewModel;
  /** Serileştirilmiş biçim (kayıtla aynı). */
  viewState: ViewState;
  setDashaPath(path: Graha[]): void;
  /** Chara dasha ağacının açık dalı (çevrim ve burç yolu). */
  setCharaPath(path: SignDashaPath): void;
  /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolları yüklenir. */
  openChart(chartId: number, dashaPath: unknown, charaPath?: unknown): void;
  /** Açık harita kapandı (ör. danışan silindi): bekleyen kayıt yazılır, sonraki kayıt yalnızca düzendir. */
  closeChart(): void;
  /**
   * Bekleyen kaydı hemen yazar; bekleyen yoksa bir şey yapmaz. Haritayı ya da danışanı silen
   * komuttan önce çağrılır: kayıt, silinen haritaya silmeden sonra gönderilmez (carried rule C5).
   */
  flush(): void;
}

const ViewContext = createContext<{ view: ViewStore; layout: LayoutStore } | null>(null);

export function ViewStateProvider({
  initial,
  persistence,
  children,
  debounceMs = SAVE_DEBOUNCE_MS,
}: {
  initial: ViewModel;
  persistence: ViewPersistence;
  children: ReactNode;
  debounceMs?: number;
}) {
  const [state, setState] = useState(() => ({ ...initial, layout: normalizeState(initial.layout) }));

  // Üst bileşen her render'da yeni bir `persistence` verebilir; debounce sıfırlanmasın.
  const persistenceRef = useRef(persistence);
  persistenceRef.current = persistence;
  const debounceRef = useRef(debounceMs);
  debounceRef.current = debounceMs;

  // Model `dispatch` içinde eşzamanlı ilerler; `stateRef` asıl kaynaktır, `state` onun render'a
  // yansımasıdır. Böylece kirli durum ve kayıt, render ya da effect zamanlamasına bağlı kalmaz:
  // aynı tick'te gelen eylemler ve `openChart` sırayla, kayıpsız uygulanır.
  const stateRef = useRef(state);
  /** Kaydı bekleyen son durum; `null` ise son durum kayıtlıdır (açılış ya da yeni açılan harita). */
  const pending = useRef<ViewModel | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const flushRef = useRef(() => {
    clearTimeout(timer.current);
    if (pending.current) {
      const m = pending.current;
      pending.current = null;
      persistenceRef.current.save({ chartId: m.chartId, viewState: toViewState(m) });
    }
  });

  const dispatch = useCallback((action: ViewAction) => {
    // Yeni ya da kapanan harita: bekleyen kayıt önce önceki haritaya yazılır; açılan haritanın
    // kendi durumu store'dan geldiği için yeniden yazılmaz (carried rule C5).
    const switching = action.type === "openChart" || action.type === "closeChart";
    if (switching) flushRef.current();
    const next = viewReducer(stateRef.current, action);
    if (next === stateRef.current) return;
    stateRef.current = next;
    if (!switching) {
      pending.current = next;
      clearTimeout(timer.current);
      timer.current = setTimeout(flushRef.current, debounceRef.current);
    }
    setState(next);
  }, []);

  // Tauri webview kapanışı `pagehide` atlayabildiği için `visibilitychange` de dinlenir.
  useEffect(() => {
    const flush = () => flushRef.current();
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flush();
    };
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      window.removeEventListener("pagehide", flush);
      document.removeEventListener("visibilitychange", onVisibility);
      flush();
    };
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const command = matchShortcut(e);
      if (command?.id !== "selectLayout" || command.index >= stateRef.current.layout.layouts.length) return;
      e.preventDefault();
      dispatch({ type: "layout", action: { type: "selectIndex", index: command.index } });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const value = useMemo(() => {
    const layoutDispatch: Dispatch<LayoutAction> = (action) => dispatch({ type: "layout", action });
    const layout: LayoutStore = {
      state: state.layout,
      active: activeLayout(state.layout),
      dispatch: layoutDispatch,
      select: (id) => layoutDispatch({ type: "select", id }),
      saveAs: (name) => layoutDispatch({ type: "saveAs", id: newId("user"), name }),
      rename: (id, name) => layoutDispatch({ type: "rename", id, name }),
      duplicate: (sourceId, name) => layoutDispatch({ type: "duplicate", sourceId, id: newId("user"), name }),
      remove: (id) => layoutDispatch({ type: "delete", id }),
      resetBuiltin: (id) => layoutDispatch({ type: "resetBuiltin", id }),
      updatePanels: (panels) => layoutDispatch({ type: "updatePanels", panels }),
      toggleNotes: () => layoutDispatch({ type: "toggleNotes", panelId: newId("notes") }),
      togglePanel: (kind) => layoutDispatch({ type: "togglePanel", kind, panelId: newId(kind) }),
      setPanelSettings: (panelId, patch) => layoutDispatch({ type: "setPanelSettings", panelId, patch }),
    };
    const view: ViewStore = {
      model: state,
      viewState: toViewState(state),
      setDashaPath: (path) => dispatch({ type: "setDashaPath", path }),
      setCharaPath: (path) => dispatch({ type: "setCharaPath", path }),
      openChart: (chartId, dashaPath, charaPath) => dispatch({ type: "openChart", chartId, dashaPath, charaPath }),
      closeChart: () => dispatch({ type: "closeChart" }),
      flush: () => flushRef.current(),
    };
    return { view, layout };
  }, [state, dispatch]);
  return <ViewContext.Provider value={value}>{children}</ViewContext.Provider>;
}

function useViewContext() {
  const ctx = useContext(ViewContext);
  if (ctx === null) throw new Error("view state hooks must be used inside <ViewStateProvider>");
  return ctx;
}

export function useViewStore(): ViewStore {
  return useViewContext().view;
}

export function useLayoutStore(): LayoutStore {
  return useViewContext().layout;
}
