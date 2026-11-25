//! Ashtakavarga: yedi grahanın bhinnashtakavargası (BAV) ve sarvashtakavarga (SAV)
//! (spec §3, §5).
//!
//! Kaynak: BPHS böl. 66 (R. Santhanam çevirisi); B. V. Raman, *Ashtakavarga System of
//! Prediction* ile aynı tablolar. Her hedef graha için sekiz referansın (yedi graha ve
//! lagna) her biri, kendi burcundan sayılan belirli evlere birer bindu verir; aynı burç
//! 1. evdir. Rahu ve Ketu katılmaz. Trikona/ekadhipatya shodhana kapsam dışıdır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::zodiac::Sign;

/// BAV satır sırası ve referans sırasının ilk yedisi.
pub const ASHTAKAVARGA_GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// BPHS toplamları: her BAV'ın bindu sayısı burç yerleşiminden bağımsızdır.
pub const BAV_TOTALS: [u8; 7] = [48, 49, 39, 54, 56, 52, 39];
pub const SAV_TOTAL: u16 = 337;

/// `BENEFIC_PLACES[hedef][referans]`: referansın burcundan sayılan bindu evleri.
/// Referans sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Lagna.
const BENEFIC_PLACES: [[&[u8]; 8]; 7] = [
    // Güneş
    [
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11, 12],
        &[5, 6, 9, 11],
        &[6, 7, 12],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ],
    // Ay
    [
        &[3, 6, 7, 8, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ],
    // Mars
    [
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 3, 6, 10, 11],
    ],
    // Merkür
    [
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ],
    // Jüpiter
    [
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ],
    // Venüs
    [
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 5, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ],
    // Satürn
    [
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[3, 5, 6, 11],
        &[1, 3, 4, 6, 10, 11],
    ],
];

/// Spec §5 `Ashtakavarga`. Burç sütunları Koç'tan (0) Balık'a (11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Ashtakavarga {
    /// `ASHTAKAVARGA_GRAHAS` sırasında yedi BAV.
    pub bav: [[u8; 12]; 7],
    /// Burç başına yedi BAV'ın toplamı; toplamı her zaman `SAV_TOTAL`.
    pub sav: [u8; 12],
}

/// `signs`: `ASHTAKAVARGA_GRAHAS` sırasında yedi grahanın burcu; `lagna`: lagna burcu.
/// İkisi de aynı (rasi) çerçeveden gelmelidir.
pub fn compute_ashtakavarga(signs: [Sign; 7], lagna: Sign) -> Ashtakavarga {
    let refs = [
        signs[0], signs[1], signs[2], signs[3], signs[4], signs[5], signs[6], lagna,
    ];
    let mut bav = [[0u8; 12]; 7];
    for (row, table) in bav.iter_mut().zip(&BENEFIC_PLACES) {
        for (r, places) in refs.iter().zip(table) {
            for &p in *places {
                // p 1..=12: referans burcunun kendisi 1. evdir.
                row[(r.index() + usize::from(p) + 11) % 12] += 1;
            }
        }
    }
    let mut sav = [0u8; 12];
    for row in &bav {
        for (s, v) in sav.iter_mut().zip(row) {
            *s += v;
        }
    }
    Ashtakavarga { bav, sav }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::Sign::*;

    #[test]
    fn tables_have_the_bphs_totals() {
        let mut all = 0u16;
        for (g, table) in BENEFIC_PLACES.iter().enumerate() {
            let n: usize = table.iter().map(|places| places.len()).sum();
            assert_eq!(
                n,
                usize::from(BAV_TOTALS[g]),
                "{:?}",
                ASHTAKAVARGA_GRAHAS[g]
            );
            for places in table {
                assert!(places.windows(2).all(|w| w[0] < w[1]), "{places:?}");
                assert!(places.iter().all(|p| (1..=12).contains(p)), "{places:?}");
            }
            all += u16::from(BAV_TOTALS[g]);
        }
        assert_eq!(all, SAV_TOTAL);
        assert_eq!(ASHTAKAVARGA_GRAHAS[..], Graha::ALL[..7]);
    }

    #[test]
    fn hand_checked_chart() {
        // FakeEph varsayılan haritası (sidereal): Güneş Balık, Ay İkizler, Mars Başak,
        // Merkür Balık, Jüpiter Akrep, Venüs Koç, Satürn Oğlak; lagna Boğa.
        let a = compute_ashtakavarga(
            [Pisces, Gemini, Virgo, Pisces, Scorpio, Aries, Capricorn],
            Taurus,
        );
        // Güneş BAV'ı elle, Koç sütunu: Güneş'ten (Balık) 2., Ay'dan (İkizler) 11., Mars'tan
        // (Başak) 8., Jüpiter'den (Akrep) 6., Satürn'den (Oğlak) 4., lagnadan (Boğa) 12. ev:
        // altısı da Güneş tablosunda → 6. Merkür'den (Balık) 2. ve Venüs'ten (Koç) 1. ev
        // tabloda yok. Balık sütunu: Güneş 1., Ay 10., Mars 7., Jüpiter 5., Venüs 12.,
        // lagna 11. → 6.
        assert_eq!(a.bav[0], [6, 2, 2, 5, 4, 5, 5, 4, 3, 3, 3, 6]);
        assert_eq!(a.bav[1], [1, 5, 6, 4, 4, 3, 6, 4, 4, 4, 4, 4]);
        assert_eq!(a.bav[2], [4, 3, 1, 5, 5, 4, 4, 3, 2, 3, 2, 3]);
        assert_eq!(a.bav[3], [5, 4, 4, 6, 5, 4, 4, 5, 4, 4, 5, 4]);
        assert_eq!(a.bav[4], [4, 5, 6, 3, 4, 5, 4, 4, 7, 5, 4, 5]);
        assert_eq!(a.bav[5], [3, 6, 4, 6, 7, 4, 3, 4, 2, 6, 4, 3]);
        assert_eq!(a.bav[6], [3, 2, 3, 2, 4, 3, 4, 4, 2, 3, 4, 5]);
        assert_eq!(a.sav, [26, 27, 26, 31, 33, 28, 30, 28, 24, 28, 26, 30]);
    }

    #[test]
    fn shifting_every_sign_rotates_the_tables() {
        let signs = [Pisces, Gemini, Virgo, Pisces, Scorpio, Aries, Capricorn];
        let base = compute_ashtakavarga(signs, Taurus);
        for k in 1..12 {
            let shifted = compute_ashtakavarga(
                signs.map(|s| Sign::from_index(s.index() + k)),
                Sign::from_index(Taurus.index() + k),
            );
            for s in 0..12 {
                let from = (s + 12 - k) % 12;
                assert_eq!(shifted.sav[s], base.sav[from], "k {k}, sign {s}");
                for g in 0..7 {
                    assert_eq!(shifted.bav[g][s], base.bav[g][from], "k {k}, graha {g}");
                }
            }
        }
    }
}
