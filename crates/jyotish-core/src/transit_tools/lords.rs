//! Nakshatra lordları tablosu (yol haritası §4.7).
//!
//! Yirmi yedi nakshatranın Vimshottari lordu, nakshatra sırasıyla: Ashwini Ketu, Bharani Venüs,
//! Krittika Güneş … dokuzlu sıra üç kez yinelenir. Tablo, arayüze bağlama dosyasıyla sabit
//! olarak çıkar: geçiş tablosu, graha tablosu ve olay araması nakshatra numarasının lordunu
//! buradan okur (C4: tablo Rust'ındır, arayüz yalnızca satırı seçer). Hesap yapan Rust kodu
//! `natal_extras::nakshatra_lord`'u çağırmayı sürdürür; testler ikisinin aynı olduğunu sabitler.

use crate::graha::Graha;

/// `NAKSHATRA_LORDS[n - 1]`, n. nakshatranın (1..=27) lordudur.
pub const NAKSHATRA_LORDS: [Graha; 27] = [
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::{birth_dasha, VIMSHOTTARI_ORDER};
    use crate::natal_extras::nakshatra_lord;
    use crate::zodiac::NAKSHATRA_SPAN_DEG;

    #[test]
    fn the_table_is_the_vimshottari_order_three_times() {
        assert_eq!(NAKSHATRA_LORDS.len(), 27);
        for (i, lord) in NAKSHATRA_LORDS.iter().enumerate() {
            assert_eq!(*lord, VIMSHOTTARI_ORDER[i % 9], "nakshatra {}", i + 1);
        }
        // Her graha üç nakshatranın lordudur.
        for graha in Graha::ALL {
            let n = NAKSHATRA_LORDS.iter().filter(|l| **l == graha).count();
            assert_eq!(n, 3, "{graha:?}");
        }
    }

    #[test]
    fn hand_examples() {
        // 1 Ashwini Ketu; 4 Rohini Ay; 8 Pushya Satürn; 10 Magha Ketu; 14 Chitra Mars;
        // 18 Jyeshtha Merkür; 19 Mula Ketu; 22 Shravana Ay; 27 Revati Merkür.
        let expected = [
            (1, Graha::Ketu),
            (4, Graha::Moon),
            (8, Graha::Saturn),
            (10, Graha::Ketu),
            (14, Graha::Mars),
            (18, Graha::Mercury),
            (19, Graha::Ketu),
            (22, Graha::Moon),
            (27, Graha::Mercury),
        ];
        for (nakshatra, lord) in expected {
            assert_eq!(
                NAKSHATRA_LORDS[nakshatra - 1],
                lord,
                "nakshatra {nakshatra}"
            );
        }
    }

    #[test]
    fn the_table_agrees_with_the_function_and_the_dasha_seed() {
        for n in 1..=27_u8 {
            let lord = NAKSHATRA_LORDS[usize::from(n) - 1];
            // İkinci yazım: hesap yapan kodun kullandığı fonksiyon.
            assert_eq!(lord, nakshatra_lord(n), "nakshatra {n}");
            // Üçüncü yazım: nakshatranın ortasındaki bir Ay'ın doğum dasası.
            let mid = (f64::from(n) - 0.5) * NAKSHATRA_SPAN_DEG;
            assert_eq!(lord, birth_dasha(mid).0, "nakshatra {n}");
        }
    }
}
