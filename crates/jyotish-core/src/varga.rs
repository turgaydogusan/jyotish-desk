//! Shodashavarga: BPHS böl. 6'nın bölümlü haritaları (Parashara tanımları).
//!
//! Her varga, rasi çerçevesindeki boylamı (burç `s`, burç içi derece `d`) bir varga burcuna
//! ve varga içi dereceye eşler. Varga içi derece, grahanın düştüğü parçadaki konumunun
//! 30°'ye ölçeklenmiş hâlidir: parça `[a, b)` ise `(d − a) / (b − a) × 30`. Eşit parçalı
//! vargalarda bu `(d × N) mod 30`'dur. Parça sınırı sonraki parçaya aittir.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;

use crate::zodiac::{deg_in_sign, Sign};

const ARIES: usize = Sign::Aries as usize;
const CANCER: usize = Sign::Cancer as usize;
const LEO: usize = Sign::Leo as usize;
const LIBRA: usize = Sign::Libra as usize;
const SAGITTARIUS: usize = Sign::Sagittarius as usize;
const CAPRICORN: usize = Sign::Capricorn as usize;

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
    /// Shodasamsa.
    D16,
    /// Vimsamsa.
    D20,
    /// Chaturvimsamsa (Siddhamsa).
    D24,
    /// Saptavimsamsa (Bhamsa).
    D27,
    /// Trimsamsa (eşit olmayan parçalar).
    D30,
    /// Khavedamsa.
    D40,
    /// Akshavedamsa.
    D45,
    /// Shashtyamsa.
    D60,
}

impl Varga {
    /// BPHS'nin Shodashavarga sırası.
    pub const ALL: [Varga; 16] = [
        Varga::D1,
        Varga::D2,
        Varga::D3,
        Varga::D4,
        Varga::D7,
        Varga::D9,
        Varga::D10,
        Varga::D12,
        Varga::D16,
        Varga::D20,
        Varga::D24,
        Varga::D27,
        Varga::D30,
        Varga::D40,
        Varga::D45,
        Varga::D60,
    ];

    /// Bir burcun kaç parçaya bölündüğü (N). D30'da parçalar eşit değildir.
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
            Self::D16 => 16,
            Self::D20 => 20,
            Self::D24 => 24,
            Self::D27 => 27,
            Self::D30 => 30,
            Self::D40 => 40,
            Self::D45 => 45,
            Self::D60 => 60,
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

/// D27 başlangıcı: ateş Koç, toprak Yengeç, hava Terazi, su Oğlak.
fn by_element(s: Sign) -> usize {
    match s.index() % 4 {
        0 => ARIES,
        1 => CANCER,
        2 => LIBRA,
        _ => CAPRICORN,
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

/// Trimsamsa, tek burç: (parça bitişi, varga burcu). Mars Koç, Satürn Kova, Jüpiter Yay,
/// Merkür İkizler, Venüs Terazi.
const TRIMSAMSA_ODD: [(f64, Sign); 5] = [
    (5.0, Sign::Aries),
    (10.0, Sign::Aquarius),
    (18.0, Sign::Sagittarius),
    (25.0, Sign::Gemini),
    (30.0, Sign::Libra),
];

/// Trimsamsa, çift burç (ters sıra): Venüs Boğa, Merkür Başak, Jüpiter Balık, Satürn Oğlak,
/// Mars Akrep.
const TRIMSAMSA_EVEN: [(f64, Sign); 5] = [
    (5.0, Sign::Taurus),
    (12.0, Sign::Virgo),
    (20.0, Sign::Pisces),
    (25.0, Sign::Capricorn),
    (30.0, Sign::Scorpio),
];

fn trimsamsa(d: f64, odd: bool) -> (Sign, f64) {
    let table = if odd { &TRIMSAMSA_ODD } else { &TRIMSAMSA_EVEN };
    let mut from = 0.0;
    for &(to, sign) in table {
        if d < to {
            let within = ((d - from) / (to - from) * 30.0).clamp(0.0, 30.0_f64.next_down());
            return (sign, within);
        }
        from = to;
    }
    // `d` her zaman 30'dan küçüktür; NaN gibi durumlar son parçaya düşer.
    (table[4].1, 30.0_f64.next_down())
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
        Varga::D16 => by_modality(s, ARIES, LEO, SAGITTARIUS) + part,
        Varga::D20 => by_modality(s, ARIES, SAGITTARIUS, LEO) + part,
        Varga::D24 => (if odd { LEO } else { CANCER }) + part,
        Varga::D27 => by_element(s) + part,
        Varga::D30 => return trimsamsa(d, odd),
        Varga::D40 => (if odd { ARIES } else { LIBRA }) + part,
        Varga::D45 => by_modality(s, ARIES, LEO, SAGITTARIUS) + part,
        Varga::D60 => i + part,
    };
    (Sign::from_index(sign), within)
}

/// Bir grahanın varga burcu ve varga içi derecesi (spec §5 `(graha, sign, deg)`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VargaPlacement {
    pub graha: Graha,
    pub sign: Sign,
    /// Varga içi derece, [0, 30): parçadaki konum 30°'ye ölçeklenmiş (modül belgesi).
    pub deg: f64,
}

/// Bir varga haritası (spec §5 `VargaChart`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VargaChart {
    pub varga: Varga,
    pub lagna_sign: Sign,
    /// Girdi sırasında; haritada `Graha::ALL` sırası.
    pub placements: Vec<VargaPlacement>,
}

