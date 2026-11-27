import { expect, test } from "vitest";
import type { HomePlace } from "../bindings";
import { fixture, unknownTimeOpened } from "../test/fixture";
import {
  chartPlace,
  EMPTY_CRITERIA,
  hitPlace,
  MAX_MIN_MINUTES,
  MUHURTA_FLAGS,
  muhurtaPlace,
  natalMoon,
  parseCriteria,
  parseHomePlace,
  placeChoice,
  placeLabel,
  resolvePlace,
  toggleMember,
  VARAS,
} from "./muhurta";

const FX = fixture();
const DELHI: HomePlace = FX.muhurta.home_place;

test("the place choice defaults to the current place and unknown values fall back to it", () => {
  expect(placeChoice({})).toBe("home");
  expect(placeChoice({ place: "chart" })).toBe("chart");
  expect(placeChoice({ place: "custom" })).toBe("custom");
  expect(placeChoice({ place: "moon" })).toBe("home");
  expect(placeChoice({ place: 3 })).toBe("home");
});

test("the place comes from the preferences, the open chart or the panel's own setting", () => {
  const opened = FX.opened;
  const prefs = { ...FX.prefs.prefs, home_place: DELHI };
  expect(chartPlace(opened)).toEqual({ place: opened.chart_record.birth.place, tz: { Iana: "Asia/Kolkata" } });
  expect(resolvePlace("chart", opened, null, null)).toEqual(chartPlace(opened));
  expect(resolvePlace("home", opened, prefs, null)).toEqual(DELHI);
  // Bulunulan yer seçilmemiş ya da tercihler okunamamış: yer yok (doğum yerine sessizce düşülmez).
  expect(resolvePlace("home", opened, FX.prefs.prefs, null)).toBeNull();
  expect(resolvePlace("home", opened, null, DELHI)).toBeNull();
  expect(resolvePlace("custom", opened, prefs, null)).toBeNull();
  const tokyo: HomePlace = { place: { name: "Tokyo", country: "Japan", admin1: "Tokyo", lat: 35.69, lon: 139.69, geoname_id: 1850147 }, tz: { Iana: "Asia/Tokyo" } };
  expect(resolvePlace("custom", opened, prefs, tokyo)).toEqual(tokyo);
  // Komuta yalnızca koordinat ve saat dilimi gider; ad gitmez.
  expect(muhurtaPlace(tokyo)).toEqual({ lat: 35.69, lon: 139.69, tz: { Iana: "Asia/Tokyo" } });
  expect(muhurtaPlace(DELHI)).toEqual(FX.muhurta.place);
});

test("a stored custom place is read tolerantly", () => {
  expect(parseHomePlace(DELHI)).toEqual(DELHI);
  expect(parseHomePlace(JSON.parse(JSON.stringify(DELHI)))).toEqual(DELHI);
  const manual = { place: { lat: -33.5, lon: 151.2 }, tz: { Fixed: 36000 }, extra: 1 };
  expect(parseHomePlace(manual)).toEqual({ place: { name: "", country: "", admin1: "", lat: -33.5, lon: 151.2, geoname_id: null }, tz: { Fixed: 36000 } });
  expect(parseHomePlace({ place: { lat: 1, lon: 2 }, tz: "Lmt" })?.tz).toBe("Lmt");
  for (const broken of [
    null,
    "Delhi",
    [],
    {},
    { place: DELHI.place },
    { place: { ...DELHI.place, lat: 91 }, tz: DELHI.tz },
    { place: { ...DELHI.place, lon: "77" }, tz: DELHI.tz },
    { place: { ...DELHI.place, lat: Number.NaN }, tz: DELHI.tz },
    { place: DELHI.place, tz: { Iana: " " } },
    { place: DELHI.place, tz: { Fixed: 1.5 } },
    { place: DELHI.place, tz: "Utc" },
  ]) {
    expect(parseHomePlace(broken), JSON.stringify(broken)).toBeNull();
  }
});

