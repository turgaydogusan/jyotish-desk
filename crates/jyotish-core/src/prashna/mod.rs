//! Prashna (ileri modüller spec'i §11): soru anının haritası ve o haritanın olguları. Hesap
//! Parashari motorunun aynısıdır; ayrı bir prashna ekolünün kuralları, hüküm ve evet/hayır
//! sonucu yoktur.
//!
//! Bu dosya sayı prashnasının lagnasını verir: grahalar anın grahalarıdır, lagna 1–108
//! arasındaki sayıdan gelir (`number_lagna_lon`). Harita `compute_chart_with_lagna` ile kurulur.

use crate::CoreError;

/// Sayı prashnasının en büyük sayısı: 12 burç × 9 navamsa.
pub const PRASHNA_NUMBER_MAX: u8 = 108;

/// Bir navamsanın yay dakikası olarak genişliği (3°20′).
const NAVAMSA_ARCMIN: u16 = 200;

/// Sayı prashnasının lagnası: `n`. navamsanın ortası, sidereal boylam (derece):
/// `(n − 1) × 3°20′ + 1°40′`. Sayı 1..=108 değilse `InvalidPrashnaInput("number")`.
/// Yay dakikası üzerinden hesaplanır: aynı sayı her zaman aynı `f64` değerini verir.
pub fn number_lagna_lon(number: u8) -> Result<f64, CoreError> {
    if !(1..=PRASHNA_NUMBER_MAX).contains(&number) {
        return Err(CoreError::InvalidPrashnaInput("number"));
    }
    let arcmin = (u16::from(number) - 1) * NAVAMSA_ARCMIN + NAVAMSA_ARCMIN / 2;
    Ok(f64::from(arcmin) / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};

    /// Derece ve dakika olarak elle yazılmış beklenen lagnalar (spec §11 "Test").
    const HAND: [(u8, Sign, u16, u16); 5] = [
        (1, Sign::Aries, 1, 40),
        (9, Sign::Aries, 28, 20),
        (10, Sign::Taurus, 1, 40),
        (54, Sign::Virgo, 28, 20),
        (108, Sign::Pisces, 28, 20),
    ];

    #[test]
    fn the_number_gives_the_middle_of_its_navamsa() {
        for (n, sign, deg, min) in HAND {
            let lon = number_lagna_lon(n).unwrap();
            assert_eq!(Sign::of(lon), sign, "{n}");
            let want = f64::from(deg) + f64::from(min) / 60.0;
            assert!((deg_in_sign(lon) - want).abs() < 1e-9, "{n}");
        }
        assert_eq!(number_lagna_lon(1).unwrap(), 100.0 / 60.0);
        assert_eq!(number_lagna_lon(108).unwrap(), 21_500.0 / 60.0);
    }

    #[test]
    fn every_number_is_a_distinct_navamsa_in_order() {
        // 108 navamsa 27 nakshatranın 4 padasıdır: n. sayı, n. padanın ortasıdır.
        for n in 1..=PRASHNA_NUMBER_MAX {
            let lon = number_lagna_lon(n).unwrap();
            assert!((0.0..360.0).contains(&lon));
            let (nakshatra, pada) = nakshatra_pada(lon);
            assert_eq!((nakshatra - 1) * 4 + pada, n);
            // Burç başına dokuz sayı.
            assert_eq!(Sign::of(lon).index(), usize::from(n - 1) / 9);
        }
    }

    #[test]
    fn numbers_outside_the_range_are_rejected_by_field_name() {
        for n in [0, 109, 200, u8::MAX] {
            assert_eq!(
                number_lagna_lon(n),
                Err(CoreError::InvalidPrashnaInput("number"))
            );
        }
    }
}
