// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, iki dasha yolu, eş
// bağı ve açılan harita burada değişir.
import type { CompatRoles, Graha, SignDashaPath } from "../bindings";
import { layoutReducer, type LayoutAction } from "../layout/reducer";
import { emptyCharaPath, emptyCompat, parseCharaPath, parseCompat, parseDashaPath, sameCharaPath, sameCompat, type ViewModel } from "./model";

export type ViewAction =
  | { type: "layout"; action: LayoutAction }
  | { type: "setDashaPath"; path: Graha[] }
  | { type: "setCharaPath"; path: SignDashaPath }
  /** Uyum panelinin eşi; `partnerUuid: null` bağı kaldırır. Harita açık değilken yok sayılır. */
  | { type: "setCompatPartner"; partnerUuid: string | null; roles: CompatRoles }
  /** İki rolü değiştirir; eş yokken yok sayılır. */
  | { type: "swapCompatRoles" }
  | { type: "openChart"; chartId: number; dashaPath: unknown; charaPath?: unknown; compat?: unknown }
  | { type: "closeChart" };

export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
  switch (action.type) {
    case "layout": {
      const layout = layoutReducer(state.layout, action.action);
      return layout === state.layout ? state : { ...state, layout };
    }
    case "setDashaPath": {
      const path = parseDashaPath(action.path);
      const same = path.length === state.dashaPath.length && path.every((g, i) => g === state.dashaPath[i]);
      return same ? state : { ...state, dashaPath: path };
    }
    case "setCharaPath": {
      const path = parseCharaPath(action.path);
      return sameCharaPath(path, state.charaPath) ? state : { ...state, charaPath: path };
    }
    case "setCompatPartner": {
      if (state.chartId === null) return state;
      const compat = parseCompat({ partnerUuid: action.partnerUuid, roles: action.roles });
      return sameCompat(compat, state.compat) ? state : { ...state, compat };
    }
    case "swapCompatRoles":
      if (state.chartId === null || state.compat.partnerUuid === null) return state;
      return { ...state, compat: { ...state.compat, roles: state.compat.roles === "OpenIsGroom" ? "OpenIsBride" : "OpenIsGroom" } };
    case "openChart":
      return {
        ...state,
        chartId: action.chartId,
        dashaPath: parseDashaPath(action.dashaPath),
        charaPath: parseCharaPath(action.charaPath),
        compat: parseCompat(action.compat),
      };
    case "closeChart":
      return state.chartId === null ? state : { ...state, chartId: null, dashaPath: [], charaPath: emptyCharaPath(), compat: emptyCompat() };
  }
}
