// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, dasha yolu ve
// açılan harita burada değişir.
import type { Graha } from "../bindings";
import { layoutReducer, type LayoutAction } from "../layout/reducer";
import { parseDashaPath, type ViewModel } from "./model";

export type ViewAction =
  | { type: "layout"; action: LayoutAction }
  | { type: "setDashaPath"; path: Graha[] }
  | { type: "openChart"; chartId: number; dashaPath: unknown };

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
    case "openChart":
      return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath) };
  }
}
