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
