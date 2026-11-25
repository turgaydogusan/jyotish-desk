// Doğum formu modeli (saf). Form metin alanlarını tutar; `toNewChart` biçim denetimi yapar
// (boş zorunlu alan, sayı olmayan koordinat, ofset biçimi). Anlam denetimi (tarih geçerli mi,
// DST boşluğu, aralıklar) Rust'tadır: `preview_moment` ve kayıt komutları `invalid_input`
// ve an hatalarıyla döner; `errorField` bunları form alanına bağlar.
import type { BirthData, ChartKind, CommandError, FoldChoice, Gender, NewChart, NewClient, OpenedClient, PlaceHit, TimeCertainty, TzChoice } from "../bindings";

export type TzMode = "iana" | "fixed" | "lmt";

export interface BirthDraft {
  name: string;
  gender: Gender | null;
  label: string;
  kind: ChartKind;
  date: string;
  /** "17:30"; saniye varsa "20:17:40". */
  time: string;
  certainty: TimeCertainty;
  placeName: string;
  country: string;
  admin1: string;
  geonameId: number | null;
  lat: string;
  lon: string;
  tzMode: TzMode;
  tzIana: string;
  /** "+05:30" biçimi; saniyeli ofsette "-04:56:16". */
  tzFixed: string;
  fold: FoldChoice | null;
  notes: string;
}

export type BirthField = "name" | "label" | "date" | "time" | "certainty" | "lat" | "lon" | "tz" | "fold";
/** Alan → i18n anahtarı (`birth.error.*`) ya da arka uç hatası. */
export type FieldErrors = Partial<Record<BirthField, { key: string } | { error: CommandError }>>;

export function emptyDraft(): BirthDraft {
  return {
    name: "",
    gender: null,
    label: "",
    kind: "Natal",
    date: "",
    time: "",
    certainty: "Exact",
    placeName: "",
    country: "",
    admin1: "",
    geonameId: null,
    lat: "",
    lon: "",
    tzMode: "iana",
    tzIana: "",
    tzFixed: "+00:00",
    fold: null,
    notes: "",
  };
}

/** "+05:30" / "-03:00" / "+5:45" / "-04:56:16" → saniye; biçim dışıysa `null`. */
export function parseFixedOffset(text: string): number | null {
  const m = /^([+\-−])(\d{1,2}):(\d{2})(?::(\d{2}))?$/.exec(text.trim());
  if (!m) return null;
  const [h, min, s] = [Number(m[2]), Number(m[3]), Number(m[4] ?? 0)];
  if (min >= 60 || s >= 60) return null;
  // `|| 0`: "-0:00" negatif sıfır değil sıfırdır (JSON'da ve karşılaştırmada fark yaratmasın).
  return (m[1] === "+" ? 1 : -1) * (h * 3600 + min * 60 + s) || 0;
}

/** Saniyeli ofset (JHD içe aktarımı, LMT önizlemesi) saniyesiyle yazılır; kırpılmaz. */
export function formatFixedOffset(secs: number): string {
  const a = Math.abs(secs);
  const two = (n: number) => String(n).padStart(2, "0");
  const s = a % 60;
  return `${secs < 0 ? "-" : "+"}${two(Math.floor(a / 3600))}:${two(Math.floor((a % 3600) / 60))}${s === 0 ? "" : `:${two(s)}`}`;
}

function parseCoord(text: string): number | null {
  const t = text.trim().replace(",", ".").replace("−", "-");
  if (!/^[+-]?\d+(\.\d+)?$/.test(t)) return null;
  // Çok uzun bir rakam dizisi `Infinity` olur ve JSON'da `null`'a döner; sayı sayılmaz.
  const n = Number(t);
  return Number.isFinite(n) ? n : null;
}

/**
 * Saklanan saat ("17:30:00", "20:17:40", "20:17:40.5") → form metni. Yalnızca sıfır saniye
 * atılır; sıfırdan farklı saniye ve kesir korunur, yoksa düzenleyip kaydetmek doğum anını
 * sessizce değiştirirdi.
 */
function formTime(time: string): string {
  return time.replace(/^(\d{2}:\d{2}):00(\.0+)?$/, "$1");
}

