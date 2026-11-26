//! Arudha padalar (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, pada bölümü. Evin burcu `S`, lordu `T` burcunda olsun; `n`, `S`'den `T`'ye
//! kapsayıcı sayımdır. Pada `T`'den `n`. burçtur. Hesaplanan burç `S`'nin kendisi ya da `S`'den
//! 7. burç ise pada, hesaplanan burçtan 10. burçtur. Evler lagna burcundan sayılan rasi
//! evleridir (K4); Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir. Graha arudhaları yoktur.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::lordship::house_sign;
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::lords::jaimini_lord;

/// Bir evin padası. A1 Arudha Lagna (AL), A12 Upapada (UL) diye de anılır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArudhaPada {
    /// Ev, 1..=12.
    pub house: u8,
    /// Padanın düştüğü burç.
    pub sign: Sign,
    /// Evin hesapta kullanılan lordu (Akrep ve Kova'da seçilen eş lord).
    pub lord: Graha,
    /// İstisna uygulandı: hesaplanan burç evin kendisi ya da 7. burcuydu.
    pub exception: bool,
}

/// `sign` burcunun, lordu `lord_sign` burcundayken padası ve istisnanın uygulanıp
/// uygulanmadığı.
pub fn arudha_of(sign: Sign, lord_sign: Sign) -> (Sign, bool) {
    let n = usize::from(lord_sign.house_from(sign));
    let computed = Sign::from_index(lord_sign.index() + n - 1);
    if matches!(computed.house_from(sign), 1 | 7) {
        (Sign::from_index(computed.index() + 9), true)
    } else {
        (computed, false)
    }
}

/// On iki evin padası, ev sırasıyla. Doğum saati bilinmiyorsa (lagna yoksa) `None`.
pub fn arudha_padas(positions: &Positions, co_lords: bool) -> Option<[ArudhaPada; 12]> {
    let lagna = positions.lagna_sign?;
    Some(std::array::from_fn(|i| {
        let house = i as u8 + 1;
        let house_sign = house_sign(lagna, house);
        let lord = jaimini_lord(house_sign, positions, co_lords);
        let (sign, exception) = arudha_of(house_sign, positions.sign_of(lord));
        ArudhaPada {
            house,
            sign,
            lord,
            exception,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn pada_is_as_far_from_the_lord_as_the_lord_is_from_the_house() {
        // Lord 2. burçta: n = 2; lordun burcundan 2. burç, evden 3. burçtur.
        assert_eq!(arudha_of(Aries, Taurus), (Gemini, false));
        // Lord 5. burçta: Aslan'dan 5. burç Yay.
        assert_eq!(arudha_of(Aries, Leo), (Sagittarius, false));
        // Lord 12. burçta: Balık'tan 12. burç Kova.
        assert_eq!(arudha_of(Aries, Pisces), (Aquarius, false));
        // Lord 9. burçta: Yay'dan 9. burç Aslan.
        assert_eq!(arudha_of(Aries, Sagittarius), (Leo, false));
    }

    #[test]
    fn exceptions_move_the_pada_ten_signs_on() {
        // Lord evin kendi burcunda: hesaplanan burç evin kendisi; padası evden 10. burç.
        assert_eq!(arudha_of(Aries, Aries), (Capricorn, true));
        // Lord 7. burçta: Terazi'den 7. burç yine Koç; pada Oğlak.
        assert_eq!(arudha_of(Aries, Libra), (Capricorn, true));
        // Lord 4. burçta: Yengeç'ten 4. burç Terazi (evden 7.); Terazi'den 10. burç Yengeç.
        assert_eq!(arudha_of(Aries, Cancer), (Cancer, true));
        // Lord 10. burçta: Oğlak'tan 10. burç Terazi (evden 7.); pada Yengeç.
        assert_eq!(arudha_of(Aries, Capricorn), (Cancer, true));
    }

    #[test]
    fn no_pada_falls_in_the_house_or_its_seventh_and_the_exceptions_are_four() {
        for sign in Sign::ALL {
            let mut exceptions = Vec::new();
            for lord_sign in Sign::ALL {
                let (pada, exception) = arudha_of(sign, lord_sign);
                assert!(
                    !matches!(pada.house_from(sign), 1 | 7),
                    "{sign:?} with lord in {lord_sign:?}"
                );
                if exception {
                    exceptions.push(lord_sign.house_from(sign));
                    // İstisnada pada evden 10. (lord 1. ya da 7.) ya da 4. (lord 4. ya da 10.).
                    let want = if matches!(lord_sign.house_from(sign), 1 | 7) {
                        10
                    } else {
                        4
                    };
                    assert_eq!(pada.house_from(sign), want);
                }
            }
            exceptions.sort_unstable();
            assert_eq!(exceptions, [1, 4, 7, 10], "{sign:?}");
        }
    }

    #[test]
    fn twelve_padas_from_the_lagna_and_none_without_it() {
        // Koç lagna. Güneş Aslan, Ay Boğa, Mars Yengeç, Merkür Başak, Jüpiter Yay, Venüs
        // Balık, Satürn Terazi, Rahu İkizler, Ketu Yay.
        let lons = [125.0, 40.0, 100.0, 160.0, 250.0, 340.0, 190.0, 70.0, 250.0];
        let p = Positions::synthetic(Some(5.0), lons);
        let padas = arudha_padas(&p, false).expect("lagna");
        for (i, pada) in padas.iter().enumerate() {
            assert_eq!(pada.house, i as u8 + 1);
        }
        // 1. ev Koç, lord Mars Yengeç'te (4.): istisna, pada Yengeç.
        assert_eq!(
            (padas[0].sign, padas[0].lord, padas[0].exception),
            (Cancer, Mars, true)
        );
        // 5. ev Aslan, lord Güneş Aslan'da: istisna, pada Aslan'dan 10. burç Boğa.
        assert_eq!(
            (padas[4].sign, padas[4].lord, padas[4].exception),
            (Taurus, Sun, true)
        );
        // 2. ev Boğa, lord Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak.
        assert_eq!(
            (padas[1].sign, padas[1].lord, padas[1].exception),
            (Capricorn, Venus, false)
        );
        // 8. ev Akrep: eş lordlar devre dışıyken Mars (Yengeç, 9.): Yengeç'ten 9. burç Balık.
        assert_eq!((padas[7].sign, padas[7].lord), (Pisces, Mars));
        // Eş lordlar açıkken: Mars Yengeç'te yalnız, Ketu Yay'da Jüpiter'le: Ketu (2. ölçüt).
        // Ketu Yay'da (Akrep'ten 2.): Yay'dan 2. burç Oğlak.
        let with_co_lords = arudha_padas(&p, true).expect("lagna");
        assert_eq!(
            (with_co_lords[7].sign, with_co_lords[7].lord),
            (Capricorn, Ketu)
        );
        assert_eq!(arudha_padas(&p.clone().without_lagna(), true), None);
    }
}
