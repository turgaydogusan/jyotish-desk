//! Dagdha rasi ve panchang eklerinin raporu (yol haritası §4.1).
//!
//! Dagdha rasi, doğum tithisine bağlı burçlardır: on üç tithide iki burç, chaturdashide dört
//! burç; dolunayda ve yeni ayda yoktur. Tablo bellekten yazılmıştır ve basılı kaynakla hücre
//! hücre denetlenmeyi bekler (Y4): `tests/golden/modules/natal_extras/REVIEW.md`. Shukla ve
//! Krishna paksha aynı tabloyu paylaşır (tek okuma; `PANCHANG_EXTRAS_METHOD`).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::panchang::{elongation, tithi_no};
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::yogi::{yogi_facts, YogiFacts};

/// Yöntem kimliği (Y5): 93°20′ yay, altıncı nakshatradan avayogi, iki pakshada ortak dagdha
/// rasi tablosu.
pub const PANCHANG_EXTRAS_METHOD: &str = "panchang_extras.v1";

/// Paksha içi tithi sırasına (1..=15) göre dagdha burçlar; her hücre zodyak sırasındadır.
const DAGDHA_RASI: [&[Sign]; 15] = [
    &[Sign::Libra, Sign::Capricorn],
    &[Sign::Sagittarius, Sign::Pisces],
    &[Sign::Leo, Sign::Capricorn],
    &[Sign::Taurus, Sign::Aquarius],
    &[Sign::Gemini, Sign::Virgo],
    &[Sign::Aries, Sign::Leo],
    &[Sign::Cancer, Sign::Sagittarius],
    &[Sign::Gemini, Sign::Virgo],
    &[Sign::Leo, Sign::Scorpio],
    &[Sign::Leo, Sign::Scorpio],
    &[Sign::Sagittarius, Sign::Pisces],
    &[Sign::Libra, Sign::Capricorn],
    &[Sign::Taurus, Sign::Leo],
    &[Sign::Gemini, Sign::Virgo, Sign::Sagittarius, Sign::Pisces],
    &[],
];

/// Tithinin (1..=30) dagdha burçları; 15 (Purnima) ve 30 (Amavasya) için boştur. Aralık dışı
/// numara boş liste verir (panik yok).
pub fn dagdha_rasis(tithi_no: u8) -> &'static [Sign] {
    match tithi_no {
        1..=30 => DAGDHA_RASI[usize::from(tithi_no - 1) % 15],
        _ => &[],
    }
}

/// `compute_panchang_extras` sonucu: doğum anının beş ek panchang olgusu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PanchangExtras {
    pub yogi: YogiFacts,
    /// Doğum tithisi, 1..=30; dagdha rasi buna bağlıdır.
    pub tithi: u8,
    /// Zodyak sırasında; dolunay ve yeni ayda boş.
    pub dagdha_rasis: Vec<Sign>,
    /// Doğum saati bilinmiyor: değerler öğlen konumlarındandır (Ay günde yaklaşık 13° yürür).
    pub approximate: bool,
    pub method: String,
}

