//! Doğal iyicil/kötücül sınıfı ve Kuja dosha olgusu (ileri modüller spec'i K6, §4). Modüller
//! kendi iyicil/kötücül listesini tutmaz; hepsi `nature` fonksiyonunu çağırır.
//!
//! Kaynak: BPHS, graha nitelikleri bölümü. Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu
//! ve Ketu kötücül; Ay büyürken iyicil, küçülürken kötücül; Merkür aynı burçta kötücül varken
//! iyicil yoksa kötücül, değilse iyicil.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::normalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Nature {
    Benefic,
    Malefic,
}

/// Ay büyüyor mu: Güneş'ten uzanım [0°, 180°) (Shukla paksha; `panchang::paksha` ile aynı
/// sınır). Uzanım iki boylamın farkıdır, ayanamsadan bağımsızdır.
pub fn moon_is_waxing(positions: &Positions) -> bool {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    normalize(moon - sun) < 180.0
}

/// Merkür dışındaki grahaların sınıfı (Merkür'ün sınıfı bunlara bağlıdır).
fn fixed_nature(graha: Graha, positions: &Positions) -> Nature {
    match graha {
        Graha::Jupiter | Graha::Venus => Nature::Benefic,
        Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Rahu | Graha::Ketu => Nature::Malefic,
        Graha::Moon => {
            if moon_is_waxing(positions) {
                Nature::Benefic
            } else {
                Nature::Malefic
            }
        }
        // Çağrılmaz; `nature` Merkür'ü kendisi çözer.
        Graha::Mercury => Nature::Benefic,
    }
}

/// Grahanın doğal sınıfı (K6). "Aynı burç" rasi çerçevesindeki burçtur (K5).
pub fn nature(graha: Graha, positions: &Positions) -> Nature {
    if graha != Graha::Mercury {
        return fixed_nature(graha, positions);
    }
    let sign = positions.sign_of(Graha::Mercury);
    let (mut benefic, mut malefic) = (false, false);
    for other in Graha::ALL {
        if other == Graha::Mercury || positions.sign_of(other) != sign {
            continue;
        }
        match fixed_nature(other, positions) {
            Nature::Benefic => benefic = true,
            Nature::Malefic => malefic = true,
        }
    }
    if malefic && !benefic {
        Nature::Malefic
    } else {
        Nature::Benefic
    }
}

/// Kuja dosha olgusu: Mars ilgili referanstan sayılan 1, 2, 4, 7, 8 ya da 12. rasi evinde mi.
/// İptal kuralı uygulanmaz. Yoga (3.5) ve uyum (3.6) modülleri aynı olguyu kullanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct KujaDosha {
    /// Doğum saati bilinmiyorsa `None`.
    pub from_lagna: Option<bool>,
    pub from_moon: bool,
    pub from_venus: bool,
}

/// Mars'ın dosha sayılan evleri.
pub const KUJA_DOSHA_HOUSES: [u8; 6] = [1, 2, 4, 7, 8, 12];

