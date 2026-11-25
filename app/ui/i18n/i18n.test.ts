import { en } from "./en";
import { fr } from "./fr";
import { currentLanguage, i18n, LANGUAGE_STORAGE_KEY, loadLanguage, setLanguage } from "./index";
import { tr } from "./tr";

type Tree = { [k: string]: string | Tree };

function flatten(tree: Tree, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(tree)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (typeof v === "string") out[key] = v;
    else Object.assign(out, flatten(v, key));
  }
  return out;
}

const placeholders = (s: string) => [...s.matchAll(/{{\s*(\w+)\s*}}/g)].map((m) => m[1]).sort();
const ORDER = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const;

test.each([
  ["en", en],
  ["fr", fr],
])("%s has exactly the Turkish keys, non-empty values and the same placeholders", (_, messages) => {
  const base = flatten(tr);
  const other = flatten(messages);
  expect(Object.keys(other).sort()).toEqual(Object.keys(base).sort());
  for (const [key, value] of Object.entries(other)) {
    expect(value.trim(), key).not.toBe("");
    expect(placeholders(value), key).toEqual(placeholders(base[key]));
  }
});

test("graha abbreviation sets per language", () => {
  expect(ORDER.map((g) => tr.graha[g])).toEqual(["Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"]);
  expect(ORDER.map((g) => en.graha[g])).toEqual(["Su", "Mo", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"]);
  expect(ORDER.map((g) => fr.graha[g])).toEqual(["So", "Lu", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"]);
});

test("each language has 12 distinct sign abbreviations", () => {
  for (const m of [tr, en, fr]) {
    const signs = Array.from({ length: 12 }, (_, i) => m.sign[String(i) as keyof typeof m.sign]);
    expect(new Set(signs).size).toBe(12);
  }
});

test("Turkish is the default and initialisation is synchronous", () => {
  expect(currentLanguage()).toBe("tr");
  expect(i18n.t("panel.noChart")).toBe("Açık harita yok.");
  expect(i18n.t("errors.calc", { detail: "x<y" })).toBe("Hesap tamamlanamadı (x<y).");
});

test("setLanguage switches strings, persists the choice and sets <html lang>", async () => {
  await setLanguage("fr");
  expect(i18n.t("graha.Sun")).toBe("So");
  expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe("fr");
  expect(document.documentElement.lang).toBe("fr");
  expect(loadLanguage()).toBe("fr");
  localStorage.setItem(LANGUAGE_STORAGE_KEY, "de");
  expect(loadLanguage()).toBe("tr");
});

test("chart accessible-name words exist in every language", () => {
  expect([tr.chart.retro, tr.chart.combust, tr.chart.vargottama]).toEqual(["retro", "yanık", "vargottama"]);
  expect([en.chart.retro, en.chart.combust, en.chart.vargottama]).toEqual(["retrograde", "combust", "vargottama"]);
  expect([fr.chart.retro, fr.chart.combust, fr.chart.vargottama]).toEqual(["rétrograde", "combuste", "vargottama"]);
});

// Arka ucun döndürebildiği her `CommandError.code` (commands.rs, client_commands.rs,
// atlas_commands.rs, store_commands.rs), dosya ve içe aktarma kodlarıyla birlikte.
const BACKEND_CODES = [
  "ipc", "calc", "engine_stopped", "task_failed", "data_files_missing", "invalid_path", "env_override", "already_open", "job_panicked", "print_failed",
  "store_sqlite", "store_migration", "store_json", "store_too_new", "store_unavailable", "not_found", "invalid_input",
  "unknown_time_zone", "nonexistent_local_time", "ambiguous_local_time", "offset_out_of_range", "longitude_out_of_range",
  "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input",
  "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
  "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
] as const;

test("every backend error code reachable from the UI has a message", () => {
  for (const code of BACKEND_CODES) expect(Object.keys(tr.errors), code).toContain(code);
});

test("warning badges and hints cover every moment and chart warning", () => {
  const warnings = ["TimeUnknownNoon", "LmtFromLongitude", "PreTzdbReliability", "LowPrecision", "PlacidusPolarFallback", "SunriseUnavailable"];
  expect(Object.keys(tr.badge).sort()).toEqual([...warnings].sort());
  expect(Object.keys(tr.badgeHint).sort()).toEqual([...warnings].sort());
});