test("an atlas hit becomes a place with its zone, and a place is named by its parts or its coordinates", () => {
  const hit = FX.places[0];
  const place = hitPlace(hit);
  expect(place).toEqual({ place: { name: "New Delhi", country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: "Asia/Kolkata" } });
  expect(hitPlace({ ...hit, lat: null })).toBeNull();
  expect(placeLabel(DELHI)).toBe("New Delhi, Delhi, India");
  // Yinelenen parça bir kez yazılır; adı olmayan yer koordinatlarıyla anılır.
  expect(placeLabel({ ...DELHI, place: { ...DELHI.place, name: "İstanbul", admin1: "İstanbul", country: "Türkiye" } })).toBe("İstanbul, Türkiye");
  expect(placeLabel({ ...DELHI, place: { ...DELHI.place, name: " ", admin1: "", country: "" } })).toBe("28.61°, 77.21°");
});

test("the natal Moon is read from the computed chart, also when the birth time is unknown", () => {
  const moon = FX.opened.chart.grahas[1];
  expect(moon.graha).toBe("Moon");
  expect(natalMoon(FX.opened)).toEqual({ nakshatra: moon.nakshatra, sign: moon.sign });
  expect(natalMoon(FX.opened)).toEqual(FX.muhurta.natal);
  expect(natalMoon(unknownTimeOpened())).toEqual(FX.muhurta.natal);
});

test("stored criteria are read tolerantly into the canonical order", () => {
  expect(parseCriteria(undefined)).toEqual(EMPTY_CRITERIA);
  expect(parseCriteria("x")).toEqual(EMPTY_CRITERIA);
  expect(parseCriteria(FX.muhurta.query.criteria)).toEqual(FX.muhurta.query.criteria);
  expect(
    parseCriteria({
      weekdays: ["Friday", "Funday", "Sunday", "Friday"],
      tithis: [30, 2, 0, 31, 2.5, "3"],
      nakshatras: [27, 28, 1],
      lagna_signs: ["Leo", "Aries", "Ophiuchus"],
      daytime_only: "yes",
      min_minutes: 99999,
      exclude: ["Gulika", "Rikta", "Eclipse"],
      require_tara: true,
      require_chandra: 1,
      future: true,
    }),
  ).toEqual({
    weekdays: ["Sunday", "Friday"],
    tithis: [2, 30],
    nakshatras: [1, 27],
    lagna_signs: ["Aries", "Leo"],
    daytime_only: false,
    min_minutes: 10080,
    exclude: ["Rikta", "Gulika"],
    require_tara: true,
    require_chandra: false,
  });
  expect(parseCriteria({ min_minutes: -5 }).min_minutes).toBe(0);
  expect(parseCriteria({ min_minutes: 12.5 }).min_minutes).toBe(0);
  // Okunan nesne her çağrıda yenidir.
  expect(parseCriteria(null)).not.toBe(EMPTY_CRITERIA);
});

test("toggling a member keeps the set in the canonical order", () => {
  expect(toggleMember([], "Friday", VARAS)).toEqual(["Friday"]);
  expect(toggleMember(["Friday"], "Monday", VARAS)).toEqual(["Monday", "Friday"]);
  expect(toggleMember(["Monday", "Friday"], "Monday", VARAS)).toEqual(["Friday"]);
  expect(toggleMember([9, 2], 5, [1, 2, 3, 4, 5, 6, 7, 8, 9])).toEqual([2, 5, 9]);
});

test("the weekday and flag lists and the length limit are the Rust ones", () => {
  expect([...MUHURTA_FLAGS]).toEqual(FX.muhurta_ids.flags);
  expect(MAX_MIN_MINUTES).toBe(FX.muhurta_ids.max_min_minutes);
  expect(VARAS).toHaveLength(7);
  expect(VARAS[0]).toBe("Sunday");
});
