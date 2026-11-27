import { vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    engineStatus: vi.fn(),
    printWindow: vi.fn(),
    restoreSession: vi.fn(),
    openClient: vi.fn(),
    createClient: vi.fn(),
    addChart: vi.fn(),
    updateChart: vi.fn(),
    deleteClient: vi.fn(),
    deleteChart: vi.fn(),
    listClients: vi.fn(),
    listClientsByTag: vi.fn(),
    listCharts: vi.fn(),
    searchClients: vi.fn(),
    listTags: vi.fn(),
    setChartTags: vi.fn(),
    listNotes: vi.fn(),
    addNote: vi.fn(),
    updateNote: vi.fn(),
    deleteNote: vi.fn(),
    saveViewState: vi.fn(),
    loadPanelLayout: vi.fn(),
    savePanelLayout: vi.fn(),
    exportClientsJson: vi.fn(),
    exportClientsCsv: vi.fn(),
    importClientsJson: vi.fn(),
    dashaChildren: vi.fn(),
    computeDrishti: vi.fn(),
    computeJaimini: vi.fn(),
    computeCharaDasha: vi.fn(),
    charaDashaChildren: vi.fn(),
    charaDashaAt: vi.fn(),
    computeShadbala: vi.fn(),
    computeYogas: vi.fn(),
    computeCompat: vi.fn(),
    chartInput: vi.fn(),
    chartBrief: vi.fn(),
    dayPanchang: vi.fn(),
    muhurtaSearch: vi.fn(),
    loadMuhurtaFilters: vi.fn(),
    saveMuhurtaFilters: vi.fn(),
    createPrashna: vi.fn(),
    updatePrashna: vi.fn(),
    computePrashnaFacts: vi.fn(),
    searchPlaces: vi.fn(),
    previewMoment: vi.fn(),
    getAppPrefs: vi.fn(),
    setAppPrefs: vi.fn(),
    saveCalcSettings: vi.fn(),
    updateClient: vi.fn(),
    transitsAt: vi.fn(),
    keepStoredMoment: vi.fn(),
    appInfo: vi.fn(),
    checkForUpdate: vi.fn(),
    installUpdate: vi.fn(),
  },
}));

import { api } from "./api";
import { commands } from "./bindings";
import bindingsSource from "./bindings.ts?raw";

test("passes ok results through", async () => {
  vi.mocked(commands.engineStatus).mockResolvedValueOnce({
    status: "ok",
    data: { seVersion: "2.10.03" },
  });
  expect(await api.engineStatus()).toEqual({ status: "ok", data: { seVersion: "2.10.03" } });
});

test("passes real CommandError objects through", async () => {
  const error = { code: "calc", params: { detail: "x" } };
  vi.mocked(commands.restoreSession).mockResolvedValueOnce({ status: "error", error });
  expect(await api.restoreSession()).toEqual({ status: "error", error });
});

test("normalizes a string rejection into an ipc error", async () => {
  vi.mocked(commands.printWindow).mockResolvedValueOnce({
    status: "error",
    error: "command print_window not found" as never,
  });
  expect(await api.printWindow()).toEqual({
    status: "error",
    error: { code: "ipc", params: { detail: "command print_window not found" } },
  });
});

test("catches thrown errors into an ipc error", async () => {
  vi.mocked(commands.engineStatus).mockRejectedValueOnce(new Error("boom"));
  expect(await api.engineStatus()).toEqual({
    status: "error",
    error: { code: "ipc", params: { detail: "Error: boom" } },
  });
});

