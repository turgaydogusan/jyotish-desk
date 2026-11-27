//! Shayanadi avasthaları: on iki durum, tek bir sayma formülünden.
//!
//! Bu dosyadaki okuma (varyant noktaları `tests/golden/modules/avastha/REVIEW.md`'dedir):
//! `toplam = N × G × A + J + Gh + L`. Durum, toplamın 12'ye bölümünden artanın gösterdiği
//! durumdur; artan 0 ise on ikinci durumdur.
//!
//! - `N`: grahanın nakshatrası, Ashwini'den 1..=27 (sidereal boylamdan).
//! - `G`: grahanın sayısı: Güneş 1, Ay 2, Mars 3, Merkür 4, Jüpiter 5, Venüs 6, Satürn 7.
//! - `A`: grahanın bulunduğu navamsa diliminin burç içindeki sırası, 1..=9 (rasi çerçevesindeki
//!   burç içi dereceden; dilim 3°20′).
//! - `J`: doğum nakshatrası (Ay'ın nakshatrası), 1..=27.
//! - `Gh`: gün doğumundan doğuma **tamamlanmış** ghati sayısı (bir ghati 24 dakikadır).
//! - `L`: lagnanın rasi burcunun sırası, Koç 1 … Balık 12.
//!
//! Lagna ya da gün doğumu yoksa durum hesaplanmaz. Üç alt durum (cheshta) kapsam dışıdır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::nakshatra_pada;

/// Bir günde ghati sayısı: bir ghati 24 dakikadır.
pub const GHATIS_PER_DAY: f64 = 60.0;

/// Navamsa diliminin genişliği, derece (3°20′).
pub const NAVAMSA_SPAN_DEG: f64 = 30.0 / 9.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Shayanadi {
    Shayana,
    Upaveshana,
    Netrapani,
    Prakashana,
    Gamana,
    Agamana,
    Sabha,
    Agama,
    Bhojana,
    NrityaLipsa,
    Kautuka,
    Nidra,
}

impl Shayanadi {
    /// Artan sırası: artan 1 `Shayana`, …, artan 11 `Kautuka`, artan 0 `Nidra`.
    pub const ALL: [Shayanadi; 12] = [
        Shayanadi::Shayana,
        Shayanadi::Upaveshana,
        Shayanadi::Netrapani,
        Shayanadi::Prakashana,
        Shayanadi::Gamana,
        Shayanadi::Agamana,
        Shayanadi::Sabha,
        Shayanadi::Agama,
        Shayanadi::Bhojana,
        Shayanadi::NrityaLipsa,
        Shayanadi::Kautuka,
        Shayanadi::Nidra,
    ];
}

/// Formülün haritaya ait üç terimi: her graha için aynıdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShayanadiBasis {
    /// Doğum nakshatrası (Ay), 1..=27.
    pub birth_nakshatra: u8,
    /// Gün doğumundan doğuma tamamlanmış ghati.
    pub ghatis: u32,
    /// Lagnanın rasi burcunun sırası, 1..=12.
    pub lagna_sign_number: u8,
}

/// Bir grahanın Shayanadi durumu ve formülün o grahaya ait terimleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShayanadiHit {
    pub state: Shayanadi,
    /// Grahanın nakshatrası, 1..=27.
    pub nakshatra: u8,
    /// Grahanın sayısı, 1..=7.
    pub graha_number: u8,
    /// Navamsa diliminin burç içindeki sırası, 1..=9.
    pub navamsa: u8,
    /// `N × G × A + J + Gh + L`.
    pub total: u32,
    /// Toplamın 12'ye bölümünden artan, 0..=11; 0 on ikinci durumdur.
    pub remainder: u8,
}

/// Grahanın formüldeki sayısı; düğümler için `None`.
pub fn graha_number(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Sun => Some(1),
        Graha::Moon => Some(2),
        Graha::Mars => Some(3),
        Graha::Mercury => Some(4),
        Graha::Jupiter => Some(5),
        Graha::Venus => Some(6),
        Graha::Saturn => Some(7),
        Graha::Rahu | Graha::Ketu => None,
    }
}

/// Burç içi derecenin navamsa dilimi, 1..=9. Dilim sınırı sonraki dilime aittir.
pub fn navamsa_number(deg_in_sign: f64) -> u8 {
    (deg_in_sign / NAVAMSA_SPAN_DEG).floor().clamp(0.0, 8.0) as u8 + 1
}

/// Gün doğumundan `jd_ut` anına tamamlanmış ghati sayısı. An gün doğumundan önceyse ya da
/// değerlerden biri sonlu değilse `None`.
pub fn elapsed_ghatis(jd_ut: f64, sunrise_jd_ut: f64) -> Option<u32> {
    let ghatis = ((jd_ut - sunrise_jd_ut) * GHATIS_PER_DAY).floor();
    // Bir günden çok uzun aralık bir gün doğumu değildir (kutup); çağıran `None` verir.
    (ghatis.is_finite() && (0.0..=f64::from(u16::MAX)).contains(&ghatis)).then_some(ghatis as u32)
}

