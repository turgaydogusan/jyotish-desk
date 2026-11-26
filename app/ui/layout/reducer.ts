// Düzen durumu için saf reducer. Kimlikler eylemle gelir; reducer rastgelelik üretmez.
import { movePanel, normalizePanels } from "./grid";
import { builtinLayout, MAX_LAYOUTS, MAX_PANELS, MODULE_PANELS, type Layout, type LayoutState, type ModulePanelKind, type Panel } from "./model";

export type LayoutAction =
  | { type: "select"; id: string }
  | { type: "selectIndex"; index: number }
  | { type: "saveAs"; id: string; name: string }
  | { type: "rename"; id: string; name: string }
  | { type: "duplicate"; sourceId: string; id: string; name: string }
  | { type: "delete"; id: string }
  | { type: "resetBuiltin"; id: string }
  | { type: "updatePanels"; panels: Panel[] }
  | { type: "toggleNotes"; panelId: string }
  | { type: "togglePanel"; kind: ModulePanelKind; panelId: string }
  | { type: "setPanelSettings"; panelId: string; patch: Record<string, unknown> };

/** Notlar paneli düzende yoksa sağa 4 sütun genişliğinde eklenir (spec §3.2). */
export const NOTES_PANEL = { x: 8, y: 0, w: 4, h: 8 } as const;

let seq = 0;
/** Oturum içinde benzersiz kimlik (ör. "user:m3k9x2a0"). */
export function newId(prefix: string): string {
  seq += 1;
  return `${prefix}:${Date.now().toString(36)}${seq.toString(36)}`;
}

export function activeLayout(state: LayoutState): Layout {
  return state.layouts.find((l) => l.id === state.active_layout_id) ?? state.layouts[0];
}

/** Yüklenen durumdaki her düzenin panellerini geçerli hale getirir. */
export function normalizeState(state: LayoutState): LayoutState {
  return { ...state, layouts: state.layouts.map((l) => ({ ...l, panels: normalizePanels(l.panels) })) };
}

function copyPanels(panels: Panel[]): Panel[] {
  return panels.map((p) => ({ ...p, settings: structuredClone(p.settings) }));
}

function mapLayout(state: LayoutState, id: string, f: (l: Layout) => Layout): LayoutState {
  return { ...state, layouts: state.layouts.map((l) => (l.id === id ? f(l) : l)) };
}

export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutState {
  switch (action.type) {
    case "select":
      return state.layouts.some((l) => l.id === action.id) ? { ...state, active_layout_id: action.id } : state;
    case "selectIndex": {
      const target = state.layouts[action.index];
      return target ? { ...state, active_layout_id: target.id } : state;
    }
    case "saveAs": {
      const name = action.name.trim();
      // Sınırda yeni düzen eklenmez (Rust fazlasını okumaz; LayoutPicker düğmeyi kapatır).
      if (name === "" || state.layouts.length >= MAX_LAYOUTS || state.layouts.some((l) => l.id === action.id)) return state;
      const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(activeLayout(state).panels) };
      return { ...state, active_layout_id: copy.id, layouts: [...state.layouts, copy] };
    }
    case "rename": {
      const name = action.name.trim();
      const target = state.layouts.find((l) => l.id === action.id);
      if (name === "" || !target || target.builtin) return state;
      return mapLayout(state, action.id, (l) => ({ ...l, name }));
    }
    case "duplicate": {
      const name = action.name.trim();
      const index = state.layouts.findIndex((l) => l.id === action.sourceId);
      if (name === "" || index < 0 || state.layouts.length >= MAX_LAYOUTS || state.layouts.some((l) => l.id === action.id)) return state;
      const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(state.layouts[index].panels) };
      const layouts = [...state.layouts];
      layouts.splice(index + 1, 0, copy);
      return { ...state, layouts };
    }
    case "delete": {
      const target = state.layouts.find((l) => l.id === action.id);
      if (!target || target.builtin) return state;
      const layouts = state.layouts.filter((l) => l.id !== action.id);
      const active = state.active_layout_id === action.id ? layouts[0].id : state.active_layout_id;
      return { ...state, layouts, active_layout_id: active };
    }
    case "resetBuiltin": {
      const fresh = builtinLayout(action.id);
      return fresh ? mapLayout(state, action.id, () => fresh) : state;
    }
    case "updatePanels":
      return mapLayout(state, state.active_layout_id, (l) => ({ ...l, panels: normalizePanels(action.panels) }));
    case "setPanelSettings":
      // Yalnızca verilen anahtarlar değişir; bilinmeyen ayarlar korunur (spec §3.3).
      return mapLayout(state, state.active_layout_id, (l) =>
        l.panels.some((p) => p.id === action.panelId)
          ? { ...l, panels: l.panels.map((p) => (p.id === action.panelId ? { ...p, settings: { ...p.settings, ...action.patch } } : p)) }
          : l,
      );
    case "toggleNotes":
      return mapLayout(state, state.active_layout_id, (l) => {
        if (l.panels.some((p) => p.kind === "notes")) {
          // Kaldırırken diğer paneller yerinde kalır (kullanıcının boşlukları korunur).
          return { ...l, panels: l.panels.filter((p) => p.kind !== "notes") };
        }
        const notes: Panel = { id: action.panelId, kind: "notes", ...NOTES_PANEL, settings: {} };
        // Notlar sağ üste sabitlenir; çakışan paneller aşağı itilir.
        return { ...l, panels: movePanel([...l.panels, notes], notes.id, NOTES_PANEL.x, NOTES_PANEL.y) };
      });
    case "togglePanel":
      return mapLayout(state, state.active_layout_id, (l) => {
        if (l.panels.some((p) => p.kind === action.kind)) {
          // Kaldırırken diğer paneller yerinde durur (notlar gibi).
          return { ...l, panels: l.panels.filter((p) => p.kind !== action.kind) };
        }
        const size = MODULE_PANELS.find((m) => m.kind === action.kind);
        // Sınırda panel eklenmez (Rust fazlasını okumaz).
        if (!size || l.panels.length >= MAX_PANELS || l.panels.some((p) => p.id === action.panelId)) return l;
        // Yeni panel düzenin altına, sola eklenir: hiçbir paneli yerinden oynatmaz.
        const y = Math.max(0, ...l.panels.map((p) => p.y + p.h));
        return { ...l, panels: [...l.panels, { id: action.panelId, kind: action.kind, x: 0, y, w: size.w, h: size.h, settings: {} }] };
      });
  }
}