/** Saklanan harita → düzenleme taslağı (Cmd+E). */
export function draftFromOpened(o: OpenedClient): BirthDraft {
  const b = o.chart_record.birth;
  // `{ Fixed } & { Iana?: never }` biçiminde `in` daraltmaz; alan değerine bakılır.
  const iana = b.tz === "Lmt" ? undefined : b.tz.Iana;
  const fixed = b.tz === "Lmt" ? undefined : b.tz.Fixed;
  return {
    name: o.client.name,
    gender: o.client.gender,
    label: o.chart_record.label,
    kind: o.chart_record.kind,
    date: b.date,
    time: b.time ? formTime(b.time) : "",
    certainty: b.time_certainty,
    placeName: b.place.name,
    country: b.place.country,
    admin1: b.place.admin1,
    geonameId: b.place.geoname_id,
    lat: b.place.lat === null ? "" : String(b.place.lat),
    lon: b.place.lon === null ? "" : String(b.place.lon),
    tzMode: iana !== undefined ? "iana" : fixed !== undefined ? "fixed" : "lmt",
    tzIana: iana ?? "",
    tzFixed: fixed !== undefined ? formatFixedOffset(fixed) : "+00:00",
    fold: b.fold,
    notes: b.notes,
  };
}

/** Atlas sonucu: yer alanları ve önerilen IANA saat dilimi. */
export function applyPlace(d: BirthDraft, hit: PlaceHit): BirthDraft {
  return {
    ...d,
    placeName: hit.name,
    country: hit.country,
    admin1: hit.admin1,
    geonameId: hit.geoname_id,
    lat: String(hit.lat),
    lon: String(hit.lon),
    tzMode: "iana",
    tzIana: hit.timezone,
    fold: null,
  };
}

/** Elle koordinat: atlas bağı kopar (`geoname_id` boşalır), ad korunur. */
export function setCoordinate(d: BirthDraft, field: "lat" | "lon", value: string): BirthDraft {
  return { ...d, [field]: value, geonameId: null };
}

/** DST boşluğu önerisi ("2014-03-31T04:30:00") tarih ve saate yazılır; saniyesi korunur. */
export function applySuggestion(d: BirthDraft, suggestion: string): BirthDraft {
  const [date, time] = suggestion.split("T");
  return { ...d, date, time: formTime(time) };
}

function tzChoice(d: BirthDraft): TzChoice | null {
  if (d.tzMode === "lmt") return "Lmt";
  if (d.tzMode === "iana") return d.tzIana.trim() === "" ? null : { Iana: d.tzIana.trim() };
  const secs = parseFixedOffset(d.tzFixed);
  return secs === null ? null : { Fixed: secs };
}

/** Biçim denetimi: hatasızsa doğum verisi, değilse alan hataları. */
export function toBirthData(d: BirthDraft): { birth: BirthData; errors: null } | { birth: null; errors: FieldErrors } {
  const errors: FieldErrors = {};
  if (d.name.trim() === "") errors.name = { key: "birth.error.required" };
  if (d.date.trim() === "") errors.date = { key: "birth.error.required" };
  const unknown = d.certainty === "Unknown";
  if (!unknown && d.time.trim() === "") errors.time = { key: "birth.error.required" };
  const lat = parseCoord(d.lat);
  const lon = parseCoord(d.lon);
  if (lat === null) errors.lat = { key: "birth.error.number" };
  if (lon === null) errors.lon = { key: "birth.error.number" };
  const tz = tzChoice(d);
  if (tz === null) errors.tz = { key: d.tzMode === "fixed" ? "birth.error.offset" : "birth.error.required" };
  if (Object.keys(errors).length > 0 || lat === null || lon === null || tz === null) return { birth: null, errors };
  return {
    errors: null,
    birth: {
      date: d.date.trim(),
      time: unknown ? null : d.time.trim(),
      time_certainty: d.certainty,
      place: { name: d.placeName.trim(), country: d.country.trim(), admin1: d.admin1.trim(), lat, lon, geoname_id: d.geonameId },
      tz,
      fold: d.fold,
      notes: d.notes,
    },
  };
}

export function toNewChart(d: BirthDraft, birth: BirthData): NewChart {
  return { label: d.label.trim(), kind: d.kind, birth };
}

export function toNewClient(d: BirthDraft, birth: BirthData): NewClient {
  return { name: d.name.trim(), gender: d.gender, chart: toNewChart(d, birth) };
}

const INPUT_FIELDS: Record<string, BirthField> = {
  "birth.date": "date",
  "birth.time": "time",
  "birth.time_certainty": "certainty",
  "birth.place.lat": "lat",
  "birth.place.lon": "lon",
  name: "name",
  label: "label",
};

/** Arka uç hatası → form alanı; alana bağlanamayan hata `null` (form üstünde gösterilir). */
export function errorField(error: CommandError): BirthField | null {
  switch (error.code) {
    case "invalid_input":
      return INPUT_FIELDS[error.params.field] ?? null;
    case "nonexistent_local_time":
      return "time";
    case "ambiguous_local_time":
      return "fold";
    case "unknown_time_zone":
    case "offset_out_of_range":
      return "tz";
    case "longitude_out_of_range":
      return "lon";
    case "date_out_of_range":
      return "date";
    default:
      return null;
  }
}
