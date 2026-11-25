// Hesap ayarlarının arayüz modeli (spec §4 "Ayar katmanları"). Ayarların anlamı ve doğrulaması
// Rust'tadır (`CalcSettings::validate`); burada seçenek listeleri, alan karşılaştırması ve metin
// girişinin sayıya çevrilmesi vardır. Hesap yoktur (C4).
import type { CalcSettings, CombustionOrbs } from "../bindings";

export const ZODIACS = ["Sidereal", "TropicalRasiSiderealNakshatra"] as const satisfies readonly CalcSettings["zodiac"][];
/** Seçilebilir ayanamsalar; `Custom` yalnızca gösterilir (Plan 11 D11). */
export const PRESET_AYANAMSAS = ["Lahiri", "Raman", "Kp", "TrueChitra"] as const;
export const NODES = ["Mean", "True"] as const satisfies readonly CalcSettings["node"][];
export const HOUSE_SYSTEMS = ["WholeSign", "Sripati", "Placidus"] as const satisfies readonly CalcSettings["house_system"][];
export const SUNRISE_DEFS = ["DiskCenterNoRefraction", "UpperLimbRefracted"] as const satisfies readonly CalcSettings["sunrise_def"][];
/** Seçilebilir Vimshottari yıl uzunlukları (gün): Julian yılı, savana yılı, tropik yıl. */
export const DASHA_YEARS = [365.25, 360, 365.2422] as const;
export const ORB_KEYS = ["moon", "mars", "mercury", "mercury_retro", "jupiter", "venus", "venus_retro", "saturn"] as const satisfies readonly (keyof CombustionOrbs)[];
export type OrbKey = (typeof ORB_KEYS)[number];

/** Karşılaştırılan ve gösterilen alanlar, pencere sırasıyla (`schema_version` hariç). */
export const SETTINGS_FIELDS = ["zodiac", "ayanamsa", "node", "house_system", "dasha_year", "combustion_orbs", "sunrise_def"] as const satisfies readonly (keyof CalcSettings)[];
export type SettingsField = (typeof SETTINGS_FIELDS)[number];

/** Yapısal eşitlik; nesne anahtarlarının sırası önemsizdir. */
export function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  const ka = Object.keys(a);
  const kb = Object.keys(b);
  return ka.length === kb.length && ka.every((k) => k in b && sameValue((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]));
}

/** `a`'nın `b`'den farklı alanları (fark rozeti, Plan 11 D12). */
export function settingsDiff(a: CalcSettings, b: CalcSettings): SettingsField[] {
  return SETTINGS_FIELDS.filter((f) => !sameValue(a[f], b[f]));
}

export function ayanamsaKey(a: CalcSettings["ayanamsa"]): (typeof PRESET_AYANAMSAS)[number] | "Custom" {
  return typeof a === "string" ? a : "Custom";
}

/** Arayüzün kabul ettiği en büyük yanma orbu (derece); kesin aralık Rust'ta denetlenir. */
export const MAX_ORB_DEG = 30;

/** Orb metni → derece. Ondalık ayırıcı nokta ya da virgül olabilir. */
export function parseOrb(text: string): number | null {
  const s = text.trim().replace(",", ".");
  if (!/^\d+(\.\d+)?$/.test(s)) return null;
  const v = Number(s);
  return v <= MAX_ORB_DEG ? v : null;
}
