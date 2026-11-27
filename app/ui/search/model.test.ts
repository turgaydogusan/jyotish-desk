import i18next from "i18next";
import { beforeAll, expect, test } from "vitest";
import type { Condition, MatchFact } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { fixture } from "../test/fixture";
import {
  CONDITION_KINDS,
  conditionBrief,
  conditionIssue,
  defaultCondition,
  EMPTY_QUERY,
  EVENT_KINDS,
  EXAMPLE_CONDITION,
  factSummary,
  GRAHA_FLAGS,
  hitSummary,
  KARAKA_ROLE_IDS,
  kindOf,
  MAX_CONDITIONS,
  MAX_DASHA_LEVEL,
  MAX_EVENTS,
  MAX_SCOPE_TAGS,
  MAX_YOGA_IDS,
  parseTags,
  queryIssue,
  queryKey,
  toggle,
  VARGAS,
  yogaIds,
} from "./model";

const i18n = i18next.createInstance();
beforeAll(() => i18n.init({ lng: "tr", resources: { tr: { translation: tr } }, interpolation: { escapeValue: false } }));
const t = i18n.t.bind(i18n);

const FACT: MatchFact = { condition: 0, negated: false, grahas: [], sign: null, house: null, nakshatra: null, pada: null, retrograde: false, flags: [], dignity: null, tithi: null, yogas: [] };

test("the lists and limits are the ones Rust uses", () => {
  const ids = fixture().search_ids;
  expect([...EVENT_KINDS]).toEqual(ids.event_kinds);
  expect([...GRAHA_FLAGS]).toEqual(ids.flags);
  expect([...VARGAS]).toEqual(ids.vargas);
  expect([...KARAKA_ROLE_IDS]).toEqual(ids.karaka_roles);
  expect([MAX_CONDITIONS, MAX_SCOPE_TAGS, MAX_YOGA_IDS, MAX_DASHA_LEVEL, MAX_EVENTS]).toEqual([ids.max_conditions, ids.max_scope_tags, ids.max_yoga_ids, ids.max_dasha_level, ids.max_events]);
  // Yoga seçicisinin listesi katalogla aynı kimlikleri, aynı sırayla taşır (üç dilde).
  const catalog = fixture().yoga_catalog.map((y) => y.id);
  for (const lang of [tr, en, fr]) expect(yogaIds(lang.yogas.name)).toEqual(catalog);
});

test("a new condition of every kind is that kind and names what is missing", () => {
  expect(CONDITION_KINDS).toHaveLength(13);
  for (const kind of CONDITION_KINDS) {
    const cond = defaultCondition(kind);
    expect(kindOf(cond)).toBe(kind);
    // Yalnızca bakış koşulu kümesiz kurulur; ötekiler boş kümeyle başlar ve eksiğini söyler.
    const issue = conditionIssue(cond);
    if (kind === "GrahaAspects") expect(issue).toBeNull();
    else expect(tr.search.issue).toHaveProperty(issue ?? "");
    expect(tr.search.kind).toHaveProperty(kind);
  }
  expect(conditionIssue(EXAMPLE_CONDITION.cond)).toBeNull();
  expect(conditionIssue({ GrahaAspects: { from: "Mars", to: "Mars" } })).toBe("sameGraha");
  expect(conditionIssue({ Conjunction: { grahas: ["Sun"] } })).toBe("twoGrahas");
  expect(conditionIssue({ MoonPhase: { tithis: [], paksha: "Shukla" } })).toBeNull();
  expect(conditionIssue({ DashaAt: { date: "27.11.2026", level: 1, lords: ["Venus"] } })).toBe("date");
  expect(conditionIssue({ DashaAt: { date: "2026-11-27", level: 1, lords: ["Venus"] } })).toBeNull();
  expect(conditionIssue({ YogaPresent: { ids: Array.from({ length: 17 }, (_, i) => `x.${i}`) } })).toBe("tooManyYogas");
});

test("the first incomplete row blocks the query and the key follows the query", () => {
  expect(queryIssue(EMPTY_QUERY)).toBeNull();
  const query = { ...EMPTY_QUERY, all_of: [EXAMPLE_CONDITION, { cond: defaultCondition("GrahaInHouse"), negate: true }] };
  expect(queryIssue(query)).toEqual({ row: 1, issue: "houses" });
  expect(queryKey(query)).toBe(queryKey(structuredClone(query)));
  expect(queryKey(query)).not.toBe(queryKey({ ...query, scope: { ...query.scope, exclude_prashna: false } }));
});

