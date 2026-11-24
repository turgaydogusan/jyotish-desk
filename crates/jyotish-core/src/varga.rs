//! Shodashavarga: BPHS böl. 6'nın bölümlü haritaları (Parashara tanımları).
//!
//! Her varga, rasi çerçevesindeki boylamı (burç `s`, burç içi derece `d`) bir varga burcuna
//! ve varga içi dereceye eşler. Varga içi derece, grahanın düştüğü parçadaki konumunun
//! 30°'ye ölçeklenmiş hâlidir: parça `[a, b)` ise `(d − a) / (b − a) × 30`. Eşit parçalı
//! vargalarda bu `(d × N) mod 30`'dur. Parça sınırı sonraki parçaya aittir.

use serde::{Deserialize, Serialize};

use crate::zodiac::{deg_in_sign, Sign};

const CANCER: usize = Sign::Cancer as usize;
const LEO: usize = Sign::Leo as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Varga {
    /// Rasi.
    D1,
    /// Hora (Parashara).
    D2,
    /// Drekkana (Parashara).
    D3,
    /// Chaturthamsa.
    D4,
    /// Saptamsa.
    D7,
    /// Navamsa.
    D9,
    /// Dasamsa.
    D10,
    /// Dwadasamsa.
    D12,
}

impl Varga {
    pub const ALL: [Varga; 8] = [
        Varga::D1,
        Varga::D2,
        Varga::D3,
        Varga::D4,
        Varga::D7,
        Varga::D9,
        Varga::D10,
        Varga::D12,
    ];

    /// Bir burcun kaç parçaya bölündüğü (N).
    pub fn divisions(self) -> u8 {
        match self {
            Self::D1 => 1,
            Self::D2 => 2,
            Self::D3 => 3,
            Self::D4 => 4,
            Self::D7 => 7,
            Self::D9 => 9,
            Self::D10 => 10,
            Self::D12 => 12,
        }
    }
}

/// Tek (oja) burç: Koç, İkizler, Aslan, Terazi, Yay, Kova.
fn is_odd_sign(s: Sign) -> bool {
    s.index().is_multiple_of(2)
}

/// Hareketli, sabit ve çift doğalı burçlar için ayrı başlangıç.
fn by_modality(s: Sign, movable: usize, fixed: usize, dual: usize) -> usize {
    match s.index() % 3 {
        0 => movable,
        1 => fixed,
        _ => dual,
    }
}

/// Eşit parçalı bölüm: `d`'nin düştüğü parça (0..n) ve parça içi derece, [0, 30).
fn equal_part(d: f64, n: u8) -> (usize, f64) {
    let scaled = d * f64::from(n);
    let last = usize::from(n.max(1)) - 1;
    let part = ((scaled / 30.0) as usize).min(last);
    // Yuvarlama 30'a ya da sıfırın altına taşırmasın.
    let within = (scaled - part as f64 * 30.0).clamp(0.0, 30.0_f64.next_down());
    (part, within)
}

