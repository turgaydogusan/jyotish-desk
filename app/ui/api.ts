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
  type ChartQuery,
  type ChartSearchCursor,
  type CommandError,
  type DashaSeed,
  type EventSearchSpec,
  type Gender,
  type Graha,
  type MonthId,
  type MuhurtaPageCursor,
  type MuhurtaPlace,
  type MuhurtaQuery,
  type NatalMoon,
  type NewChart,
  type NewClient,
  type BirthData,
  type PanelLayout,
  type PrashnaEdit,
  type PrashnaRequest,
  type SavedMuhurtaFilter,
  type SaveSearchRequest,
  type Sign,
  type TagChartsRequest,
  type TransitAt,
  type TransitStepRequest,
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
  // Geçiş araçları (yol haritası Plan 26). `year` boşsa yerin bugünkü yılıdır ("bugün" Rust'ta
  // okunur). Yer kişisel veridir: loglanmaz.
  sankrantiList: (year: number | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.sankrantiList(year, place, settings)),
  // `month` boşsa yerin bugünkü ayıdır; önceki ve sonraki ay yanıtla gelir (arayüz tarih hesabı yapmaz).
  ephemerisMonth: (month: MonthId | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.ephemerisMonth(month, place, settings)),
  // Adım aritmetiği Rust'tadır; `request.tz` tarayıcının IANA dilimidir (yol ya da adres değil, C2′).
  transitStep: (request: TransitStepRequest) => wrap(() => commands.transitStep(request)),
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
  // Bhava tablosu ve Sudarshana chakra (Plan 25). Tablo: `null` doğum saati bilinmeyen haritadır.
  // Chakra saat bilinmese de döner; lagna halkasının evleri `null`, sonuç `approximate` olur.
  computeBhavaTable: (input: ChartInput) => wrap(() => commands.computeBhavaTable(input)),
  computeSudarshana: (input: ChartInput) => wrap(() => commands.computeSudarshana(input)),
  computeJaimini: (input: ChartInput) => wrap(() => commands.computeJaimini(input)),
  // Chara dasha: `null` doğum saati bilinmeyen haritadır. `path` dönemin `signs`'ı, `cycle`
  // çevrimidir; `charaDashaAt` transit anının dönem yolunu verir (arayüz tarih karşılaştırmaz).
  computeCharaDasha: (input: ChartInput) => wrap(() => commands.computeCharaDasha(input)),
  charaDashaChildren: (input: ChartInput, path: Sign[], cycle: number) => wrap(() => commands.charaDashaChildren(input, path, cycle)),
  charaDashaAt: (input: ChartInput, jdUt: number) => wrap(() => commands.charaDashaAt(input, jdUt)),
  // Shadbala ve Bhava bala: `null` doğum saati bilinmeyen ya da o gün Güneş'in doğmadığı ya da
  // batmadığı haritadır.
  computeShadbala: (input: ChartInput) => wrap(() => commands.computeShadbala(input)),
  // Doğum anı ekleri (Plan 23): üçü de lagna istemez; doğum saati bilinmiyorsa sonuç
  // `approximate` işaretlidir. `transitJdUt` transit anıdır (`TransitView.snapshot.jd_ut`);
  // `null` ise navatara yalnızca doğum tablosunu döner.
  computePanchangExtras: (input: ChartInput) => wrap(() => commands.computePanchangExtras(input)),
  computeNavatara: (input: ChartInput, transitJdUt: number | null) => wrap(() => commands.computeNavatara(input, transitJdUt)),
  computeVimshopaka: (input: ChartInput) => wrap(() => commands.computeVimshopaka(input)),
  // Yogalar: doğum saati bilinmiyorsa rapor yine döner; lagna isteyen kurallar atlanmış,
  // diğer kayıtlar `approximate` işaretlidir.
  computeYogas: (input: ChartInput) => wrap(() => commands.computeYogas(input)),
  // Avasthalar: doğum saati bilinmiyorsa rapor yine döner; Lajjita değerlendirilmemiş, Shayanadi
  // hesaplanmamıştır (`time_known: false`). O gün gün doğumu yoksa yalnızca Shayanadi boştur.
  computeAvasthas: (input: ChartInput) => wrap(() => commands.computeAvasthas(input)),
  // Varshaphala (Plan 27): `age` tamamlanan yıldır, `null` ise süren yılı Rust bulur; `place`
  // yıllık haritanın yeridir. Kayıt komutu yeri ve anı veritabanından okur: yalnızca kaynak
  // haritanın kimliği, yaş ve "bulunulan yer" seçimi gider. Girdi ve yer loglanmaz.
  computeVarsha: (input: ChartInput, age: number | null, place: MuhurtaPlace) => wrap(() => commands.computeVarsha(input, age, place)),
  saveVarshaChart: (chartId: number, age: number, home: boolean) => wrap(() => commands.saveVarshaChart(chartId, age, home)),
  // Uyum: `groom` damat rolünün, `bride` gelin rolünün girdisidir. Eş haritanın girdisi ve kısa
  // kimliği store'dan okunur; harita yoksa (silinmiş eş) ikisi de `null` döner. Girdiler doğum
  // verisidir, eş bağı ilişki bilgisidir: loglanmaz.
  computeCompat: (groom: ChartInput, bride: ChartInput) => wrap(() => commands.computeCompat(groom, bride)),
  chartInput: (chartUuid: string) => wrap(() => commands.chartInput(chartUuid)),
  chartBrief: (chartUuid: string) => wrap(() => commands.chartBrief(chartUuid)),
  // Muhurta: gün panchangı transit anını içeren gündür ("şimdi" Rust'ta okunur). Arama dilim
  // dilim yürür: ilk çağrıda `cursor` boştur, dönen `next` sonraki çağrıya olduğu gibi verilir;
  // aramayı durdurmak sonraki dilimi istememektir. Yer ve doğum Ay'ı kişisel veridir: loglanmaz.
  dayPanchang: (at: TransitAt, place: MuhurtaPlace, settings: CalcSettings, natal: NatalMoon | null) => wrap(() => commands.dayPanchang(at, place, settings, natal)),
  muhurtaSearch: (query: MuhurtaQuery, settings: CalcSettings, cursor: MuhurtaPageCursor | null) => wrap(() => commands.muhurtaSearch(query, settings, cursor)),
  // Geçiş takvimi (Plan 29): bir ayın takvim günleri, gün doğumu olguları ve olayları. `month`
  // boşsa yerin bugünkü ayıdır ("bugün" Rust'ta okunur); önceki ve sonraki ay yanıtta döner.
  // İstek yer ve (doğum türü seçiliyse) açık haritanın doğum verisini taşır: loglanmaz.
  monthCalendar: (request: Parameters<typeof commands.monthCalendar>[0]) => wrap(() => commands.monthCalendar(request)),
  // Kayıtlı süzgeçler: ad ve koşullar; yer, tarih ve kişi kaydedilmez. Kayıt listenin tamamını yazar.
  loadMuhurtaFilters: () => wrap(() => commands.loadMuhurtaFilters()),
  saveMuhurtaFilters: (filters: SavedMuhurtaFilter[]) => wrap(() => commands.saveMuhurtaFilters(filters)),
  // Prashna: soru anının haritası ("şimdi" Rust'ta okunur; girilen an sorunun yerinin duvar
  // saatidir). `createPrashna` haritayı kurar ve açar; `updatePrashna` soru metnini ve odak evini
  // kaydeder; olgular panel açıkken istenir ve saklanmaz. Soru metni, yer ve an loglanmaz.
  createPrashna: (request: PrashnaRequest) => wrap(() => commands.createPrashna(request)),
  updatePrashna: (chartId: number, edit: PrashnaEdit) => wrap(() => commands.updatePrashna(chartId, edit)),
  computePrashnaFacts: (input: ChartInput, focusHouse: number | null) => wrap(() => commands.computePrashnaFacts(input, focusHouse)),
  // Arama (ileri modüller spec'i §12). Harita araması sayfa sayfa yürür: ilk çağrıda `cursor`
  // boştur ve `searchId` yeni bir aramayı başlatır; dönen `next` sonraki çağrıya olduğu gibi
  // verilir. Durdurulan ya da yerine yenisi başlatılan aramanın isteği `search_stale` döner.
  // Olay araması tek çağrıdır ve `eventSearchCancel` ile durdurulur. Sorgu, sonuçlar (adlar,
  // sayılar, kimlikler) ve yer loglanmaz.
  chartSearch: (query: ChartQuery, searchId: number, cursor: ChartSearchCursor | null) => wrap(() => commands.chartSearch(query, searchId, cursor)),
  chartSearchCancel: (searchId: number) => wrap(() => commands.chartSearchCancel(searchId)),
  tagCharts: (request: TagChartsRequest) => wrap(() => commands.tagCharts(request)),
  listSavedSearches: () => wrap(() => commands.listSavedSearches()),
  saveSearch: (request: SaveSearchRequest) => wrap(() => commands.saveSearch(request)),
  deleteSavedSearch: (searchId: number) => wrap(() => commands.deleteSavedSearch(searchId)),
  eventSearch: (spec: EventSearchSpec, searchId: number) => wrap(() => commands.eventSearch(spec, searchId)),
  eventSearchCancel: (searchId: number) => wrap(() => commands.eventSearchCancel(searchId)),
  searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
  previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
  appInfo: () => wrap(() => commands.appInfo()),
  // Güncelleme: denetim yalnızca kullanıcının isteğiyle çağrılır (Plan 13 D1); `version`
  // kullanıcının kurmayı onayladığı sürümdür, adres ya da yol değildir (C2′).
  checkForUpdate: () => wrap(() => commands.checkForUpdate()),
  installUpdate: (version: string) => wrap(() => commands.installUpdate(version)),
};

export type Api = typeof api;
