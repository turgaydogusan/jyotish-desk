// Harita aramasının arayüz modeli (ileri modüller spec'i §12.1): koşul türleri, yeni koşulun
// başlangıç değeri, biçim denetimi ve eşleşen olgunun kısa özeti. Burada hesap yoktur (C4):
// koşulları Rust değerlendirir, olgular Rust'tan gelir; buradaki her şey seçim listesi, biçim
// denetimi ve gösterimdir. Sorgu ve sonuçlar loglanmaz.
import type { TFunction } from "i18next";
import type { ChartQuery, Condition, Dignity, GrahaFlag, KarakaRole, MatchFact, Paksha, QueryCondition, QueryScope, ScopeKind, Sign, SkyEventKind } from "../bindings";
import { signIndex, SIGNS, VARGAS } from "../chart/adapter";
import { GRAHA_IDS, type GrahaId } from "../chart/types";
import { arudhaLabel, KARAKA_ROLES, nakshatraName, tithiName, tithiShort } from "../panels/labels";

/** Rust `Condition`'ın on üç türü, formdaki sırayla. */
export const CONDITION_KINDS = [
  "GrahaInSign",
  "GrahaInHouse",
  "GrahaInNakshatra",
  "GrahaState",
  "LagnaInSign",
  "Conjunction",
  "GrahaAspects",
  "HouseLordInHouse",
  "MoonPhase",
  "DashaAt",
  "CharaKaraka",
  "ArudhaInSign",
  "YogaPresent",
] as const;
export type ConditionKind = (typeof CONDITION_KINDS)[number];

/** Rust `GrahaFlag::ALL`. */
export const GRAHA_FLAGS = ["Retrograde", "Combust", "Vargottama"] as const satisfies readonly GrahaFlag[];
export const DIGNITIES = ["Exalted", "Moolatrikona", "OwnSign", "GreatFriend", "Friend", "Neutral", "Enemy", "GreatEnemy", "Debilitated"] as const satisfies readonly Dignity[];
/** Rust `EIGHT_KARAKA_ROLES`. */
export const KARAKA_ROLE_IDS = ["Atma", "Amatya", "Bhratri", "Matri", "Pitri", "Putra", "Gnati", "Dara"] as const satisfies readonly KarakaRole[];
export const PAKSHAS = ["Shukla", "Krishna"] as const satisfies readonly Paksha[];
export const SCOPE_KINDS = ["Natal", "Event"] as const satisfies readonly ScopeKind[];
/** Tam bakış atan grahalar (Rust `ASPECTING_GRAHAS`): düğümler bakmaz. */
export const ASPECTING = GRAHA_IDS.slice(0, 7);
/** Rust `SkyEventKind::ALL`. */
export const EVENT_KINDS = [
  "SignIngress",
  "NakshatraIngress",
  "StationRetrograde",
  "StationDirect",
  "Conjunction",
  "NewMoon",
  "FullMoon",
  "TransitToNatal",
  "HouseFromNatalMoon",
] as const satisfies readonly SkyEventKind[];
export const HOUSES = Array.from({ length: 12 }, (_, i) => i + 1);
export const NAKSHATRAS = Array.from({ length: 27 }, (_, i) => i + 1);
export const TITHIS = Array.from({ length: 30 }, (_, i) => i + 1);
export const PADAS = [1, 2, 3, 4];
/** Rust `MAX_CONDITIONS`, `MAX_SCOPE_TAGS`, `MAX_YOGA_IDS`, `MAX_SEARCH_DASHA_LEVEL`, `MAX_SKY_EVENTS`. */
export const MAX_CONDITIONS = 12;
export const MAX_SCOPE_TAGS = 16;
export const MAX_YOGA_IDS = 16;
export const MAX_DASHA_LEVEL = 3;
export const MAX_EVENTS = 2000;
/** Rust `store::MAX_SAVED_SEARCHES`. */
export const MAX_SAVED_SEARCHES = 64;

