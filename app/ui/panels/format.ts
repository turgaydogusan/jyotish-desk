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
