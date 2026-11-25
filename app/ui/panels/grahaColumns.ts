// Graha tablosu sütunları (spec §3.3 `graha_table.settings.columns`). Bilinmeyen sütun
// kimlikleri okunurken atlanır; ayar hiç yoksa ya da boşsa varsayılan sütunlar kullanılır.
export const GRAHA_COLUMNS = ["sign", "degree", "lon", "lat", "speed", "retro", "nakshatra", "pada", "combust", "dignity", "house"] as const;
export type GrahaColumn = (typeof GRAHA_COLUMNS)[number];

export const DEFAULT_GRAHA_COLUMNS: readonly GrahaColumn[] = ["sign", "degree", "nakshatra", "pada", "retro", "combust", "dignity", "house"];

function isGrahaColumn(v: unknown): v is GrahaColumn {
  return typeof v === "string" && (GRAHA_COLUMNS as readonly string[]).includes(v);
}

/** Ayardaki sütunlar, `GRAHA_COLUMNS` sırasında ve tekrarsız. */
export function grahaColumns(settings: Record<string, unknown>): GrahaColumn[] {
  const raw = settings.columns;
  if (!Array.isArray(raw)) return [...DEFAULT_GRAHA_COLUMNS];
  const chosen = new Set(raw.filter(isGrahaColumn));
  return chosen.size === 0 ? [...DEFAULT_GRAHA_COLUMNS] : GRAHA_COLUMNS.filter((c) => chosen.has(c));
}

/** Sütunu aç/kapat; son sütun kapatılamaz (boş tablo olmaz). */
export function toggleColumn(columns: readonly GrahaColumn[], column: GrahaColumn): GrahaColumn[] {
  if (columns.includes(column)) return columns.length === 1 ? [...columns] : columns.filter((c) => c !== column);
  return GRAHA_COLUMNS.filter((c) => c === column || columns.includes(c));
}
