import type { TzChoice } from "../bindings";
import { fixture } from "../test/fixture";
import {
  applyPlace,
  applySuggestion,
  draftFromOpened,
  emptyDraft,
  errorField,
  formatFixedOffset,
  parseFixedOffset,
  setCoordinate,
  toBirthData,
  toNewClient,
} from "./model";

const delhiDraft = () => ({ ...applyPlace({ ...emptyDraft(), name: " Ayşe ", date: "2000-01-01", time: "17:30" }, fixture().places[0]) });

test("fixed offsets parse and format both ways", () => {
  expect(parseFixedOffset("+05:30")).toBe(19_800);
  expect(parseFixedOffset("-3:00")).toBe(-10_800);
  expect(parseFixedOffset("−03:00")).toBe(-10_800);
  expect(parseFixedOffset("+5:75")).toBeNull();
  expect(parseFixedOffset("0530")).toBeNull();
  // İşaret zorunlu: işaretsiz ya da bozuk giriş ters işaretli bir ofsete dönmez.
  expect(parseFixedOffset("05:30")).toBeNull();
  expect(parseFixedOffset("105:30")).toBeNull();
  expect(parseFixedOffset("x5:30")).toBeNull();
  expect(formatFixedOffset(19_800)).toBe("+05:30");
  expect(formatFixedOffset(-34_200)).toBe("-09:30");
  // Saniyeli ofset iki yönde de korunur; kırpılmaz.
  expect(parseFixedOffset("-04:56:16")).toBe(-17_776);
  expect(parseFixedOffset("+05:30:60")).toBeNull();
  expect(formatFixedOffset(-17_776)).toBe("-04:56:16");
  expect(formatFixedOffset(18_530)).toBe("+05:08:50");
});

test("an atlas hit fills the place and suggests its IANA zone", () => {
  const d = delhiDraft();
  expect(d).toMatchObject({ placeName: "New Delhi", lat: "28.62137", tzMode: "iana", tzIana: "Asia/Kolkata", geonameId: 1261481 });
  const manual = setCoordinate(d, "lat", "28.7");
  expect(manual).toMatchObject({ lat: "28.7", geonameId: null, placeName: "New Delhi" });
});

test("a complete draft becomes NewClient; unknown time sends no time", () => {
  const { birth } = toBirthData(delhiDraft());
  expect(birth).toEqual({
    date: "2000-01-01",
    time: "17:30",
    time_certainty: "Exact",
    place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: 28.62137, lon: 77.2148, geoname_id: 1261481 },
    tz: { Iana: "Asia/Kolkata" },
    fold: null,
    notes: "",
  });
  expect(toNewClient(delhiDraft(), birth!)).toMatchObject({ name: "Ayşe", gender: null, chart: { label: "", kind: "Natal" } });
  const unknown = toBirthData({ ...delhiDraft(), time: "", certainty: "Unknown" });
  expect(unknown.birth).toMatchObject({ time: null, time_certainty: "Unknown" });
  expect(toBirthData({ ...delhiDraft(), tzMode: "fixed", tzFixed: "+05:30" }).birth!.tz).toEqual({ Fixed: 19_800 });
  expect(toBirthData({ ...delhiDraft(), tzMode: "lmt" }).birth!.tz).toBe("Lmt");
  expect(toBirthData({ ...delhiDraft(), lat: "28,5" }).birth!.place.lat).toBe(28.5);
  // Tipografik eksi ofsette olduğu gibi koordinatta da kabul edilir.
  expect(toBirthData({ ...delhiDraft(), lon: "−74.08" }).birth!.place.lon).toBe(-74.08);
});

test("format errors are reported per field before anything is sent", () => {
  const { errors } = toBirthData({ ...emptyDraft(), lat: "north", tzMode: "fixed", tzFixed: "5" });
  expect(errors).toEqual({
    name: { key: "birth.error.required" },
    date: { key: "birth.error.required" },
    time: { key: "birth.error.required" },
    lat: { key: "birth.error.number" },
    lon: { key: "birth.error.number" },
    tz: { key: "birth.error.offset" },
  });
  // Sonlu olmayan sayı (`Infinity` JSON'da `null` olur) arka uca gitmez.
  expect(toBirthData({ ...delhiDraft(), lat: "9".repeat(400) }).errors).toEqual({ lat: { key: "birth.error.number" } });
});

