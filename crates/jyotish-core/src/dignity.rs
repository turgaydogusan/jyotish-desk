//! BPHS dignity (böl. 3, R. Santhanam çevirisi): yücelme ve düşüş, moolatrikona,
//! kendi burcu ve panchadha maitri (doğal + geçici dostluk). Düğümlere dignity atanmaz.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::zodiac::Sign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dignity {
    Exalted,
    Moolatrikona,
    OwnSign,
    GreatFriend,
    Friend,
    Neutral,
    Enemy,
    GreatEnemy,
    Debilitated,
}

/// Doğal (naisargika) ilişki.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    Friend,
    Neutral,
    Enemy,
}

pub fn sign_lord(sign: Sign) -> Graha {
    match sign {
        Sign::Aries | Sign::Scorpio => Graha::Mars,
        Sign::Taurus | Sign::Libra => Graha::Venus,
        Sign::Gemini | Sign::Virgo => Graha::Mercury,
        Sign::Cancer => Graha::Moon,
        Sign::Leo => Graha::Sun,
        Sign::Sagittarius | Sign::Pisces => Graha::Jupiter,
        Sign::Capricorn | Sign::Aquarius => Graha::Saturn,
    }
}

/// Derin yücelme noktası (burç, burç içi derece).
pub fn exaltation_point(graha: Graha) -> Option<(Sign, f64)> {
    match graha {
        Graha::Sun => Some((Sign::Aries, 10.0)),
        Graha::Moon => Some((Sign::Taurus, 3.0)),
        Graha::Mars => Some((Sign::Capricorn, 28.0)),
        Graha::Mercury => Some((Sign::Virgo, 15.0)),
        Graha::Jupiter => Some((Sign::Cancer, 5.0)),
        Graha::Venus => Some((Sign::Pisces, 27.0)),
        Graha::Saturn => Some((Sign::Libra, 20.0)),
        Graha::Rahu | Graha::Ketu => None,
    }
}

/// Derin düşüş: yücelme noktasının tam karşısı.
pub fn debilitation_point(graha: Graha) -> Option<(Sign, f64)> {
    exaltation_point(graha).map(|(s, deg)| (Sign::from_index(s.index() + 6), deg))
}

/// Moolatrikona burcu ve `[başlangıç, bitiş)` derece aralığı.
pub fn moolatrikona(graha: Graha) -> Option<(Sign, f64, f64)> {
    match graha {
        Graha::Sun => Some((Sign::Leo, 0.0, 20.0)),
        Graha::Moon => Some((Sign::Taurus, 3.0, 30.0)),
        Graha::Mars => Some((Sign::Aries, 0.0, 12.0)),
        Graha::Mercury => Some((Sign::Virgo, 15.0, 20.0)),
        Graha::Jupiter => Some((Sign::Sagittarius, 0.0, 10.0)),
        Graha::Venus => Some((Sign::Libra, 0.0, 15.0)),
        Graha::Saturn => Some((Sign::Aquarius, 0.0, 20.0)),
        Graha::Rahu | Graha::Ketu => None,
    }
}

pub fn own_signs(graha: Graha) -> &'static [Sign] {
    match graha {
        Graha::Sun => &[Sign::Leo],
        Graha::Moon => &[Sign::Cancer],
        Graha::Mars => &[Sign::Aries, Sign::Scorpio],
        Graha::Mercury => &[Sign::Gemini, Sign::Virgo],
        Graha::Jupiter => &[Sign::Sagittarius, Sign::Pisces],
        Graha::Venus => &[Sign::Taurus, Sign::Libra],
        Graha::Saturn => &[Sign::Capricorn, Sign::Aquarius],
        Graha::Rahu | Graha::Ketu => &[],
    }
}