/// `lon` (rasi çerçevesi) boylamının `varga`'daki burcu ve varga içi derecesi.
pub fn varga_position(varga: Varga, lon: f64) -> (Sign, f64) {
    let s = Sign::of(lon);
    let d = deg_in_sign(lon);
    let i = s.index();
    let odd = is_odd_sign(s);
    let (part, within) = equal_part(d, varga.divisions());
    let sign = match varga {
        Varga::D1 => i,
        Varga::D2 => {
            if (part == 0) == odd {
                LEO
            } else {
                CANCER
            }
        }
        Varga::D3 => i + 4 * part,
        Varga::D4 => i + 3 * part,
        Varga::D7 => (if odd { i } else { i + 6 }) + part,
        Varga::D9 => by_modality(s, i, i + 8, i + 4) + part,
        Varga::D10 => (if odd { i } else { i + 8 }) + part,
        Varga::D12 => i + part,
    };
    (Sign::from_index(sign), within)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cases: &[(Varga, f64, Sign)]) {
        for &(v, lon, want) in cases {
            assert_eq!(varga_position(v, lon).0, want, "{v:?} at {lon}°");
        }
    }

    fn deg(v: Varga, lon: f64) -> f64 {
        varga_position(v, lon).1
    }

    #[test]
    fn all_in_order_with_divisions() {
        let n: Vec<u8> = Varga::ALL.iter().map(|v| v.divisions()).collect();
        assert_eq!(n, [1, 2, 3, 4, 7, 9, 10, 12]);
    }

    #[test]
    fn lower_vargas_follow_parashari_rules() {
        use Sign::*;
        use Varga::*;
        check(&[
            // D1: burcun kendisi.
            (D1, 0.0, Aries),
            (D1, 29.999, Aries),
            (D1, 30.0, Taurus),
            (D1, 359.99, Pisces),
            // D2: tek burç Aslan → Yengeç, çift burç Yengeç → Aslan.
            (D2, 0.0, Leo),
            (D2, 14.999, Leo),
            (D2, 15.0, Cancer),
            (D2, 30.0, Cancer),
            (D2, 45.0, Leo),
            (D2, 359.0, Leo),
            // D3: kendisi, 5., 9.
            (D3, 0.0, Aries),
            (D3, 9.999, Aries),
            (D3, 10.0, Leo),
            (D3, 20.0, Sagittarius),
            (D3, 40.0, Virgo),
            (D3, 355.0, Scorpio),
            // D4: kendisi, 4., 7., 10.
            (D4, 0.0, Aries),
            (D4, 7.5, Cancer),
            (D4, 15.0, Libra),
            (D4, 22.5, Capricorn),
            (D4, 37.5, Leo),
            // D7: tek burç kendisinden, çift burç 7.'den. 30/7 = 4.2857°.
            (D7, 4.28, Aries),
            (D7, 4.29, Taurus),
            (D7, 29.0, Libra),
            (D7, 30.0, Scorpio),
            (D7, 59.9, Taurus),
            // D9: hareketli kendisinden, sabit 9.'dan, çift doğalı 5.'den.
            (D9, 0.0, Aries),
            (D9, 3.33, Aries),
            (D9, 3.34, Taurus),
            (D9, 30.0, Capricorn),
            (D9, 60.0, Libra),
            (D9, 90.0, Cancer),
            (D9, 120.0, Aries),
            (D9, 359.9, Pisces),
            // D10: tek burç kendisinden, çift burç 9.'dan.
            (D10, 0.0, Aries),
            (D10, 3.0, Taurus),
            (D10, 30.0, Capricorn),
            (D10, 59.0, Libra),
            // D12: kendisinden.
            (D12, 0.0, Aries),
            (D12, 2.5, Taurus),
            (D12, 29.9, Pisces),
            (D12, 120.0, Leo),
        ]);
    }

    #[test]
    fn degree_in_varga_is_position_in_part_scaled_to_30() {
        assert!((deg(Varga::D1, 46.5) - 16.5).abs() < 1e-9);
        assert!((deg(Varga::D2, 20.0) - 10.0).abs() < 1e-9); // 20 × 2 = 40 → 40 − 30
        assert!((deg(Varga::D3, 25.0) - 15.0).abs() < 1e-9); // 25 × 3 = 75 → 75 − 60
        assert!((deg(Varga::D9, 1.0) - 9.0).abs() < 1e-9);
        assert!((deg(Varga::D12, 31.25) - 15.0).abs() < 1e-9); // Boğa 1.25 × 12
    }

    #[test]
    fn top_of_sign_stays_in_last_part() {
        let top = 30.0_f64.next_down();
        for v in Varga::ALL {
            let (sign, d) = varga_position(v, top);
            assert_eq!(sign, varga_position(v, 29.999_999).0, "{v:?}");
            assert!((0.0..30.0).contains(&d), "{v:?}: {d}");
        }
    }

    #[test]
    fn serde_uses_division_names() {
        assert_eq!(serde_json::to_string(&Varga::D9).unwrap(), r#""D9""#);
        assert_eq!(
            serde_json::from_str::<Varga>(r#""D12""#).unwrap(),
            Varga::D12
        );
    }
}
