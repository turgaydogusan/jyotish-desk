// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolu tek saf reducer'dan geçer;
// ≈ 500 ms debounce ile kayıt (spec §4), pencere gizlenince ya da kapanınca hemen kayıt,
// Cmd+1…9 ile sıradaki düzene geçiş (UI spec §3.2). Paneller düzene `useLayoutStore`,
// dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
import type { Graha, ViewState } from "../bindings";
import { isEditableTarget } from "../keyboard";
import type { Layout, LayoutState, Panel } from "../layout/model";
import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
import { toViewState, type ViewModel } from "./model";
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
}

export interface ViewStore {
  model: ViewModel;
  /** Serileştirilmiş biçim (kayıtla aynı). */
  viewState: ViewState;
  setDashaPath(path: Graha[]): void;
  /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolu yüklenir. */
  openChart(chartId: number, dashaPath: unknown): void;
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
    // Yeni harita: bekleyen kayıt önce önceki haritaya yazılır; açılan haritanın kendi durumu
    // store'dan geldiği için yeniden yazılmaz.
    if (action.type === "openChart") flushRef.current();
    const next = viewReducer(stateRef.current, action);
    if (next === stateRef.current) return;
    stateRef.current = next;
    if (action.type !== "openChart") {
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
      if (e.repeat || e.isComposing || isEditableTarget(e.target)) return;
      if (!e.metaKey || e.shiftKey || e.altKey || e.ctrlKey) return;
      // Fiziksel tuş (`code`): AZERTY'de `key` rakam değildir.
      if (!/^Digit[1-9]$/.test(e.code)) return;
      const index = Number(e.code.slice(5)) - 1;
      if (index >= stateRef.current.layout.layouts.length) return;
      e.preventDefault();
      dispatch({ type: "layout", action: { type: "selectIndex", index } });
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
    };
    const view: ViewStore = {
      model: state,
      viewState: toViewState(state),
      setDashaPath: (path) => dispatch({ type: "setDashaPath", path }),
      openChart: (chartId, dashaPath) => dispatch({ type: "openChart", chartId, dashaPath }),
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
