// bindings.ts'in ince sarmalayıcısı: IPC hataları da arayüze `{code, params}` olarak ulaşır.
// Üretilen `typedError` Tauri'nin düz string reddini (bilinmeyen komut, argüman çözme,
// ACL, yönetilmeyen state) CommandError diye işaretler ve `Error`'ları yeniden fırlatır.
//
// Arayüz yalnızca burada listelenen komutları çağırır. İçe ve dışa aktarma komutları
// (`importJson`, `importJhd`, `exportJson`, `exportCsv`) Plan 08 ertelenen maddeleri 1 ve 3
// kapanana kadar (M6) bilerek yoktur; `api.test.ts` bunu denetler.
// Hata parametreleri (`CommandError.params`) kişisel veri içerebilir: hiçbir yerde loglanmaz.
import {
  commands,
  type CommandError,
  type DashaSeed,
  type Graha,
  type NewChart,
  type NewClient,
  type BirthData,
  type PanelLayout,
  type ViewState,
} from "./bindings";

export type Result<T> = { status: "ok"; data: T } | { status: "error"; error: CommandError };

export function isCommandError(e: unknown): e is CommandError {
  return typeof e === "object" && e !== null && typeof (e as { code?: unknown }).code === "string";
}

function ipcError(e: unknown): CommandError {
  return { code: "ipc", params: { detail: String(e) } };
}

async function wrap<T>(call: () => Promise<Result<T>>): Promise<Result<T>> {
  try {
    const r = await call();
    if (r.status === "ok" || isCommandError(r.error)) return r;
    return { status: "error", error: ipcError(r.error) };
  } catch (e) {
    return { status: "error", error: ipcError(e) };
  }
}

export const api = {
  engineStatus: () => wrap(() => commands.engineStatus()),
  planetLongitudes: (jdUt: number | null) => wrap(() => commands.planetLongitudes(jdUt)),
  printWindow: () => wrap(() => commands.printWindow()),
  restoreSession: () => wrap(() => commands.restoreSession()),
  createClient: (input: NewClient) => wrap(() => commands.createClient(input)),
  updateChart: (chartId: number, chart: NewChart) => wrap(() => commands.updateChart(chartId, chart)),
  saveViewState: (chartId: number, viewState: ViewState) => wrap(() => commands.saveViewState(chartId, viewState)),
  loadPanelLayout: () => wrap(() => commands.loadPanelLayout()),
  savePanelLayout: (layout: PanelLayout | null) => wrap(() => commands.savePanelLayout(layout)),
  dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
  searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
  previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
};

export type Api = typeof api;