export const EMPTY_SCOPE: QueryScope = { tags: [], kind: null, exclude_prashna: true };
export const EMPTY_QUERY: ChartQuery = { all_of: [], scope: EMPTY_SCOPE };

export function kindOf(cond: Condition): ConditionKind {
  return CONDITION_KINDS.find((k) => k in cond) ?? "GrahaInSign";
}

/** Yeni eklenen koşulun başlangıç değeri: kümeler boştur ve astrolog doldurur. */
export function defaultCondition(kind: ConditionKind): Condition {
  switch (kind) {
    case "GrahaInSign":
      return { GrahaInSign: { graha: "Sun", signs: [], varga: "D1" } };
    case "GrahaInHouse":
      return { GrahaInHouse: { graha: "Sun", houses: [] } };
    case "GrahaInNakshatra":
      return { GrahaInNakshatra: { graha: "Moon", nakshatras: [], padas: [] } };
    case "GrahaState":
      return { GrahaState: { graha: "Mars", flags: [], dignities: [] } };
    case "LagnaInSign":
      return { LagnaInSign: { signs: [], varga: "D1" } };
    case "Conjunction":
      return { Conjunction: { grahas: [] } };
    case "GrahaAspects":
      return { GrahaAspects: { from: "Saturn", to: "Moon" } };
    case "HouseLordInHouse":
      return { HouseLordInHouse: { lord_of: 1, houses: [] } };
    case "MoonPhase":
      return { MoonPhase: { tithis: [], paksha: null } };
    case "DashaAt":
      return { DashaAt: { date: null, level: 1, lords: [] } };
    case "CharaKaraka":
      return { CharaKaraka: { role: "Atma", grahas: [] } };
    case "ArudhaInSign":
      return { ArudhaInSign: { pada: 1, signs: [] } };
    case "YogaPresent":
      return { YogaPresent: { ids: [] } };
  }
}

/** "Örnek koşul": boş formun tek eyleminin eklediği, hemen aranabilen koşul. */
export const EXAMPLE_CONDITION: QueryCondition = { cond: { GrahaInSign: { graha: "Moon", signs: ["Cancer"], varga: "D1" } }, negate: false };

const DATE = /^\d{4}-\d{2}-\d{2}$/;

/**
 * Koşulun eksiği (i18n anahtarı `search.issue.<değer>`); tamamsa `null`. Yalnızca biçim
 * denetimidir: aramayı başlatmadan önce hangi satırın eksik olduğunu söyler. Asıl denetim
 * Rust'tadır (`ChartQuery::validate`).
 */
export function conditionIssue(cond: Condition): string | null {
  if (cond.GrahaInSign) return cond.GrahaInSign.signs.length === 0 ? "signs" : null;
  if (cond.GrahaInHouse) return cond.GrahaInHouse.houses.length === 0 ? "houses" : null;
  if (cond.GrahaInNakshatra) return cond.GrahaInNakshatra.nakshatras.length === 0 ? "nakshatras" : null;
  if (cond.GrahaState) return cond.GrahaState.flags.length + cond.GrahaState.dignities.length === 0 ? "states" : null;
  if (cond.LagnaInSign) return cond.LagnaInSign.signs.length === 0 ? "signs" : null;
  if (cond.Conjunction) return cond.Conjunction.grahas.length < 2 ? "twoGrahas" : null;
  if (cond.GrahaAspects) return cond.GrahaAspects.from === cond.GrahaAspects.to ? "sameGraha" : null;
  if (cond.HouseLordInHouse) return cond.HouseLordInHouse.houses.length === 0 ? "houses" : null;
  if (cond.MoonPhase) return cond.MoonPhase.tithis.length === 0 && cond.MoonPhase.paksha === null ? "phase" : null;
  if (cond.DashaAt) return cond.DashaAt.lords.length === 0 ? "lords" : cond.DashaAt.date !== null && !DATE.test(cond.DashaAt.date) ? "date" : null;
  if (cond.CharaKaraka) return cond.CharaKaraka.grahas.length === 0 ? "grahas" : null;
  if (cond.ArudhaInSign) return cond.ArudhaInSign.signs.length === 0 ? "signs" : null;
  if (cond.YogaPresent) return cond.YogaPresent.ids.length === 0 ? "yogas" : cond.YogaPresent.ids.length > MAX_YOGA_IDS ? "tooManyYogas" : null;
  return null;
}

