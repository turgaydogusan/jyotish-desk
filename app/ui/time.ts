/** Unix milisaniyesini UT Jülyen gününe çevirir (1970-01-01T00:00Z = JD 2440587.5). */
export function unixMsToJdUt(ms: number): number {
  return ms / 86_400_000 + 2440587.5;
}
