// Düzen durumu için React bağlamı: reducer, 500 ms debounce ile kayıt (v1 spec §4) ve
// Cmd+1…9 ile sıradaki düzene geçiş (spec §3.2).
import { createContext, useContext, useEffect, useMemo, useReducer, useRef, type Dispatch, type ReactNode } from "react";
import { parseLayoutState, type Layout, type LayoutState, type Panel } from "./model";
import type { LayoutPersistence } from "./persistence";
import { activeLayout, layoutReducer, newId, normalizeState, type LayoutAction } from "./reducer";

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

const LayoutContext = createContext<LayoutStore | null>(null);

export function LayoutProvider({
  persistence,
  children,
  debounceMs = SAVE_DEBOUNCE_MS,
}: {
  persistence: LayoutPersistence;
  children: ReactNode;
  debounceMs?: number;
}) {
  const [state, dispatch] = useReducer(layoutReducer, persistence, (p) => normalizeState(parseLayoutState(p.load())));

  // Üst bileşen her render'da yeni bir `persistence` nesnesi verebilir; efektler buna bağlanırsa
  // debounce her render'da sıfırlanır. Bu yüzden son değer ref üzerinden okunur.
  const persistenceRef = useRef(persistence);
  persistenceRef.current = persistence;
  const layoutCount = useRef(state.layouts.length);
  layoutCount.current = state.layouts.length;

  // İlk durum zaten kayıtlıdır; yalnızca değişiklikler kaydedilir.
  const initial = useRef(state);
  const pending = useRef<LayoutState | null>(null);
  useEffect(() => {
    if (state === initial.current) return;
    pending.current = state;
    const timer = setTimeout(() => {
      if (pending.current !== state) return; // pagehide ile zaten yazıldı
      pending.current = null;
      persistenceRef.current.save(state);
    }, debounceMs);
    return () => clearTimeout(timer);
  }, [state, debounceMs]);

  // Pencere kapanırken ya da gizlenirken bekleyen kayıt hemen yazılır. Tauri webview kapanışı
  // `pagehide` atlayabildiği için `visibilitychange` de dinlenir.
  useEffect(() => {
    const flush = () => {
      if (pending.current) {
        persistenceRef.current.save(pending.current);
        pending.current = null;
      }
    };
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
      if (e.repeat || e.isComposing) return;
      if (!e.metaKey || e.shiftKey || e.altKey || e.ctrlKey) return;
      // Fiziksel tuş (`code`): AZERTY'de `key` rakam değildir.
      if (!/^Digit[1-9]$/.test(e.code)) return;
      const index = Number(e.code.slice(5)) - 1;
      if (index >= layoutCount.current) return;
      e.preventDefault();
      dispatch({ type: "selectIndex", index });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const store = useMemo<LayoutStore>(
    () => ({
      state,
      active: activeLayout(state),
      dispatch,
      select: (id) => dispatch({ type: "select", id }),
      saveAs: (name) => dispatch({ type: "saveAs", id: newId("user"), name }),
      rename: (id, name) => dispatch({ type: "rename", id, name }),
      duplicate: (sourceId, name) => dispatch({ type: "duplicate", sourceId, id: newId("user"), name }),
      remove: (id) => dispatch({ type: "delete", id }),
      resetBuiltin: (id) => dispatch({ type: "resetBuiltin", id }),
      updatePanels: (panels) => dispatch({ type: "updatePanels", panels }),
      toggleNotes: () => dispatch({ type: "toggleNotes", panelId: newId("notes") }),
    }),
    [state],
  );
  return <LayoutContext.Provider value={store}>{children}</LayoutContext.Provider>;
}

export function useLayoutStore(): LayoutStore {
  const store = useContext(LayoutContext);
  if (store === null) throw new Error("useLayoutStore must be used inside <LayoutProvider>");
  return store;
}
