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

/** Günlük hız, derece/gün, üç ondalık: 12.0212 → "+12.021", -0.05 → "−0.050". */
export function formatSpeed(degPerDay: number): string {
  return `${degPerDay < 0 ? "−" : "+"}${Math.abs(degPerDay).toFixed(3)}`;
}
