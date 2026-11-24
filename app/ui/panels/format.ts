/** Burç içi dereceyi derece ve yay dakikasıyla yazar: 16.5 → "16°30′" (dakika aşağı yuvarlanır). */
export function formatDegree(deg: number): string {
  const totalMinutes = Math.floor(deg * 60 + 1e-9);
  const d = Math.floor(totalMinutes / 60);
  const m = totalMinutes % 60;
  return `${d}°${String(m).padStart(2, "0")}′`;
}