test("every wrapper forwards its arguments to the command of the same name", async () => {
  const calls: [keyof typeof api, unknown[]][] = [
    ["engineStatus", []],
    ["printWindow", []],
    ["restoreSession", []],
    ["openClient", [2, null]],
    ["createClient", [{ name: "A" }]],
    ["addChart", [2, { label: "Olay" }]],
    ["updateChart", [3, { label: "" }]],
    ["deleteClient", [2]],
    ["deleteChart", [3]],
    ["listClients", [100]],
    ["listClientsByTag", ["kariyer", 100]],
    ["listCharts", [2]],
    ["searchClients", ["ayşe", 8]],
    ["listTags", []],
    ["setChartTags", [3, ["kariyer"]]],
    ["listNotes", [2, 100]],
    ["addNote", [2, 3, ""]],
    ["updateNote", [5, "metin"]],
    ["deleteNote", [5]],
    ["saveViewState", [3, { schema_version: 1 }]],
    ["loadPanelLayout", []],
    ["savePanelLayout", [null]],
    ["exportClientsJson", [false]],
    ["exportClientsCsv", []],
    ["importClientsJson", []],
    ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
    ["computeDrishti", [{ jd_ut: 1, time_known: true }]],
    ["computeJaimini", [{ jd_ut: 1, time_known: false }]],
    ["computeCharaDasha", [{ jd_ut: 1, time_known: true }]],
    ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
    ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
    ["computeShadbala", [{ jd_ut: 1, time_known: true }]],
    ["computeYogas", [{ jd_ut: 1, time_known: false }]],
    ["computeCompat", [{ jd_ut: 1, time_known: true }, { jd_ut: 2, time_known: false }]],
    ["chartInput", ["00000000-0000-4000-8000-000000000012"]],
    ["chartBrief", ["00000000-0000-4000-8000-000000000012"]],
    ["dayPanchang", ["Now", { lat: 28.6, lon: 77.2, tz: "Lmt" }, { ayanamsa: "Lahiri" }, { nakshatra: 15, sign: "Libra" }]],
    ["muhurtaSearch", [{ from_date: null, to_date: null }, { ayanamsa: "Lahiri" }, { cursor: { chunk: 1 } }]],
    ["loadMuhurtaFilters", []],
    ["saveMuhurtaFilters", [[{ name: "Yolculuk", criteria: {} }]]],
    ["createPrashna", [{ client_id: null, at: "Now", number: 54, question: "", focus_house: 7 }]],
    ["updatePrashna", [3, { question: "", focus_house: null }]],
    ["computePrashnaFacts", [{ jd_ut: 1, time_known: true, lagna_number: 54 }, 7]],
    ["searchPlaces", ["delhi", 8]],
    ["previewMoment", [{ date: "2000-01-01" }]],
    ["getAppPrefs", []],
    ["setAppPrefs", [{ locale: "en" }]],
    ["saveCalcSettings", [3, { ayanamsa: "Lahiri" }]],
    ["updateClient", [2, "A", null]],
    ["transitsAt", [{ kind: "now" }, { ayanamsa: "Lahiri" }, [], { birth_jd_ut: 1 }]],
    ["keepStoredMoment", [3]],
    ["appInfo", []],
    ["checkForUpdate", []],
    ["installUpdate", ["0.9.1"]],
  ];
  for (const [name, args] of calls) {
    const command = vi.mocked(commands[name] as (...a: unknown[]) => Promise<unknown>);
    command.mockResolvedValueOnce({ status: "ok", data: name });
    const call = api[name] as (...a: unknown[]) => Promise<unknown>;
    expect(await call(...args)).toEqual({ status: "ok", data: name });
    expect(command).toHaveBeenCalledWith(...args);
  }
  // Tablo her sarmalayıcıyı kapsar: yeni bir sarmalayıcı burada da yazılmadan geçmez.
  expect(calls.map(([name]) => name).sort()).toEqual(Object.keys(api).sort());
});

test("no command takes a file path from the UI and JHD import is not reachable (Plan 08 deferred items 1 and 3)", () => {
  // Dosya yolu yalnızca Rust'taki pencereden gelir; yol alan komut bağlamalarda yoktur.
  for (const name of ["export_json", "export_csv", "import_json", "import_jhd"]) {
    expect(bindingsSource).not.toContain(`"${name}"`);
  }
  expect(bindingsSource).not.toMatch(/\bpath: string\b|\bpaths: string\[\]/);
  for (const name of ["importJson", "importJhd", "exportJson", "exportCsv"]) {
    expect(Object.keys(api)).not.toContain(name);
  }
});

test("every string argument of a command is a known, non-path one (found by type, not by name)", () => {
  const block = bindingsSource.slice(bindingsSource.indexOf("export const commands = {"));
  const found: string[] = [];
  for (const m of block.matchAll(/^\s+(\w+): \(([^)]*)\) =>/gm)) {
    for (const p of m[2].split(",")) {
      const arg = /^\s*(\w+)\??: (string(?:\[\])?)(?: \| null)?\s*$/.exec(p);
      if (arg) found.push(`${m[1]}.${arg[1]}`);
    }
  }
  expect(found.sort()).toEqual(
    // `chartUuid` bir kayıt kimliğidir (Rust'ta UUID olarak denetlenir), yol ya da adres değildir.
    ["addNote.body", "chartBrief.chartUuid", "chartInput.chartUuid", "installUpdate.version", "listClientsByTag.tag", "searchClients.query", "searchPlaces.query", "setChartTags.tags", "updateClient.name", "updateNote.body"].sort(),
  );
});