/// Konumlardan panchang ekleri. Tithi, Güneş ve Ay'ın sidereal boylamlarının farkındandır
/// (uzanım ayanamsadan bağımsızdır). Lagna gerekmez.
pub fn panchang_extras(positions: &Positions, approximate: bool) -> PanchangExtras {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    let tithi = tithi_no(elongation(sun, moon));
    PanchangExtras {
        yogi: yogi_facts(positions),
        tithi,
        dagdha_rasis: dagdha_rasis(tithi).to_vec(),
        approximate,
        method: PANCHANG_EXTRAS_METHOD.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sign::*;

    fn sky(sun: f64, moon: f64) -> Positions {
        let mut lons = [5.0; 9];
        lons[Graha::Sun as usize] = sun;
        lons[Graha::Moon as usize] = moon;
        Positions::synthetic(None, lons)
    }

    #[test]
    fn two_signs_on_thirteen_tithis_four_on_the_fourteenth_none_on_the_fifteenth() {
        for tithi in 1..=30u8 {
            let signs = dagdha_rasis(tithi);
            let expected = match (tithi - 1) % 15 + 1 {
                14 => 4,
                15 => 0,
                _ => 2,
            };
            assert_eq!(signs.len(), expected, "tithi {tithi}");
            // Hücre zodyak sırasındadır ve bir burcu iki kez saymaz.
            assert!(signs.windows(2).all(|w| w[0] < w[1]), "tithi {tithi}");
        }
        // İki paksha aynı tabloyu paylaşır.
        for tithi in 1..=15u8 {
            assert_eq!(
                dagdha_rasis(tithi),
                dagdha_rasis(tithi + 15),
                "tithi {tithi}"
            );
        }
        // Tablonun satırları, elle (paksha içi sıra).
        assert_eq!(dagdha_rasis(1), [Libra, Capricorn]);
        assert_eq!(dagdha_rasis(2), [Sagittarius, Pisces]);
        assert_eq!(dagdha_rasis(3), [Leo, Capricorn]);
        assert_eq!(dagdha_rasis(4), [Taurus, Aquarius]);
        assert_eq!(dagdha_rasis(5), [Gemini, Virgo]);
        assert_eq!(dagdha_rasis(6), [Aries, Leo]);
        assert_eq!(dagdha_rasis(7), [Cancer, Sagittarius]);
        assert_eq!(dagdha_rasis(8), [Gemini, Virgo]);
        assert_eq!(dagdha_rasis(9), [Leo, Scorpio]);
        assert_eq!(dagdha_rasis(10), [Leo, Scorpio]);
        assert_eq!(dagdha_rasis(11), [Sagittarius, Pisces]);
        assert_eq!(dagdha_rasis(12), [Libra, Capricorn]);
        assert_eq!(dagdha_rasis(13), [Taurus, Leo]);
        assert_eq!(dagdha_rasis(14), [Gemini, Virgo, Sagittarius, Pisces]);
        assert!(dagdha_rasis(15).is_empty());
        assert!(dagdha_rasis(30).is_empty());
    }

    /// İkinci yazım: tablo burçtan okunur. Her burç için, o burcun dagdha olduğu paksha içi
    /// tithiler (REVIEW.md'deki ikinci tablo).
    const BY_SIGN: [(Sign, &[u8]); 12] = [
        (Aries, &[6]),
        (Taurus, &[4, 13]),
        (Gemini, &[5, 8, 14]),
        (Cancer, &[7]),
        (Leo, &[3, 6, 9, 10, 13]),
        (Virgo, &[5, 8, 14]),
        (Libra, &[1, 12]),
        (Scorpio, &[9, 10]),
        (Sagittarius, &[2, 7, 11, 14]),
        (Capricorn, &[1, 3, 12]),
        (Aquarius, &[4]),
        (Pisces, &[2, 11, 14]),
    ];

    #[test]
    fn the_table_agrees_with_its_second_writing_by_sign() {
        let mut cells = 0;
        for (sign, tithis) in BY_SIGN {
            for tithi in 1..=15u8 {
                assert_eq!(
                    dagdha_rasis(tithi).contains(&sign),
                    tithis.contains(&tithi),
                    "{sign:?} on tithi {tithi}"
                );
            }
            cells += tithis.len();
        }
        // 13 × 2 + 4 = 30 hücre.
        assert_eq!(cells, 30);
        assert_eq!((1..=15u8).map(|t| dagdha_rasis(t).len()).sum::<usize>(), 30);
    }

    #[test]
    fn out_of_range_tithis_have_no_dagdha_sign() {
        assert!(dagdha_rasis(0).is_empty());
        assert!(dagdha_rasis(31).is_empty());
        assert!(dagdha_rasis(u8::MAX).is_empty());
    }

    #[test]
    fn extras_join_the_yogi_facts_the_tithi_and_the_dagdha_signs() {
        // Güneş 10°, Ay 101°: uzanım 91° → 91 / 12 = 7,58 → 8. tithi (Shukla Ashtami):
        // İkizler ve Başak. Yogi olguları `yogi.rs`'in A örneğidir (nokta 204°20′).
        let a = panchang_extras(&sky(10.0, 101.0), false);
        assert_eq!(a.tithi, 8);
        assert_eq!(a.dagdha_rasis, [Gemini, Virgo]);
        assert_eq!(a.yogi, yogi_facts(&sky(10.0, 101.0)));
        assert_eq!((a.yogi.nakshatra, a.yogi.yogi), (16, Graha::Jupiter));
        assert!(!a.approximate);
        assert_eq!(a.method, "panchang_extras.v1");
        // Güneş 200°, Ay 31°40′: uzanım 191°40′ → 15,97 → 16. tithi (Krishna Pratipada):
        // Shukla Pratipada ile aynı satır, Terazi ve Oğlak.
        let c = panchang_extras(&sky(200.0, 31.0 + 40.0 / 60.0), true);
        assert_eq!(c.tithi, 16);
        assert_eq!(c.dagdha_rasis, [Libra, Capricorn]);
        assert!(c.approximate);
        // Uzanım 175° → 15. tithi (Purnima) ve 355° → 30. tithi (Amavasya): dagdha rasi yok.
        assert_eq!(panchang_extras(&sky(10.0, 185.0), false).tithi, 15);
        assert!(panchang_extras(&sky(10.0, 185.0), false)
            .dagdha_rasis
            .is_empty());
        assert_eq!(panchang_extras(&sky(10.0, 5.0), false).tithi, 30);
        assert!(panchang_extras(&sky(10.0, 5.0), false)
            .dagdha_rasis
            .is_empty());
        // Uzanım 340° → 29. tithi (Krishna Chaturdashi): dört burç.
        let k14 = panchang_extras(&sky(10.0, 350.0), false);
        assert_eq!(k14.tithi, 29);
        assert_eq!(k14.dagdha_rasis, [Gemini, Virgo, Sagittarius, Pisces]);
    }
}