/// `of`'un `toward`'a doğal ilişkisi (BPHS). Kendisi ve düğümler tablo dışıdır: `Neutral`.
pub fn natural_relation(of: Graha, toward: Graha) -> Relation {
    use Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};
    use Relation::{Enemy, Friend, Neutral};
    match (of, toward) {
        (Sun, Moon | Mars | Jupiter) => Friend,
        (Sun, Mercury) => Neutral,
        (Sun, Venus | Saturn) => Enemy,
        (Moon, Sun | Mercury) => Friend,
        (Moon, Mars | Jupiter | Venus | Saturn) => Neutral,
        (Mars, Sun | Moon | Jupiter) => Friend,
        (Mars, Venus | Saturn) => Neutral,
        (Mars, Mercury) => Enemy,
        (Mercury, Sun | Venus) => Friend,
        (Mercury, Mars | Jupiter | Saturn) => Neutral,
        (Mercury, Moon) => Enemy,
        (Jupiter, Sun | Moon | Mars) => Friend,
        (Jupiter, Saturn) => Neutral,
        (Jupiter, Mercury | Venus) => Enemy,
        (Venus, Mercury | Saturn) => Friend,
        (Venus, Mars | Jupiter) => Neutral,
        (Venus, Sun | Moon) => Enemy,
        (Saturn, Mercury | Venus) => Friend,
        (Saturn, Jupiter) => Neutral,
        (Saturn, Sun | Moon | Mars) => Enemy,
        _ => Neutral,
    }
}

/// Geçici (tatkalika) dostluk: `toward_sign`, `of_sign`'dan sayınca 2, 3, 4, 10, 11
/// veya 12. burçtaysa dost, aksi halde düşman.
pub fn temporal_friend(of_sign: Sign, toward_sign: Sign) -> bool {
    matches!(toward_sign.house_from(of_sign), 2 | 3 | 4 | 10 | 11 | 12)
}

/// Panchadha maitri: doğal ve geçici ilişkinin birleşimi.
pub fn compound_relation(natural: Relation, temporal_friend: bool) -> Dignity {
    match (natural, temporal_friend) {
        (Relation::Friend, true) => Dignity::GreatFriend,
        (Relation::Friend, false) => Dignity::Neutral,
        (Relation::Neutral, true) => Dignity::Friend,
        (Relation::Neutral, false) => Dignity::Enemy,
        (Relation::Enemy, true) => Dignity::Neutral,
        (Relation::Enemy, false) => Dignity::GreatEnemy,
    }
}