/// `lagna_lon` ve `grahas` (rasi çerçevesi boylamları) için tek bir varga haritası.
pub fn varga_chart(varga: Varga, lagna_lon: f64, grahas: &[(Graha, f64)]) -> VargaChart {
    VargaChart {
        varga,
        lagna_sign: varga_position(varga, lagna_lon).0,
        placements: grahas
            .iter()
            .map(|&(graha, lon)| {
                let (sign, deg) = varga_position(varga, lon);
                VargaPlacement { graha, sign, deg }
            })
            .collect(),
    }
}

/// 16 varganın tamamı, `Varga::ALL` sırasında.
pub fn all_vargas(lagna_lon: f64, grahas: &[(Graha, f64)]) -> Vec<VargaChart> {
    Varga::ALL
        .iter()
        .map(|&v| varga_chart(v, lagna_lon, grahas))
        .collect()
}

/// Vargottama: D1 ve D9 burcu aynı.
pub fn is_vargottama(lon: f64) -> bool {
    varga_position(Varga::D1, lon).0 == varga_position(Varga::D9, lon).0
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
        assert_eq!(
            n,
            [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60]
        );
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

    #[test]
    fn upper_vargas_follow_parashari_rules() {
        use Sign::*;
        use Varga::*;
        check(&[
            (D16, 0.0, Aries),
            (D16, 1.875, Taurus),
            (D16, 30.0, Leo),
            (D16, 60.0, Sagittarius),
            (D16, 29.99, Cancer),
            (D20, 0.0, Aries),
            (D20, 1.5, Taurus),
            (D20, 30.0, Sagittarius),
            (D20, 60.0, Leo),
            (D20, 61.5, Virgo),
            (D24, 0.0, Leo),
            (D24, 1.25, Virgo),
            (D24, 30.0, Cancer),
            (D24, 29.99, Cancer),
            (D27, 0.0, Aries),
            (D27, 1.12, Taurus),
            (D27, 30.0, Cancer),
            (D27, 60.0, Libra),
            (D27, 90.0, Capricorn),
            (D27, 120.0, Aries),
            (D30, 0.0, Aries),
            (D30, 4.999, Aries),
            (D30, 5.0, Aquarius),
            (D30, 10.0, Sagittarius),
            (D30, 17.999, Sagittarius),
            (D30, 18.0, Gemini),
            (D30, 25.0, Libra),
            (D30, 29.99, Libra),
            (D30, 30.0, Taurus),
            (D30, 34.999, Taurus),
            (D30, 35.0, Virgo),
            (D30, 42.0, Pisces),
            (D30, 50.0, Capricorn),
            (D30, 55.0, Scorpio),
            (D30, 59.99, Scorpio),
            (D30, 138.0, Gemini),
            (D40, 0.0, Aries),
            (D40, 0.75, Taurus),
            (D40, 30.0, Libra),
            (D40, 30.75, Scorpio),
            (D40, 29.9, Cancer),
            (D45, 0.0, Aries),
            (D45, 30.0, Leo),
            (D45, 60.0, Sagittarius),
            (D45, 29.9, Sagittarius),
            (D60, 0.0, Aries),
            (D60, 0.5, Taurus),
            (D60, 30.0, Taurus),
            (D60, 59.9, Aries),
            (D60, 359.99, Aquarius),
        ]);
    }

    #[test]
    fn trimsamsa_degree_is_proportional_to_its_unequal_part() {
        assert!((deg(Varga::D30, 14.0) - 15.0).abs() < 1e-9);
        assert!((deg(Varga::D30, 38.5) - 15.0).abs() < 1e-9);
        assert!((deg(Varga::D60, 0.25) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn varga_chart_places_every_graha_in_order() {
        let grahas = [(Graha::Sun, 346.0), (Graha::Moon, 71.0)];
        let c = varga_chart(Varga::D9, 46.0, &grahas);
        assert_eq!(c.varga, Varga::D9);
        // Lagna Boğa 16°: floor(46 × 9 / 30) = 13 → Boğa.
        assert_eq!(c.lagna_sign, Sign::Taurus);
        assert_eq!(c.placements.len(), 2);
        assert_eq!(
            (c.placements[0].graha, c.placements[0].sign),
            (Graha::Sun, Sign::Scorpio)
        );
        assert_eq!(
            (c.placements[1].graha, c.placements[1].sign),
            (Graha::Moon, Sign::Capricorn)
        );
        // Güneş Balık 16°: 16 × 9 = 144 → 4. parça, parça içi 144 − 120 = 24°.
        assert!((c.placements[0].deg - 24.0).abs() < 1e-9);
        let all = all_vargas(46.0, &grahas);
        assert_eq!(all.iter().map(|v| v.varga).collect::<Vec<_>>(), Varga::ALL);
    }

    #[test]
    fn vargottama_compares_d1_and_d9() {
        assert!(is_vargottama(0.0)); // Koç 0°: D9 Koç
        assert!(is_vargottama(226.0)); // Akrep 16°: floor(67.8) = 67 → Akrep
        assert!(is_vargottama(359.9)); // Balık'ın son navamsası Balık
        assert!(!is_vargottama(346.0)); // Balık 16°: D9 Akrep
    }
}