test("backend errors map to fields", () => {
  const e = (code: string, params: Record<string, string> = {}) => ({ code, params });
  expect(errorField(e("invalid_input", { field: "birth.date" }))).toBe("date");
  expect(errorField(e("invalid_input", { field: "birth.place.lat" }))).toBe("lat");
  expect(errorField(e("invalid_input", { field: "name" }))).toBe("name");
  expect(errorField(e("invalid_input", { field: "tags" }))).toBeNull();
  expect(errorField(fixture().gap_error)).toBe("time");
  expect(errorField(e("ambiguous_local_time"))).toBe("fold");
  expect(errorField(e("unknown_time_zone"))).toBe("tz");
  expect(errorField(e("store_sqlite"))).toBeNull();
});

test("the DST suggestion and editing a stored chart", () => {
  expect(applySuggestion(delhiDraft(), fixture().gap_error.params.suggestion)).toMatchObject({ date: "2014-03-31", time: "04:30" });
  const d = draftFromOpened(fixture().opened);
  expect(d).toMatchObject({ name: "Örnek Danışan", date: "2000-01-01", time: "17:30", tzMode: "iana", tzIana: "Asia/Kolkata", kind: "Natal" });
  expect(toBirthData(d).birth!.place).toEqual(fixture().opened.chart_record.birth.place);
});

test("editing a stored chart keeps the seconds of its time and of its fixed offset", () => {
  const stored = (time: string, tz: TzChoice) => {
    const o = fixture().opened;
    o.chart_record.birth = { ...o.chart_record.birth, time, tz };
    return o;
  };
  // JHD içe aktarımı saniye yazar ("20:17:40"); ofset de saniyeli olabilir (−04:56:16).
  const d = draftFromOpened(stored("20:17:40", { Fixed: -17_776 }));
  expect(d).toMatchObject({ time: "20:17:40", tzMode: "fixed", tzFixed: "-04:56:16" });
  expect(toBirthData(d).birth).toMatchObject({ time: "20:17:40", tz: { Fixed: -17_776 } });
  expect(draftFromOpened(stored("20:17:40.5", "Lmt"))).toMatchObject({ time: "20:17:40.5", tzMode: "lmt" });
  expect(draftFromOpened(stored("17:30:00", "Lmt")).time).toBe("17:30");
  // DST önerisi girilen saatin saniyesini taşır.
  expect(applySuggestion(delhiDraft(), "2014-03-31T04:30:40")).toMatchObject({ date: "2014-03-31", time: "04:30:40" });
});

test("a signed zero offset is zero, not negative zero", () => {
  expect(Object.is(parseFixedOffset("-0:00"), 0)).toBe(true);
  expect(Object.is(parseFixedOffset("+00:00"), 0)).toBe(true);
  expect(Object.is(parseFixedOffset("−00:00:00"), 0)).toBe(true);
});

test("errorField maps every field-bound backend code and nothing else", () => {
  const e = (code: string, params: Record<string, string> = {}) => ({ code, params });
  const table: [ReturnType<typeof e>, string | null][] = [
    [e("invalid_input", { field: "birth.date" }), "date"],
    [e("invalid_input", { field: "birth.time" }), "time"],
    [e("invalid_input", { field: "birth.time_certainty" }), "certainty"],
    [e("invalid_input", { field: "birth.place.lat" }), "lat"],
    [e("invalid_input", { field: "birth.place.lon" }), "lon"],
    [e("invalid_input", { field: "name" }), "name"],
    [e("invalid_input", { field: "label" }), "label"],
    [e("invalid_input", { field: "notes" }), null],
    [e("nonexistent_local_time", { local: "x", suggestion: "y" }), "time"],
    [e("ambiguous_local_time"), "fold"],
    [e("unknown_time_zone"), "tz"],
    [e("offset_out_of_range"), "tz"],
    [e("longitude_out_of_range"), "lon"],
    [e("date_out_of_range"), "date"],
    [e("atlas_unavailable", { reason: "missing" }), null],
    [e("task_failed"), null],
  ];
  for (const [error, field] of table) expect([error.code, errorField(error)]).toEqual([error.code, field]);
});