/// `graha`'nın `sign` içinde `deg_in_sign` derecedeki dignity'si. `lord_sign`, `sign`
/// burcunun lordunun haritadaki burcudur (geçici dostluk için). Düğümler `None`.
///
/// Sıra: yücelme, düşüş, moolatrikona, kendi burcu, panchadha maitri. Ay ve Merkür'ün
/// yücelme burcu moolatrikona burcuyla aynıdır; orada yücelme yalnızca moolatrikona
/// aralığından önceki derecelerde geçerlidir (Ay Boğa 0–3°, Merkür Başak 0–15°).
pub fn dignity(graha: Graha, sign: Sign, deg_in_sign: f64, lord_sign: Sign) -> Option<Dignity> {
    let (exalt_sign, _) = exaltation_point(graha)?;
    let mt = moolatrikona(graha);
    if sign == exalt_sign {
        let exalted = match mt {
            Some((mt_sign, mt_from, _)) if mt_sign == sign => deg_in_sign < mt_from,
            _ => true,
        };
        if exalted {
            return Some(Dignity::Exalted);
        }
    }
    if sign == Sign::from_index(exalt_sign.index() + 6) {
        return Some(Dignity::Debilitated);
    }
    if let Some((mt_sign, from, to)) = mt {
        if sign == mt_sign && deg_in_sign >= from && deg_in_sign < to {
            return Some(Dignity::Moolatrikona);
        }
    }
    if own_signs(graha).contains(&sign) {
        return Some(Dignity::OwnSign);
    }
    let lord = sign_lord(sign);
    Some(compound_relation(
        natural_relation(graha, lord),
        temporal_friend(sign, lord_sign),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    /// Lord burcu dignity'yi etkilemeyen durumlar için.
    fn d(g: Graha, s: Sign, deg: f64) -> Option<Dignity> {
        dignity(g, s, deg, Aries)
    }

    #[test]
    fn sign_lords() {
        let lords: Vec<_> = Sign::ALL.into_iter().map(sign_lord).collect();
        assert_eq!(
            lords,
            [
                Mars, Venus, Mercury, Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Saturn,
                Jupiter
            ]
        );
    }

    #[test]
    fn exaltation_and_debilitation_points() {
        assert_eq!(exaltation_point(Sun), Some((Aries, 10.0)));
        assert_eq!(exaltation_point(Moon), Some((Taurus, 3.0)));
        assert_eq!(exaltation_point(Mars), Some((Capricorn, 28.0)));
        assert_eq!(exaltation_point(Mercury), Some((Virgo, 15.0)));
        assert_eq!(exaltation_point(Jupiter), Some((Cancer, 5.0)));
        assert_eq!(exaltation_point(Venus), Some((Pisces, 27.0)));
        assert_eq!(exaltation_point(Saturn), Some((Libra, 20.0)));
        assert_eq!(debilitation_point(Sun), Some((Libra, 10.0)));
        assert_eq!(debilitation_point(Moon), Some((Scorpio, 3.0)));
        assert_eq!(debilitation_point(Mars), Some((Cancer, 28.0)));
        assert_eq!(debilitation_point(Mercury), Some((Pisces, 15.0)));
        assert_eq!(debilitation_point(Jupiter), Some((Capricorn, 5.0)));
        assert_eq!(debilitation_point(Venus), Some((Virgo, 27.0)));
        assert_eq!(debilitation_point(Saturn), Some((Aries, 20.0)));
        assert_eq!(exaltation_point(Rahu), None);
        assert_eq!(debilitation_point(Ketu), None);
    }

    #[test]
    fn exalted_and_debilitated_by_sign() {
        assert_eq!(d(Sun, Aries, 25.0), Some(Dignity::Exalted));
        assert_eq!(d(Sun, Libra, 5.0), Some(Dignity::Debilitated));
        assert_eq!(d(Mars, Capricorn, 1.0), Some(Dignity::Exalted));
        assert_eq!(d(Mars, Cancer, 29.0), Some(Dignity::Debilitated));
        assert_eq!(d(Jupiter, Cancer, 20.0), Some(Dignity::Exalted));
        assert_eq!(d(Jupiter, Capricorn, 0.0), Some(Dignity::Debilitated));
        assert_eq!(d(Venus, Pisces, 0.5), Some(Dignity::Exalted));
        assert_eq!(d(Venus, Virgo, 15.0), Some(Dignity::Debilitated));
        assert_eq!(d(Saturn, Libra, 29.0), Some(Dignity::Exalted));
        assert_eq!(d(Saturn, Aries, 0.0), Some(Dignity::Debilitated));
        assert_eq!(d(Moon, Scorpio, 20.0), Some(Dignity::Debilitated));
        assert_eq!(d(Mercury, Pisces, 1.0), Some(Dignity::Debilitated));
    }

    #[test]
    fn moon_and_mercury_exaltation_yields_to_moolatrikona() {
        assert_eq!(d(Moon, Taurus, 2.9), Some(Dignity::Exalted));
        assert_eq!(d(Moon, Taurus, 3.0), Some(Dignity::Moolatrikona));
        assert_eq!(d(Moon, Taurus, 29.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Mercury, Virgo, 14.9), Some(Dignity::Exalted));
        assert_eq!(d(Mercury, Virgo, 15.0), Some(Dignity::Moolatrikona));
        assert_eq!(d(Mercury, Virgo, 19.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Mercury, Virgo, 20.0), Some(Dignity::OwnSign));
    }

    #[test]
    fn moolatrikona_ranges_then_own_sign() {
        assert_eq!(d(Sun, Leo, 19.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Sun, Leo, 20.0), Some(Dignity::OwnSign));
        assert_eq!(d(Mars, Aries, 11.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Mars, Aries, 12.0), Some(Dignity::OwnSign));
        assert_eq!(d(Jupiter, Sagittarius, 9.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Jupiter, Sagittarius, 10.0), Some(Dignity::OwnSign));
        assert_eq!(d(Venus, Libra, 14.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Venus, Libra, 15.0), Some(Dignity::OwnSign));
        assert_eq!(d(Saturn, Aquarius, 19.9), Some(Dignity::Moolatrikona));
        assert_eq!(d(Saturn, Aquarius, 20.0), Some(Dignity::OwnSign));
    }

    #[test]
    fn own_signs_table() {
        assert_eq!(own_signs(Sun), [Leo]);
        assert_eq!(own_signs(Moon), [Cancer]);
        assert_eq!(own_signs(Mars), [Aries, Scorpio]);
        assert_eq!(own_signs(Mercury), [Gemini, Virgo]);
        assert_eq!(own_signs(Jupiter), [Sagittarius, Pisces]);
        assert_eq!(own_signs(Venus), [Taurus, Libra]);
        assert_eq!(own_signs(Saturn), [Capricorn, Aquarius]);
        assert!(own_signs(Rahu).is_empty());
        assert_eq!(d(Moon, Cancer, 10.0), Some(Dignity::OwnSign));
        assert_eq!(d(Mars, Scorpio, 10.0), Some(Dignity::OwnSign));
        assert_eq!(d(Jupiter, Pisces, 10.0), Some(Dignity::OwnSign));
        assert_eq!(d(Venus, Taurus, 10.0), Some(Dignity::OwnSign));
        assert_eq!(d(Saturn, Capricorn, 10.0), Some(Dignity::OwnSign));
    }

    #[test]
    fn nodes_have_no_dignity() {
        for s in Sign::ALL {
            assert_eq!(d(Rahu, s, 10.0), None);
            assert_eq!(d(Ketu, s, 10.0), None);
        }
    }

    #[test]
    fn natural_friendship_table_matches_bphs() {
        let planets = [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn];
        // Satır: graha; sütun sırası Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn.
        // F dost, N nötr, E düşman, '-' kendisi.
        let table = [
            "-FFNFEE", // Sun
            "F-NFNNN", // Moon
            "FF-EFNN", // Mars
            "FEN-NFN", // Mercury
            "FFFE-EN", // Jupiter
            "EENFN-F", // Venus
            "EEEFNF-", // Saturn
        ];
        for (row, of) in table.iter().zip(planets) {
            for (c, toward) in row.chars().zip(planets) {
                let want = match c {
                    'F' => Relation::Friend,
                    'N' => Relation::Neutral,
                    'E' => Relation::Enemy,
                    _ => continue,
                };
                assert_eq!(natural_relation(of, toward), want, "{of:?} -> {toward:?}");
            }
        }
    }

    #[test]
    fn temporal_friendship_by_house() {
        let friends: Vec<u8> = Sign::ALL
            .into_iter()
            .filter(|&s| temporal_friend(Aries, s))
            .map(|s| s.house_from(Aries))
            .collect();
        assert_eq!(friends, [2, 3, 4, 10, 11, 12]);
    }

    #[test]
    fn compound_relation_table() {
        use Relation as R;
        assert_eq!(compound_relation(R::Friend, true), Dignity::GreatFriend);
        assert_eq!(compound_relation(R::Friend, false), Dignity::Neutral);
        assert_eq!(compound_relation(R::Neutral, true), Dignity::Friend);
        assert_eq!(compound_relation(R::Neutral, false), Dignity::Enemy);
        assert_eq!(compound_relation(R::Enemy, true), Dignity::Neutral);
        assert_eq!(compound_relation(R::Enemy, false), Dignity::GreatEnemy);
    }

    #[test]
    fn compound_dignity_examples() {
        // Güneş İkizler'de; lord Merkür (Güneş için doğal nötr).
        assert_eq!(dignity(Sun, Gemini, 10.0, Cancer), Some(Dignity::Friend)); // 2. ev: geçici dost
        assert_eq!(dignity(Sun, Gemini, 10.0, Gemini), Some(Dignity::Enemy)); // 1. ev: geçici düşman
                                                                              // Satürn Aslan'da; lord Güneş (Satürn için doğal düşman).
        assert_eq!(dignity(Saturn, Leo, 10.0, Leo), Some(Dignity::GreatEnemy));
        assert_eq!(dignity(Saturn, Leo, 10.0, Virgo), Some(Dignity::Neutral));
        // Mars Yay'da; lord Jüpiter (Mars için doğal dost).
        assert_eq!(
            dignity(Mars, Sagittarius, 5.0, Capricorn),
            Some(Dignity::GreatFriend)
        );
        assert_eq!(
            dignity(Mars, Sagittarius, 5.0, Gemini),
            Some(Dignity::Neutral)
        );
    }
}