/** Sorgunun aranmasını engelleyen ilk eksik: (satır sırası, anahtar); yoksa `null`. */
export function queryIssue(query: ChartQuery): { row: number; issue: string } | null {
  for (const [row, c] of query.all_of.entries()) {
    const issue = conditionIssue(c.cond);
    if (issue !== null) return { row, issue };
  }
  return null;
}

/** Sorgunun kimliği: sonuç listesi başlatıldığı sorguya aittir. */
export function queryKey(query: ChartQuery): string {
  return JSON.stringify(query);
}

/** Kümeye ekler ya da çıkarır; sonuç `order` sırasındadır. */
export function toggle<T>(set: readonly T[], value: T, order: readonly T[]): T[] {
  const next = set.includes(value) ? set.filter((v) => v !== value) : [...set, value];
  return order.filter((v) => next.includes(v));
}

/** Virgülle ayrılmış etiket metni → etiket listesi (kırpılmış, boşlar atılmış, en çok `MAX_SCOPE_TAGS`). */
export function parseTags(text: string): string[] {
  return text
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t !== "")
    .slice(0, MAX_SCOPE_TAGS);
}

/** Yoga kimlikleri, i18n kaynağındaki `yogas.name` ağacından (`grup.ad`), katalog sırasıyla. */
export function yogaIds(names: Record<string, Record<string, string>>): string[] {
  return Object.entries(names).flatMap(([group, rules]) => Object.keys(rules).map((rule) => `${group}.${rule}`));
}

const sign = (t: TFunction, s: Sign) => t(`sign.${signIndex(s)}`);
const graha = (t: TFunction, g: GrahaId) => t(`graha.${g}`);

/** Koşulun kısa yazımı ("Gü 1/10. ev"): "değil" olgusunun ve satır özetinin metni. */
export function conditionBrief(t: TFunction, cond: Condition): string {
  const signs = (list: readonly Sign[]) => list.map((s) => sign(t, s)).join("/");
  const grahas = (list: readonly GrahaId[]) => list.map((g) => graha(t, g)).join("/");
  const varga = (v: string) => (v === "D1" ? "" : ` ${v}`);
  if (cond.GrahaInSign) return `${graha(t, cond.GrahaInSign.graha)}${varga(cond.GrahaInSign.varga)} ${signs(cond.GrahaInSign.signs)}`;
  if (cond.GrahaInHouse) return t("search.brief.house", { what: graha(t, cond.GrahaInHouse.graha), houses: cond.GrahaInHouse.houses.join("/") });
  if (cond.GrahaInNakshatra) return `${graha(t, cond.GrahaInNakshatra.graha)} ${cond.GrahaInNakshatra.nakshatras.map(nakshatraName).join("/")}`;
  if (cond.GrahaState) {
    const states = [...cond.GrahaState.flags.map((f) => t(`search.flag.${f}`)), ...cond.GrahaState.dignities.map((d) => t(`dignity.${d}`))];
    return `${graha(t, cond.GrahaState.graha)} ${states.join("/")}`;
  }
  if (cond.LagnaInSign) return `${t("chart.lagna")}${varga(cond.LagnaInSign.varga)} ${signs(cond.LagnaInSign.signs)}`;
  if (cond.Conjunction) return cond.Conjunction.grahas.map((g) => graha(t, g)).join("+");
  if (cond.GrahaAspects) return `${graha(t, cond.GrahaAspects.from)} → ${graha(t, cond.GrahaAspects.to)}`;
  if (cond.HouseLordInHouse) return t("search.brief.house", { what: t("search.brief.lordOf", { house: cond.HouseLordInHouse.lord_of }), houses: cond.HouseLordInHouse.houses.join("/") });
  if (cond.MoonPhase) return [...cond.MoonPhase.tithis.map(tithiShort), ...(cond.MoonPhase.paksha ? [cond.MoonPhase.paksha] : [])].join("/");
  if (cond.DashaAt) return t("search.brief.dasha", { level: cond.DashaAt.level, lords: grahas(cond.DashaAt.lords) });
  if (cond.CharaKaraka) return `${KARAKA_ROLES[cond.CharaKaraka.role].short} ${grahas(cond.CharaKaraka.grahas)}`;
  if (cond.ArudhaInSign) return `${arudhaLabel(cond.ArudhaInSign.pada)} ${signs(cond.ArudhaInSign.signs)}`;
  if (cond.YogaPresent) return cond.YogaPresent.ids.map((id) => t(`yogas.name.${id}`)).join("/");
  return "";
}

