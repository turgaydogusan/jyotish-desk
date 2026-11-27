// Nakshatra lordu (yol haritası Plan 26, §4.7). Tablo Rust'ındır (`jyotish_core::NAKSHATRA_LORDS`)
// ve bağlama dosyasıyla sabit olarak gelir: burada yalnızca satır seçilir, lord türetilmez (C4).
import { NAKSHATRA_LORDS, type Graha } from "../bindings";

/** n. nakshatranın (1..=27) lordu; aralık dışı ya da tam olmayan numara `null`. */
export function nakshatraLord(n: number | null): Graha | null {
  if (n === null || !Number.isInteger(n) || n < 1 || n > NAKSHATRA_LORDS.length) return null;
  return NAKSHATRA_LORDS[n - 1];
}