/// Formülün toplamı.
pub fn shayanadi_total(
    nakshatra: u8,
    graha_number: u8,
    navamsa: u8,
    basis: &ShayanadiBasis,
) -> u32 {
    u32::from(nakshatra) * u32::from(graha_number) * u32::from(navamsa)
        + u32::from(basis.birth_nakshatra)
        + basis.ghatis
        + u32::from(basis.lagna_sign_number)
}

/// Toplamın durumu: 12'ye bölümden artan 1..=11 ilk on bir durum, artan 0 on ikinci durum.
pub fn shayanadi_state(total: u32) -> Shayanadi {
    Shayanadi::ALL[((total + 11) % 12) as usize]
}

/// Haritaya ait terimler; lagna yoksa (doğum saati bilinmiyor) ya da ghati yoksa (gün doğumu
/// yok) `None`.
pub fn shayanadi_basis(positions: &Positions, ghatis: Option<u32>) -> Option<ShayanadiBasis> {
    let lagna = positions.lagna_sign?;
    Some(ShayanadiBasis {
        birth_nakshatra: nakshatra_pada(positions.point(Graha::Moon).sidereal_lon).0,
        ghatis: ghatis?,
        lagna_sign_number: lagna.index() as u8 + 1,
    })
}

/// `graha`'nın Shayanadi durumu; düğümler için `None`.
pub fn shayanadi(
    graha: Graha,
    positions: &Positions,
    basis: &ShayanadiBasis,
) -> Option<ShayanadiHit> {
    let graha_number = graha_number(graha)?;
    let point = positions.point(graha);
    let nakshatra = nakshatra_pada(point.sidereal_lon).0;
    let navamsa = navamsa_number(point.deg_in_sign);
    let total = shayanadi_total(nakshatra, graha_number, navamsa, basis);
    Some(ShayanadiHit {
        state: shayanadi_state(total),
        nakshatra,
        graha_number,
        navamsa,
        total,
        remainder: (total % 12) as u8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avastha::AVASTHA_GRAHAS;
    use Graha::*;
    use Shayanadi::*;

    fn basis(birth_nakshatra: u8, ghatis: u32, lagna_sign_number: u8) -> ShayanadiBasis {
        ShayanadiBasis {
            birth_nakshatra,
            ghatis,
            lagna_sign_number,
        }
    }

    #[test]
    fn graha_numbers_follow_the_weekday_order_and_nodes_have_none() {
        let numbers: Vec<_> = Graha::ALL.into_iter().map(graha_number).collect();
        assert_eq!(
            numbers,
            [
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                None,
                None
            ]
        );
    }

    #[test]
    fn navamsa_number_is_the_ninth_of_the_sign() {
        for (deg, want) in [
            (0.0, 1),
            (3.3, 1),
            (10.0 / 3.0, 2),
            (6.6, 2),
            (20.0 / 3.0, 3),
            (10.0, 4),
            (15.0, 5),
            (16.7, 6),
            (20.0, 7),
            (23.4, 8),
            (26.7, 9),
            (29.999, 9),
            (30.0, 9),
            (-0.1, 1),
        ] {
            assert_eq!(navamsa_number(deg), want, "{deg}");
        }
    }

    #[test]
    fn elapsed_ghatis_are_completed_ghatis_of_twenty_four_minutes() {
        let rise = 2_451_544.75; // 06:00 UT
        let minutes = |m: f64| rise + m / 1440.0;
        assert_eq!(elapsed_ghatis(rise, rise), Some(0));
        assert_eq!(elapsed_ghatis(minutes(23.9), rise), Some(0));
        assert_eq!(elapsed_ghatis(minutes(24.1), rise), Some(1));
        // 06:00'dan 17:30'a 690 dk = 28,75 ghati: tamamlanan 28.
        assert_eq!(elapsed_ghatis(minutes(690.0), rise), Some(28));
        // Gün doğumundan hemen önce doğan, önceki gün doğumundan sayar: 59 tamamlanmış ghati.
        assert_eq!(elapsed_ghatis(minutes(1439.0), rise), Some(59));
        assert_eq!(elapsed_ghatis(minutes(-1.0), rise), None);
        assert_eq!(elapsed_ghatis(f64::NAN, rise), None);
        assert_eq!(elapsed_ghatis(rise, f64::INFINITY), None);
    }

    #[test]
    fn the_formula_multiplies_three_terms_and_adds_three() {
        // El örneği: N = 4, G = 1, A = 5 → 20; J = 10, Gh = 28, L = 3 → 41; toplam 61.
        // 61 = 5 × 12 + 1: artan 1, ilk durum.
        let b = basis(10, 28, 3);
        assert_eq!(shayanadi_total(4, 1, 5, &b), 61);
        assert_eq!(shayanadi_state(61), Shayana);
        // Artan sırası: 1 Shayana, 2 Upaveshana, …, 11 Kautuka, 0 Nidra.
        let by_remainder: Vec<_> = (1..=12).map(shayanadi_state).collect();
        assert_eq!(by_remainder, Shayanadi::ALL);
        assert_eq!(shayanadi_state(12), Nidra);
        assert_eq!(shayanadi_state(24), Nidra);
        assert_eq!(shayanadi_state(0), Nidra);
        assert_eq!(shayanadi_state(13), Shayana);
        assert_eq!(shayanadi_state(71), Kautuka); // 71 = 5 × 12 + 11
        assert_eq!(shayanadi_state(57), Bhojana); // 57 = 4 × 12 + 9
    }

    #[test]
    fn second_writing_counting_through_the_cycle_agrees_for_every_input() {
        // İkinci yazım: bölme yok. Durum çemberinde her terim için adım adım yüründüğünde
        // (çarpım, tekrarlı toplama olarak) varılan yer formülün verdiği durumdur. İlk adım
        // ilk duruma varır: başlangıç on ikinci durumdur.
        fn walk(steps: [u32; 4], product: [u32; 3]) -> Shayanadi {
            let mut at = 11_usize; // Nidra: bir adım sonrası Shayana
            let mut step = |n: u32| {
                for _ in 0..n {
                    at = if at == 11 { 0 } else { at + 1 };
                }
            };
            for _ in 0..product[1] {
                for _ in 0..product[2] {
                    step(product[0]);
                }
            }
            for n in steps {
                step(n);
            }
            Shayanadi::ALL[at]
        }
        for nakshatra in 1..=27_u8 {
            for graha in 1..=7_u8 {
                for navamsa in 1..=9_u8 {
                    // Üç toplanan yalnız toplamıyla etki eder: 1 + 0 + 1 = 2 … 27 + 60 + 12 = 99.
                    for addend in 2..=99_u32 {
                        let b = basis(1, addend - 2, 1);
                        let total = shayanadi_total(nakshatra, graha, navamsa, &b);
                        assert_eq!(
                            shayanadi_state(total),
                            walk(
                                [1, addend - 2, 1, 0],
                                [nakshatra.into(), graha.into(), navamsa.into()]
                            ),
                            "{nakshatra} {graha} {navamsa} {addend}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn basis_needs_the_lagna_and_the_ghatis() {
        // Ay 100° → nakshatra 8 (93°20′–106°40′); lagna 70° İkizler → 3.
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        assert_eq!(shayanadi_basis(&p, Some(28)), Some(basis(8, 28, 3)));
        assert_eq!(shayanadi_basis(&p, None), None);
        let unknown = p.clone().without_lagna();
        assert_eq!(shayanadi_basis(&unknown, Some(28)), None);
    }

    #[test]
    fn each_graha_gets_its_state_from_its_own_three_terms() {
        // Güneş 10° Koç: nakshatra 1 (0°–13°20′), navamsa 4 (10°–13°20′), G = 1 → 4.
        // Jüpiter 250° Yay 10°: nakshatra 19 (240°–253°20′), navamsa 4, G = 5 → 380.
        // Ay 100° Yengeç 10°: nakshatra 8, navamsa 4, G = 2 → 64.
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        let b = basis(8, 28, 3); // J + Gh + L = 39
        let sun = shayanadi(Sun, &p, &b).expect("sun");
        assert_eq!(
            (sun.nakshatra, sun.graha_number, sun.navamsa, sun.total),
            (1, 1, 4, 43)
        );
        assert_eq!((sun.state, sun.remainder), (Sabha, 7)); // 43 = 3 × 12 + 7
        let jupiter = shayanadi(Jupiter, &p, &b).expect("jupiter");
        assert_eq!(
            (jupiter.nakshatra, jupiter.navamsa, jupiter.total),
            (19, 4, 419)
        );
        assert_eq!((jupiter.state, jupiter.remainder), (Kautuka, 11)); // 419 = 34 × 12 + 11
        let moon = shayanadi(Moon, &p, &b).expect("moon");
        assert_eq!((moon.nakshatra, moon.navamsa, moon.total), (8, 4, 103));
        assert_eq!(moon.state, Sabha); // 103 = 8 × 12 + 7
        assert_eq!(shayanadi(Rahu, &p, &b), None);
        assert_eq!(shayanadi(Ketu, &p, &b), None);
        for graha in AVASTHA_GRAHAS {
            let hit = shayanadi(graha, &p, &b).expect("seven grahas");
            assert_eq!(hit.state, shayanadi_state(hit.total), "{graha:?}");
            assert_eq!(u32::from(hit.remainder), hit.total % 12, "{graha:?}");
        }
    }
}
