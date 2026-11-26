// bindings.ts'in ince sarmalayıcısı: IPC hataları da arayüze `{code, params}` olarak ulaşır.
// Üretilen `typedError` Tauri'nin düz string reddini (bilinmeyen komut, argüman çözme,
// ACL, yönetilmeyen state) CommandError diye işaretler ve `Error`'ları yeniden fırlatır.
//
// Arayüz yalnızca burada listelenen komutları çağırır. JHD içe aktarma kayıtsızdır (C2′);
// `api.test.ts` bunu denetler.
// Hata parametreleri (`CommandError.params`) kişisel veri içerebilir: hiçbir yerde loglanmaz.
import {
  commands,
  type AppPrefs,
  type Bhavas,
  type CalcSettings,
  type ChartInput,
  type CommandError,
  type DashaSeed,
  type Gender,
  type Graha,
  type NewChart,
  type NewClient,
  type BirthData,
  type PanelLayout,
  type Sign,
  type TransitAt,
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
  printWindow: () => wrap(() => commands.printWindow()),
  restoreSession: () => wrap(() => commands.restoreSession()),
  openClient: (clientId: number, chartId: number | null) => wrap(() => commands.openClient(clientId, chartId)),
  createClient: (input: NewClient) => wrap(() => commands.createClient(input)),
  addChart: (clientId: number, chart: NewChart) => wrap(() => commands.addChart(clientId, chart)),
  updateChart: (chartId: number, chart: NewChart) => wrap(() => commands.updateChart(chartId, chart)),
  keepStoredMoment: (chartId: number) => wrap(() => commands.keepStoredMoment(chartId)),
  updateClient: (clientId: number, name: string, gender: Gender | null) => wrap(() => commands.updateClient(clientId, name, gender)),
  saveCalcSettings: (chartId: number, settings: CalcSettings) => wrap(() => commands.saveCalcSettings(chartId, settings)),
  getAppPrefs: () => wrap(() => commands.getAppPrefs()),
  setAppPrefs: (prefs: AppPrefs) => wrap(() => commands.setAppPrefs(prefs)),
  transitsAt: (at: TransitAt, settings: CalcSettings, natalBhavas: Bhavas, seed: DashaSeed) => wrap(() => commands.transitsAt(at, settings, natalBhavas, seed)),
  deleteClient: (clientId: number) => wrap(() => commands.deleteClient(clientId)),
  deleteChart: (chartId: number) => wrap(() => commands.deleteChart(chartId)),
  listClients: (limit: number) => wrap(() => commands.listClients(limit)),
  listClientsByTag: (tag: string, limit: number) => wrap(() => commands.listClientsByTag(tag, limit)),
  listCharts: (clientId: number) => wrap(() => commands.listCharts(clientId)),
  searchClients: (query: string, limit: number) => wrap(() => commands.searchClients(query, limit)),
  listTags: () => wrap(() => commands.listTags()),
  setChartTags: (chartId: number, tags: string[]) => wrap(() => commands.setChartTags(chartId, tags)),
  listNotes: (clientId: number, limit: number) => wrap(() => commands.listNotes(clientId, limit)),
  addNote: (clientId: number, chartId: number | null, body: string) => wrap(() => commands.addNote(clientId, chartId, body)),
  updateNote: (noteId: number, body: string) => wrap(() => commands.updateNote(noteId, body)),
  deleteNote: (noteId: number) => wrap(() => commands.deleteNote(noteId)),
  saveViewState: (chartId: number, viewState: ViewState) => wrap(() => commands.saveViewState(chartId, viewState)),
  loadPanelLayout: () => wrap(() => commands.loadPanelLayout()),
  savePanelLayout: (layout: PanelLayout | null) => wrap(() => commands.savePanelLayout(layout)),
  exportClientsJson: (includeNotes: boolean) => wrap(() => commands.exportClientsJson(includeNotes)),
  exportClientsCsv: () => wrap(() => commands.exportClientsCsv()),
  importClientsJson: () => wrap(() => commands.importClientsJson()),
  dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
  // İleri modüller: sonuç saklanmaz, panel açıkken istenir (ileri modüller spec'i K1). Girdi
  // `modules/input.ts`'teki `chartInput(opened)` ile kurulur.
  computeDrishti: (input: ChartInput) => wrap(() => commands.computeDrishti(input)),
  computeJaimini: (input: ChartInput) => wrap(() => commands.computeJaimini(input)),
  // Chara dasha: `null` doğum saati bilinmeyen haritadır. `path` dönemin `signs`'ı, `cycle`
  // çevrimidir; `charaDashaAt` transit anının dönem yolunu verir (arayüz tarih karşılaştırmaz).
  computeCharaDasha: (input: ChartInput) => wrap(() => commands.computeCharaDasha(input)),
  charaDashaChildren: (input: ChartInput, path: Sign[], cycle: number) => wrap(() => commands.charaDashaChildren(input, path, cycle)),
  charaDashaAt: (input: ChartInput, jdUt: number) => wrap(() => commands.charaDashaAt(input, jdUt)),
  searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
  previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
  appInfo: () => wrap(() => commands.appInfo()),
  // Güncelleme: denetim yalnızca kullanıcının isteğiyle çağrılır (Plan 13 D1); `version`
  // kullanıcının kurmayı onayladığı sürümdür, adres ya da yol değildir (C2′).
  checkForUpdate: () => wrap(() => commands.checkForUpdate()),
  installUpdate: (version: string) => wrap(() => commands.installUpdate(version)),
};

export type Api = typeof api;
