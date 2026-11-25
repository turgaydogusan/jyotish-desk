// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolu bir reducer'da;
// ≈ 500 ms debounce ile kayıt (spec §4), pencere gizlenince ya da kapanınca hemen kayıt,
// Cmd+1…9 ile sıradaki düzene geçiş (UI spec §3.2). Paneller düzene `useLayoutStore`,
// dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
import { createContext, useContext, useEffect, useMemo, useReducer, useRef, type Dispatch, type ReactNode } from "react";
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

type Internal = ViewAction | { type: "replace"; state: ViewModel };

function reduce(state: ViewModel, action: Internal): ViewModel {
  return action.type === "replace" ? action.state : viewReducer(state, action);
}

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
  const [state, dispatch] = useReducer(reduce, initial, (m) => ({ ...m, layout: normalizeState(m.layout) }));

  // Üst bileşen her render'da yeni bir `persistence` verebilir; debounce sıfırlanmasın.
  const persistenceRef = useRef(persistence);
  persistenceRef.current = persistence;
  const stateRef = useRef(state);
  stateRef.current = state;

  // `baseline` kaydı gerekmeyen son durumdur (açılış ya da yeni açılan harita).
  const baseline = useRef(state);
  const pending = useRef<ViewModel | null>(null);
  const flushRef = useRef(() => {
    if (pending.current) {
      const m = pending.current;
      pending.current = null;
      persistenceRef.current.save({ chartId: m.chartId, viewState: toViewState(m) });
    }
  });

  useEffect(() => {
    if (state === baseline.current) return;
    pending.current = state;
    const timer = setTimeout(() => {
      if (pending.current === state) flushRef.current();
    }, debounceMs);
    return () => clearTimeout(timer);
  }, [state, debounceMs]);

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
      openChart: (chartId, dashaPath) => {
        flushRef.current();
        const next = viewReducer(stateRef.current, { type: "openChart", chartId, dashaPath });
        baseline.current = next;
        dispatch({ type: "replace", state: next });
      },
    };
    return { view, layout };
  }, [state]);
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