/**
 * Eşleşen olgunun kısa özeti ("Ma Oğ 10. ev", "Jü R", "değil: Gü 1/10. ev"). Retro düz `R`'dir;
 * Rust düğümlerde retro işaretini hiç göndermez.
 */
export function factSummary(t: TFunction, cond: Condition, fact: MatchFact): string {
  if (fact.negated) return t("search.fact.not", { what: conditionBrief(t, cond) });
  const g = fact.grahas.map((x) => graha(t, x));
  const s = fact.sign === null ? "" : sign(t, fact.sign);
  const house = fact.house === null ? "" : ` ${t("search.fact.house", { house: fact.house })}`;
  const retro = fact.retrograde ? " R" : "";
  if (cond.GrahaInSign && cond.GrahaInSign.varga !== "D1") return `${g[0]} ${cond.GrahaInSign.varga} ${s}`;
  if (cond.GrahaInSign || cond.GrahaInHouse || cond.HouseLordInHouse) return `${g[0]} ${s}${house}${retro}`;
  if (cond.GrahaInNakshatra) return `${g[0]} ${fact.nakshatra === null ? "" : nakshatraName(fact.nakshatra)} ${fact.pada ?? ""}`.trimEnd();
  if (cond.GrahaState) {
    // Retro durumu `R` ile yazılır; yanık ve vargottama sözle, dignity kendi adıyla.
    const states = [...fact.flags.filter((f) => f !== "Retrograde").map((f) => t(`search.flag.${f}`)), ...(fact.dignity ? [t(`dignity.${fact.dignity}`)] : [])];
    return `${g[0]} ${s}${retro}${states.length > 0 ? ` ${states.join(", ")}` : ""}`;
  }
  if (cond.LagnaInSign) return `${t("chart.lagna")}${cond.LagnaInSign.varga === "D1" ? "" : ` ${cond.LagnaInSign.varga}`} ${s}`;
  if (cond.Conjunction) return `${g.join("+")} ${s}${house}`;
  if (cond.GrahaAspects) return `${g[0]} → ${g[1]}`;
  if (cond.MoonPhase) return fact.tithi === null ? "" : `${tithiShort(fact.tithi)} ${tithiName(fact.tithi)}`;
  if (cond.DashaAt) return g.join("–");
  if (cond.CharaKaraka) return `${KARAKA_ROLES[cond.CharaKaraka.role].short} ${g[0]}`;
  if (cond.ArudhaInSign) return `${arudhaLabel(cond.ArudhaInSign.pada)} ${s}`;
  if (cond.YogaPresent) return fact.yogas.map((id) => t(`yogas.name.${id}`)).join(", ");
  return "";
}

/** Bir sonucun özet satırı: koşul sırasıyla olgular, "; " ile. Sorguda karşılığı olmayan olgu atlanır. */
export function hitSummary(t: TFunction, query: ChartQuery, matched: readonly MatchFact[]): string {
  return matched
    .map((f) => {
      const c = query.all_of[f.condition];
      return c ? factSummary(t, c.cond, f) : "";
    })
    .filter((s) => s !== "")
    .join("; ");
}

export { GRAHA_IDS, SIGNS, VARGAS };