test("sets toggle in list order and tags are trimmed and capped", () => {
  expect(toggle([3, 1], 2, [1, 2, 3])).toEqual([1, 2, 3]);
  expect(toggle([1, 2, 3], 2, [1, 2, 3])).toEqual([1, 3]);
  expect(parseTags(" kariyer , ,evlilik,")).toEqual(["kariyer", "evlilik"]);
  expect(parseTags(Array.from({ length: 20 }, (_, i) => `e${i}`).join(","))).toHaveLength(MAX_SCOPE_TAGS);
});

test("the fixture hit reads as a short line of facts", () => {
  const { query, page } = fixture().search;
  const hit = page.hits[0];
  // Ay Terazi'de ve 5. evde; ikinci koşul "değil" ile sağlanmıştır.
  expect(hitSummary(t, query, hit.matched)).toBe("Ay Ter 5. ev; değil: Gü 1/10. ev");
});

test("every kind of fact has a summary and retro is a plain R", () => {
  const cases: [Condition, Partial<MatchFact>, string][] = [
    [{ GrahaInSign: { graha: "Mars", signs: ["Capricorn"], varga: "D1" } }, { grahas: ["Mars"], sign: "Capricorn", house: 10 }, "Ma Oğl 10. ev"],
    [{ GrahaInSign: { graha: "Jupiter", signs: ["Cancer"], varga: "D1" } }, { grahas: ["Jupiter"], sign: "Cancer", retrograde: true }, "Jü Yen R"],
    [{ GrahaInSign: { graha: "Sun", signs: ["Cancer"], varga: "D9" } }, { grahas: ["Sun"], sign: "Cancer" }, "Gü D9 Yen"],
    [{ GrahaInHouse: { graha: "Moon", houses: [4] } }, { grahas: ["Moon"], sign: "Cancer", house: 4 }, "Ay Yen 4. ev"],
    [{ GrahaInNakshatra: { graha: "Moon", nakshatras: [8], padas: [] } }, { grahas: ["Moon"], nakshatra: 8, pada: 1 }, "Ay Pushya 1"],
    [
      { GrahaState: { graha: "Jupiter", flags: ["Retrograde", "Combust"], dignities: ["Exalted"] } },
      { grahas: ["Jupiter"], sign: "Cancer", retrograde: true, flags: ["Retrograde", "Combust"], dignity: "Exalted" },
      "Jü Yen R yanık, Yüce",
    ],
    [{ LagnaInSign: { signs: ["Taurus"], varga: "D9" } }, { sign: "Taurus" }, "Lg D9 Boğ"],
    [{ Conjunction: { grahas: ["Moon", "Jupiter"] } }, { grahas: ["Moon", "Jupiter"], sign: "Cancer", house: 4 }, "Ay+Jü Yen 4. ev"],
    [{ GrahaAspects: { from: "Mars", to: "Sun" } }, { grahas: ["Mars", "Sun"] }, "Ma → Gü"],
    [{ HouseLordInHouse: { lord_of: 10, houses: [7] } }, { grahas: ["Saturn"], sign: "Libra", house: 7 }, "Sa Ter 7. ev"],
    [{ MoonPhase: { tithis: [8], paksha: null } }, { tithi: 8 }, "S8 Ashtami"],
    [{ DashaAt: { date: null, level: 2, lords: ["Mercury"] } }, { grahas: ["Saturn", "Mercury"] }, "Sa–Me"],
    [{ CharaKaraka: { role: "Atma", grahas: ["Mars"] } }, { grahas: ["Mars"] }, "AK Ma"],
    [{ ArudhaInSign: { pada: 1, signs: ["Cancer"] } }, { sign: "Cancer" }, "AL Yen"],
    [{ YogaPresent: { ids: ["mahapurusha.ruchaka"] } }, { yogas: ["mahapurusha.ruchaka"] }, "Ruchaka"],
  ];
  expect(new Set(cases.map(([c]) => kindOf(c)))).toEqual(new Set(CONDITION_KINDS));
  for (const [cond, fact, text] of cases) {
    expect(factSummary(t, cond, { ...FACT, ...fact })).toBe(text);
    // "Değil" olgusu koşulun kısa yazımını taşır; boş değildir.
    expect(factSummary(t, cond, { ...FACT, negated: true })).toBe(`değil: ${conditionBrief(t, cond)}`);
    expect(conditionBrief(t, cond)).not.toBe("");
  }
  // Sorguda karşılığı olmayan olgu (bozuk sıra) atlanır.
  expect(hitSummary(t, EMPTY_QUERY, [{ ...FACT, condition: 5 }])).toBe("");
});