pub fn kuja_dosha(positions: &Positions) -> KujaDosha {
    let from =
        |reference| KUJA_DOSHA_HOUSES.contains(&positions.house_from(Graha::Mars, reference));
    KujaDosha {
        from_lagna: positions.lagna_sign.map(from),
        from_moon: from(positions.sign_of(Graha::Moon)),
        from_venus: from(positions.sign_of(Graha::Venus)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: hiçbir graha Merkür'le (İkizler 65°) aynı burçta değil; Ay büyüyor.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(Some(0.0), lons)
    }

    #[test]
    fn fixed_classes() {
        let p = with(&[]);
        for g in [Jupiter, Venus] {
            assert_eq!(nature(g, &p), Nature::Benefic, "{g:?}");
        }
        for g in [Sun, Mars, Saturn, Rahu, Ketu] {
            assert_eq!(nature(g, &p), Nature::Malefic, "{g:?}");
        }
    }

    #[test]
    fn moon_is_benefic_while_waxing() {
        // Güneş 10°: uzanım = Ay − 10°.
        for (moon, waxing) in [
            (10.0, true),    // 0°: yeni ay, Shukla 1'in başı
            (100.0, true),   // 90°
            (189.999, true), // 179,999°
            (190.0, false),  // 180°: dolunay, Krishna 1'in başı
            (280.0, false),  // 270°
            (9.999, false),  // 359,999°
        ] {
            let p = with(&[(Moon, moon)]);
            assert_eq!(moon_is_waxing(&p), waxing, "moon {moon}");
            let want = if waxing {
                Nature::Benefic
            } else {
                Nature::Malefic
            };
            assert_eq!(nature(Moon, &p), want, "moon {moon}");
        }
    }

    #[test]
    fn mercury_follows_its_sign_companions() {
        // Yalnız: iyicil.
        assert_eq!(nature(Mercury, &with(&[])), Nature::Benefic);
        // Yalnızca kötücülle (her biri ayrı ayrı): kötücül.
        for g in [Sun, Mars, Saturn, Rahu, Ketu] {
            assert_eq!(
                nature(Mercury, &with(&[(g, 70.0)])),
                Nature::Malefic,
                "{g:?}"
            );
        }
        // Yalnızca iyicille: iyicil.
        for g in [Jupiter, Venus] {
            assert_eq!(
                nature(Mercury, &with(&[(g, 70.0)])),
                Nature::Benefic,
                "{g:?}"
            );
        }
        // Kötücül ve iyicil birlikte: iyicil.
        assert_eq!(
            nature(Mercury, &with(&[(Saturn, 61.0), (Jupiter, 89.0)])),
            Nature::Benefic
        );
        // Büyüyen Ay iyicil sayılır (Güneş 10°, Ay 70°: uzanım 60°).
        assert_eq!(nature(Mercury, &with(&[(Moon, 70.0)])), Nature::Benefic);
        // Küçülen Ay kötücül sayılır (Güneş 100°, Ay 70°: uzanım 330°).
        assert_eq!(
            nature(Mercury, &with(&[(Sun, 100.0), (Moon, 70.0)])),
            Nature::Malefic
        );
        // Komşu burçtaki kötücül etkilemez (Mars 59,9° Boğa, Satürn 90° Yengeç).
        assert_eq!(
            nature(Mercury, &with(&[(Mars, 59.9), (Saturn, 90.0)])),
            Nature::Benefic
        );
    }

    #[test]
    fn kuja_dosha_houses_from_each_reference() {
        // Mars Akrep'te (220°). Referans burcu r için Mars'ın evi = Akrep'in r'den sayısı.
        // Lagna Koç (0°): 8. ev → dosha. Ay Yengeç (100°): 5. ev → yok. Venüs Boğa (40°): 7. ev
        // → dosha.
        assert_eq!(
            kuja_dosha(&with(&[])),
            KujaDosha {
                from_lagna: Some(true),
                from_moon: false,
                from_venus: true
            }
        );
        // On iki evin tamamı, üç referans için: Mars Koç'ta sabit, referans geriye kayar.
        for house in 1..=12u8 {
            // Mars'ın referanstan evi `house` olsun: referans burcu = Koç − (house − 1).
            let reference = f64::from(12 - (house - 1)) * 30.0 + 15.0;
            let expected = [1, 2, 4, 7, 8, 12].contains(&house);
            let mut lons = BASE;
            lons[Mars as usize] = 5.0;
            lons[Moon as usize] = reference;
            lons[Venus as usize] = reference;
            let p = Positions::synthetic(Some(reference), lons);
            assert_eq!(p.house_from(Mars, p.sign_of(Moon)), house);
            assert_eq!(
                kuja_dosha(&p),
                KujaDosha {
                    from_lagna: Some(expected),
                    from_moon: expected,
                    from_venus: expected
                },
                "house {house}"
            );
        }
    }

    #[test]
    fn kuja_dosha_from_lagna_is_none_without_a_birth_time() {
        let p = with(&[]).without_lagna();
        assert_eq!(
            kuja_dosha(&p),
            KujaDosha {
                from_lagna: None,
                from_moon: false,
                from_venus: true
            }
        );
    }
}
