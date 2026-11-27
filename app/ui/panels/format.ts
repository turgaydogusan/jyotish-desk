/** Burç içi dereceyi derece ve yay dakikasıyla yazar: 16.5 → "16°30′" (dakika aşağı yuvarlanır). */
export function formatDegree(deg: number): string {
  const totalMinutes = Math.floor(deg * 60 + 1e-9);
  const d = Math.floor(totalMinutes / 60);
  const m = totalMinutes % 60;
  return `${d}°${String(m).padStart(2, "0")}′`;
}

/** İşaretli derece (enlem): 5.17 → "+5°10′", -0.5 → "−0°30′". */
export function formatSignedDegree(deg: number): string {
  return `${deg < 0 ? "−" : "+"}${formatDegree(Math.abs(deg))}`;
}

const UNIX_EPOCH_JD = 2440587.5;
const DAY_MS = 86_400_000;
const MINUTE_MS = 60_000;

/**
 * JD (UT) → sabit ofsette "YYYY-AA-GG SS:DD" (dakika aşağı yuvarlanır). Yalnızca gösterim:
 * "arayüz hesap yapmaz" kuralı astrolojik hesap içindir, takvim biçimlemesi değil.
 */
export function formatJdMinute(jd: number | null, offsetSecs: number): string {
  if (jd === null || !Number.isFinite(jd)) return "—";
  const ms = Math.floor(((jd - UNIX_EPOCH_JD) * DAY_MS + offsetSecs * 1000) / MINUTE_MS) * MINUTE_MS;
  const d = new Date(ms);
  if (Number.isNaN(d.getTime())) return "—";
  const two = (n: number) => String(n).padStart(2, "0");
  const y = d.getUTCFullYear();
  const year = y < 0 ? `−${String(-y).padStart(4, "0")}` : String(y).padStart(4, "0");
  return `${year}-${two(d.getUTCMonth() + 1)}-${two(d.getUTCDate())} ${two(d.getUTCHours())}:${two(d.getUTCMinutes())}`;
}

/** Günlük hız, derece/gün, üç ondalık: 12.0212 → "+12.021", -0.05 → "−0.050". */
export function formatSpeed(degPerDay: number): string {
  return `${degPerDay < 0 ? "−" : "+"}${Math.abs(degPerDay).toFixed(3)}`;
}

/**
 * JD (UT) → sabit ofsette saat, "SS:DD" (dakika aşağı yuvarlanır). An, `baseJd`'nin o ofsetteki
 * takvim gününde değilse gün farkı eklenir: "06:57 (+1)", "23:10 (−1)". Yalnızca gösterim.
 */
export function formatJdClock(jd: number | null, offsetSecs: number, baseJd: number | null): string {
  const full = formatJdMinute(jd, offsetSecs);
  if (full === "—" || jd === null) return full;
  const clock = full.slice(-5);
  if (baseJd === null || !Number.isFinite(baseJd)) return clock;
  const localDay = (x: number) => Math.floor((x - UNIX_EPOCH_JD) + offsetSecs / 86_400);
  const days = localDay(jd) - localDay(baseJd);
  if (days === 0) return clock;
  return `${clock} (${days > 0 ? "+" : "−"}${Math.abs(days)})`;
}

/** JD (UT) → sabit ofsette takvim günü, "YYYY-AA-GG". */
export function formatJdDate(jd: number | null, offsetSecs: number): string {
  const full = formatJdMinute(jd, offsetSecs);
  return full === "—" ? full : full.slice(0, -6);
}

/** İki an arasındaki süre: "2 sa 05 dk", "45 dk"; bir günden uzunsa "3 g 4 sa". Dakika aşağı yuvarlanır. */
export function formatJdSpan(startJd: number | null, endJd: number | null, units: { day: string; hour: string; minute: string }): string {
  if (startJd === null || endJd === null || !Number.isFinite(startJd) || !Number.isFinite(endJd) || endJd < startJd) return "—";
  const minutes = Math.floor((endJd - startJd) * 1440 + 1e-6);
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);
  const rest = minutes % 60;
  if (days > 0) return `${days} ${units.day} ${hours} ${units.hour}`;
  if (hours > 0) return `${hours} ${units.hour} ${String(rest).padStart(2, "0")} ${units.minute}`;
  return `${rest} ${units.minute}`;
}
