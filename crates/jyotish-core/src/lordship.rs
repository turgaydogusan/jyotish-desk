//! Ev lordluğu yardımcıları (ileri modüller spec'i §4). Evler rasi evidir: lagna burcundan
//! sayılan bütün burç evleri (K4); `CalcSettings.house_system` bunları etkilemez. Burç lordu
//! `dignity::sign_lord`'dur (Akrep: Mars, Kova: Satürn); Jaimini eş lordları burada yoktur.

use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Lagna burcundan sayılan `house`. evin burcu. `house` 1..=12'dir; aralık dışı değer 12'ye
/// göre sarılır (13 → 1, 0 → 12).
pub fn house_sign(lagna_sign: Sign, house: u8) -> Sign {
    Sign::from_index(lagna_sign.index() + (usize::from(house) + 11) % 12)
}

/// `house`. evin lordu.
pub fn house_lord(lagna_sign: Sign, house: u8) -> Graha {
    sign_lord(house_sign(lagna_sign, house))
}

/// Grahanın lordu olduğu evler, artan sırada. Güneş ve Ay birer, diğer beş graha ikişer
/// evin lordudur; Rahu ve Ketu için boştur.
pub fn houses_owned(lagna_sign: Sign, graha: Graha) -> Vec<u8> {
    (1..=12)
        .filter(|&h| house_lord(lagna_sign, h) == graha)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    #[test]
    fn house_signs_count_inclusively_from_the_lagna_sign() {
        assert_eq!(house_sign(Sign::Aries, 1), Sign::Aries);
        assert_eq!(house_sign(Sign::Aries, 7), Sign::Libra);
        assert_eq!(house_sign(Sign::Scorpio, 6), Sign::Aries);
        assert_eq!(house_sign(Sign::Pisces, 2), Sign::Aries);
        assert_eq!(house_sign(Sign::Gemini, 12), Sign::Taurus);
        // Aralık dışı değerler sarılır.
        assert_eq!(house_sign(Sign::Gemini, 13), Sign::Gemini);
        assert_eq!(house_sign(Sign::Gemini, 0), Sign::Taurus);
        for lagna in Sign::ALL {
            for h in 1..=12u8 {
                assert_eq!(house_sign(lagna, h).house_from(lagna), h);
            }
        }
    }

    #[test]
    fn house_lords_for_three_lagnas_match_the_hand_table() {
        // Koç lagna: evler Koç…Balık.
        let aries: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Aries, h)).collect();
        assert_eq!(
            aries,
            [
                Mars, Venus, Mercury, Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Saturn,
                Jupiter
            ]
        );
        // Yengeç lagna: evler Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık,
        // Koç, Boğa, İkizler.
        let cancer: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Cancer, h)).collect();
        assert_eq!(
            cancer,
            [
                Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Saturn, Jupiter, Mars, Venus,
                Mercury
            ]
        );
        // Kova lagna: evler Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi,
        // Akrep, Yay, Oğlak.
        let aquarius: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Aquarius, h)).collect();
        assert_eq!(
            aquarius,
            [
                Saturn, Jupiter, Mars, Venus, Mercury, Moon, Sun, Mercury, Venus, Mars, Jupiter,
                Saturn
            ]
        );
    }

    #[test]
    fn houses_owned_lists_each_house_once() {
        assert_eq!(houses_owned(Sign::Aries, Mars), [1, 8]);
        assert_eq!(houses_owned(Sign::Aries, Sun), [5]);
        assert_eq!(houses_owned(Sign::Cancer, Saturn), [7, 8]);
        assert_eq!(houses_owned(Sign::Aquarius, Saturn), [1, 12]);
        assert_eq!(houses_owned(Sign::Libra, Mars), [2, 7]);
        assert!(houses_owned(Sign::Aries, Rahu).is_empty());
        assert!(houses_owned(Sign::Aries, Ketu).is_empty());
        for lagna in Sign::ALL {
            let mut all: Vec<u8> = Graha::ALL
                .into_iter()
                .flat_map(|g| houses_owned(lagna, g))
                .collect();
            all.sort_unstable();
            assert_eq!(all, (1..=12).collect::<Vec<u8>>(), "{lagna:?}");
            assert_eq!(houses_owned(lagna, Sun).len(), 1);
            assert_eq!(houses_owned(lagna, Moon).len(), 1);
            for g in [Mars, Mercury, Jupiter, Venus, Saturn] {
                assert_eq!(houses_owned(lagna, g).len(), 2, "{lagna:?} {g:?}");
            }
        }
    }
}
