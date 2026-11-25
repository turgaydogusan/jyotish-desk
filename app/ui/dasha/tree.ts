// Dasha ağacı yardımcıları. Dönemler Rust'tan hazır gelir (`Chart.dasha.periods`: 1–3.
// seviye, ön-sıralı; 4–5. seviye `dasha_children` ile). Burada hesap yoktur: yalnızca
// gruplama ve gösterim için JD → yerel tarih biçimlemesi.
import type { DashaPeriod, Graha } from "../bindings";

/** Spec §5: 1 mahadasha … 5 prana. */
export const MAX_DASHA_LEVEL = 5;
/** `Chart.dasha.periods`'un içerdiği en derin seviye; daha derini istenerek yüklenir. */
export const PRELOADED_LEVEL = 3;

function startsWith(lords: readonly Graha[], prefix: readonly Graha[]): boolean {
  return prefix.every((g, i) => lords[i] === g);
}

/** `path` döneminin doğrudan alt dönemleri (boş yol: mahadashalar), sırası korunur. */
export function childrenOf(periods: readonly DashaPeriod[], path: readonly Graha[]): DashaPeriod[] {
  return periods.filter((p) => p.level === path.length + 1 && startsWith(p.lords, path));
}

export function samePath(a: readonly Graha[], b: readonly Graha[]): boolean {
  return a.length === b.length && startsWith(a, b);
}

/** `path`, `open` yolunun bir öneki mi (o düğüm açık mı)? */
export function isOpen(open: readonly Graha[], path: readonly Graha[]): boolean {
  return path.length <= open.length && startsWith(open, path);
}

const UNIX_EPOCH_JD = 2440587.5;
const DAY_MS = 86_400_000;

/**
 * JD (UT) → doğumun sabit ofsetinde "YYYY-MM-DD" (yalnızca gösterim; spec §4'teki
 * "UI hesap yapmaz" kuralı astrolojik hesap içindir, takvim biçimlemesi değil).
 */
export function formatJdDate(jd: number | null, offsetSecs: number): string {
  if (jd === null || !Number.isFinite(jd)) return "—";
  const d = new Date((jd - UNIX_EPOCH_JD) * DAY_MS + offsetSecs * 1000);
  const y = d.getUTCFullYear();
  const year = y < 0 ? `−${String(-y).padStart(4, "0")}` : String(y).padStart(4, "0");
  return `${year}-${String(d.getUTCMonth() + 1).padStart(2, "0")}-${String(d.getUTCDate()).padStart(2, "0")}`;
}
