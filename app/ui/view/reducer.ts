// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, iki dasha yolu ve
// açılan harita burada değişir.
import type { Graha, SignDashaPath } from "../bindings";
import { layoutReducer, type LayoutAction } from "../layout/reducer";
import { emptyCharaPath, parseCharaPath, parseDashaPath, sameCharaPath, type ViewModel } from "./model";

export type ViewAction =
  | { type: "layout"; action: LayoutAction }
  | { type: "setDashaPath"; path: Graha[] }
  | { type: "setCharaPath"; path: SignDashaPath }
  | { type: "openChart"; chartId: number; dashaPath: unknown; charaPath?: unknown }
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
    case "openChart":
      return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath), charaPath: parseCharaPath(action.charaPath) };
    case "closeChart":
      return state.chartId === null ? state : { ...state, chartId: null, dashaPath: [], charaPath: emptyCharaPath() };
  }
}
