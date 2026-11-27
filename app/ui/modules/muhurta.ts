// Muhurta panellerinin ortak girdileri (ileri modüller spec'i §10): hesabın yapılacağı yer, doğum
// Ay'ı ve süzgeç koşulları. Burada hesap yoktur (C4): değerler açık haritadan, tercihlerden ve
// panel ayarlarından okunur. Yer ve doğum Ay'ı kişisel veridir: loglanmaz.
import type { AppPrefs, HomePlace, MuhurtaCriteria, MuhurtaFlag, MuhurtaPlace, NatalMoon, OpenedClient, PlaceHit, Sign, TzChoice, Vara } from "../bindings";
import { SIGNS } from "../chart/adapter";

/** Yer seçimi: bulunulan yer (tercihlerden), açık haritanın doğum yeri ya da panele özel bir yer. */
export const PLACE_CHOICES = ["home", "chart", "custom"] as const;
export type PlaceChoice = (typeof PLACE_CHOICES)[number];

/** Rust `Vara::ALL`. */
export const VARAS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"] as const satisfies readonly Vara[];
/** Rust `MuhurtaFlag::ALL`. */
export const MUHURTA_FLAGS = [
  "Rikta",
  "Vishti",
  "Vyatipata",
  "Vaidhriti",
  "RahuSegment",
  "Yamaganda",
  "Gulika",
  "Panchaka",
  "JupiterCombust",
  "VenusCombust",
] as const satisfies readonly MuhurtaFlag[];
/** Rust `MUHURTA_MAX_MIN_MINUTES`: "en kısa süre" alanının üst sınırı. */
export const MAX_MIN_MINUTES = 10_080;

/** Panelin yer seçimi; tanınmayan değer ve eksik anahtar bulunulan yerdir. */
export function placeChoice(settings: Record<string, unknown>): PlaceChoice {
  return (PLACE_CHOICES as readonly unknown[]).includes(settings.place) ? (settings.place as PlaceChoice) : "home";
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function inRange(v: unknown, limit: number): v is number {
  return typeof v === "number" && Number.isFinite(v) && Math.abs(v) <= limit;
}

function parseTz(raw: unknown): TzChoice | null {
  if (raw === "Lmt") return "Lmt";
  if (!isRecord(raw)) return null;
  if (typeof raw.Iana === "string" && raw.Iana.trim() !== "") return { Iana: raw.Iana };
  if (typeof raw.Fixed === "number" && Number.isInteger(raw.Fixed)) return { Fixed: raw.Fixed };
  return null;
}

/** Panel ayarında saklanan özel yerin toleranslı okuması; okunamıyorsa `null`. */
export function parseHomePlace(raw: unknown): HomePlace | null {
  if (!isRecord(raw) || !isRecord(raw.place)) return null;
  const { name, country, admin1, lat, lon, geoname_id } = raw.place;
  const tz = parseTz(raw.tz);
  if (tz === null || !inRange(lat, 90) || !inRange(lon, 180)) return null;
  const text = (v: unknown) => (typeof v === "string" ? v : "");
  return {
    place: { name: text(name), country: text(country), admin1: text(admin1), lat, lon, geoname_id: typeof geoname_id === "number" ? geoname_id : null },
    tz,
  };
}

/** Atlas sonucu → yer; koordinatı olmayan sonuç `null`. */
export function hitPlace(hit: PlaceHit): HomePlace | null {
  if (hit.lat === null || hit.lon === null) return null;
  return {
    place: { name: hit.name, country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id },
    tz: { Iana: hit.timezone },
  };
}

/** Açık haritanın doğum yeri ve doğum verisindeki saat dilimi. */
export function chartPlace(opened: OpenedClient): HomePlace {
  const { place, tz } = opened.chart_record.birth;
  return { place, tz };
}

/** Seçime göre yer; bulunulan yer seçilmemişse ya da özel yer girilmemişse `null`. */
export function resolvePlace(choice: PlaceChoice, opened: OpenedClient, prefs: AppPrefs | null, custom: HomePlace | null): HomePlace | null {
  if (choice === "chart") return chartPlace(opened);
  if (choice === "custom") return custom;
  return prefs?.home_place ?? null;
}

/** Komuta giden yer: koordinat ve saat dilimi; ad gönderilmez. */
export function muhurtaPlace(home: HomePlace): MuhurtaPlace {
  return { lat: home.place.lat, lon: home.place.lon, tz: home.tz };
}

/** Yerin görünen adı; adı yoksa (elle girilen koordinat) koordinatları. */
export function placeLabel(home: HomePlace): string {
  const { name, admin1, country, lat, lon } = home.place;
  const parts = [name, admin1, country].map((s) => s.trim()).filter((s, i, all) => s !== "" && all.indexOf(s) === i);
  if (parts.length > 0) return parts.join(", ");
  return `${(lat ?? 0).toFixed(2)}°, ${(lon ?? 0).toFixed(2)}°`;
}

/** Doğum Ay'ı: açık haritanın Ay'ının nakshatrası ve rasi burcu (hesaplanmış haritadan okunur). */
export function natalMoon(opened: OpenedClient): NatalMoon {
  const moon = opened.chart.grahas.find((g) => g.graha === "Moon") ?? opened.chart.grahas[1];
  return { nakshatra: moon.nakshatra, sign: moon.sign };
}

export const EMPTY_CRITERIA: MuhurtaCriteria = {
  weekdays: [],
  tithis: [],
  nakshatras: [],
  lagna_signs: [],
  daytime_only: false,
  min_minutes: 0,
  exclude: [],
  require_tara: false,
  require_chandra: false,
};

function members<T>(raw: unknown, allowed: readonly T[]): T[] {
  if (!Array.isArray(raw)) return [];
  return allowed.filter((a) => raw.includes(a));
}

function numbers(raw: unknown, max: number): number[] {
  if (!Array.isArray(raw)) return [];
  const out: number[] = [];
  for (let n = 1; n <= max; n++) if (raw.includes(n)) out.push(n);
  return out;
}

/**
 * Panel ayarında saklanan koşulların toleranslı okuması: tanınmayan değerler atılır, kümeler
 * Rust'ın kanonik sırasına girer, eksik alanlar "kısıt yok"tur.
 */
export function parseCriteria(raw: unknown): MuhurtaCriteria {
  if (!isRecord(raw)) return { ...EMPTY_CRITERIA };
  const minutes = raw.min_minutes;
  return {
    weekdays: members<Vara>(raw.weekdays, VARAS),
    tithis: numbers(raw.tithis, 30),
    nakshatras: numbers(raw.nakshatras, 27),
    lagna_signs: members<Sign>(raw.lagna_signs, SIGNS),
    daytime_only: raw.daytime_only === true,
    min_minutes: typeof minutes === "number" && Number.isInteger(minutes) && minutes >= 0 ? Math.min(minutes, MAX_MIN_MINUTES) : 0,
    exclude: members<MuhurtaFlag>(raw.exclude, MUHURTA_FLAGS),
    require_tara: raw.require_tara === true,
    require_chandra: raw.require_chandra === true,
  };
}

/** Kümede `value` varsa çıkarır, yoksa ekler; sonuç `order` sırasındadır. */
export function toggleMember<T>(list: readonly T[], value: T, order: readonly T[]): T[] {
  const next = list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
  return order.filter((o) => next.includes(o));
}
