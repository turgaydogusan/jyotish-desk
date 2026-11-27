//! Muntha ve yılın lagnaya bağlı olguları. Muntha doğumda lagnanın burcundadır ve tamamlanan her
//! yılda bir burç ilerler: `age` yaşında doğum lagnasının burcundan `age` burç ötededir. Burçlar
//! rasi çerçevesindedir (K5); evler burç evidir (K4).

use serde::{Deserialize, Serialize};

use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Munthanın burcu: doğum lagnasının burcu + tamamlanan yıl (on ikide bir başa döner).
pub fn muntha_sign(natal_lagna: Sign, age: u8) -> Sign {
    Sign::from_index(natal_lagna.index() + usize::from(age))
}

/// Yılın munthası.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Muntha {
    pub sign: Sign,
    /// Muntha burcunun lordu.
    pub lord: Graha,
    /// Doğum lagnasından sayılan burç evi, 1..=12.
    pub natal_house: u8,
    /// Yıllık haritanın lagnasından sayılan burç evi, 1..=12.
    pub annual_house: u8,
}

/// Yılın lagnaya bağlı olguları; doğum saati bilinmiyorsa hiçbiri verilmez (K2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YearFacts {
    /// Yıllık haritanın lagna burcu ve lordu.
    pub lagna_sign: Sign,
    pub lagna_lord: Graha,
    /// Doğum haritasının lagna burcu ve lordu.
    pub natal_lagna_sign: Sign,
    pub natal_lagna_lord: Graha,
    pub muntha: Muntha,
}

/// Doğum lagnası, yıllık lagna ve tamamlanan yıldan yılın olguları.
pub fn year_facts(natal_lagna: Sign, annual_lagna: Sign, age: u8) -> YearFacts {
    let sign = muntha_sign(natal_lagna, age);
    YearFacts {
        lagna_sign: annual_lagna,
        lagna_lord: sign_lord(annual_lagna),
        natal_lagna_sign: natal_lagna,
        natal_lagna_lord: sign_lord(natal_lagna),
        muntha: Muntha {
            sign,
            lord: sign_lord(sign),
            natal_house: sign.house_from(natal_lagna),
            annual_house: sign.house_from(annual_lagna),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muntha_starts_in_the_lagna_sign_and_moves_one_sign_a_year() {
        assert_eq!(muntha_sign(Sign::Taurus, 0), Sign::Taurus);
        assert_eq!(muntha_sign(Sign::Taurus, 1), Sign::Gemini);
        // Boğa'dan 11 burç sonrası Koç'tur; 12. yılda başa döner.
        assert_eq!(muntha_sign(Sign::Taurus, 11), Sign::Aries);
        assert_eq!(muntha_sign(Sign::Taurus, 12), Sign::Taurus);
        // Balık lagnası, yıl 26: 26 = 2 × 12 + 2 → iki burç ötesi Boğa.
        assert_eq!(muntha_sign(Sign::Pisces, 26), Sign::Taurus);
        // En büyük yaş: 120 = 10 × 12 → lagna burcu.
        assert_eq!(muntha_sign(Sign::Leo, 120), Sign::Leo);
    }

    #[test]
    fn second_formulation_walking_sign_by_sign() {
        // Her yıl bir burç yürüyen sayaç, kapalı yazımla her lagna ve her yaşta aynıdır.
        for lagna in Sign::ALL {
            let mut walk = lagna;
            for age in 0..=120_u8 {
                assert_eq!(muntha_sign(lagna, age), walk, "{lagna:?} age {age}");
                assert_eq!(
                    muntha_sign(lagna, age).house_from(lagna),
                    age % 12 + 1,
                    "{lagna:?} age {age}: natal house"
                );
                walk = Sign::from_index(walk.index() + 1);
            }
        }
    }

    #[test]
    fn year_facts_give_the_lords_and_both_houses() {
        // Doğum lagnası Boğa, yıl 3: muntha Aslan (lordu Güneş), doğum lagnasından 4. ev.
        // Yıllık lagna Kova: Aslan, Kova'dan 7. evdir. Kova'nın lordu Satürn, Boğa'nınki Venüs.
        let f = year_facts(Sign::Taurus, Sign::Aquarius, 3);
        assert_eq!(
            f,
            YearFacts {
                lagna_sign: Sign::Aquarius,
                lagna_lord: Graha::Saturn,
                natal_lagna_sign: Sign::Taurus,
                natal_lagna_lord: Graha::Venus,
                muntha: Muntha {
                    sign: Sign::Leo,
                    lord: Graha::Sun,
                    natal_house: 4,
                    annual_house: 7,
                },
            }
        );
        // Yıl 0, yıllık lagna doğum lagnası: muntha iki haritada da 1. evdedir.
        let birth = year_facts(Sign::Scorpio, Sign::Scorpio, 0);
        assert_eq!(
            (
                birth.muntha.sign,
                birth.muntha.natal_house,
                birth.muntha.annual_house
            ),
            (Sign::Scorpio, 1, 1)
        );
        assert_eq!(birth.muntha.lord, Graha::Mars);
    }
}
