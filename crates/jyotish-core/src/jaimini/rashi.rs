//! Burç türleri, sayım yönü ve rashi drishti (ileri modüller spec'i §5, §6).
//!
//! Kaynak: Jaimini Upadesha Sutraları, bakışlar bölümü. Yürük burç, kendisinden sonraki sabit
//! burç dışındaki üç sabit burca; sabit burç, kendisinden önceki yürük burç dışındaki üç yürük
//! burca; çift doğalı burç diğer üç çift doğalı burca bakar. Bir burçtaki grahalar o burcun
//! baktığı burçlara ve oradaki grahalara bakar. Burçlar rasi çerçevesindedir (K5).

use serde::{Deserialize, Serialize};

use crate::drishti::SignAspect;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Burcun türü: yürük (chara), sabit (sthira), çift doğalı (dwiswabhava).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SignKind {
    Movable,
    Fixed,
    Dual,
}

/// Koç, Yengeç, Terazi, Oğlak yürük; Boğa, Aslan, Akrep, Kova sabit; diğer dördü çift doğalı.
pub fn sign_kind(sign: Sign) -> SignKind {
    match sign.index() % 3 {
        0 => SignKind::Movable,
        1 => SignKind::Fixed,
        _ => SignKind::Dual,
    }
}

/// Jaimini sayım yönü: Koç, Boğa, İkizler, Terazi, Akrep ve Yay'dan ileri (zodyak sırasıyla),
/// diğer altı burçtan geri sayılır. Eş lord seçiminin 5. ölçütü ve chara dasha (Alt proje 3.3)
/// bunu kullanır.
pub fn counts_forward(sign: Sign) -> bool {
    matches!(
        sign,
        Sign::Aries | Sign::Taurus | Sign::Gemini | Sign::Libra | Sign::Scorpio | Sign::Sagittarius
    )
}

/// `from`'dan `to`'ya, `from`'un sayım yönünde kapsayıcı sayım, 1..=12 (aynı burç 1).
pub fn directed_count(from: Sign, to: Sign) -> u8 {
    if counts_forward(from) {
        to.house_from(from)
    } else {
        from.house_from(to)
    }
}

/// `from` burcunun rashi drishti ile baktığı üç burç, zodyak sırasıyla `from`'dan başlayarak.
pub fn rashi_drishti_signs(from: Sign) -> [Sign; 3] {
    let offsets: [usize; 3] = match sign_kind(from) {
        SignKind::Movable => [4, 7, 10],
        SignKind::Fixed => [2, 5, 8],
        SignKind::Dual => [3, 6, 9],
    };
    offsets.map(|o| Sign::from_index(from.index() + o))
}

/// `matrix[from][to]`: `from` burcu `to` burcuna bakıyor mu. Dizinler `Sign::ALL` sırasıdır.
pub fn rashi_drishti_matrix() -> [[bool; 12]; 12] {
    let mut matrix = [[false; 12]; 12];
    for from in Sign::ALL {
        for to in rashi_drishti_signs(from) {
            matrix[from.index()][to.index()] = true;
        }
    }
    matrix
}

/// Dokuz grahanın rashi drishtisi: `Graha::ALL` sırasında, her graha için üç hedef burç ve
/// oradaki grahalar. Her zaman 27 kayıt.
pub fn graha_rashi_drishti(positions: &Positions) -> Vec<SignAspect> {
    Graha::ALL
        .into_iter()
        .flat_map(|graha| {
            rashi_drishti_signs(positions.sign_of(graha))
                .into_iter()
                .map(move |target| SignAspect {
                    graha,
                    target,
                    occupants: positions.occupants(target),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sign::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım, spec §13): her burcun baktığı üç burç.
    const HAND_TABLE: [(Sign, [Sign; 3]); 12] = [
        (Aries, [Leo, Scorpio, Aquarius]),
        (Taurus, [Cancer, Libra, Capricorn]),
        (Gemini, [Virgo, Sagittarius, Pisces]),
        (Cancer, [Scorpio, Aquarius, Taurus]),
        (Leo, [Libra, Capricorn, Aries]),
        (Virgo, [Sagittarius, Pisces, Gemini]),
        (Libra, [Aquarius, Taurus, Leo]),
        (Scorpio, [Capricorn, Aries, Cancer]),
        (Sagittarius, [Pisces, Gemini, Virgo]),
        (Capricorn, [Taurus, Leo, Scorpio]),
        (Aquarius, [Aries, Cancer, Libra]),
        (Pisces, [Gemini, Virgo, Sagittarius]),
    ];

    #[test]
    fn sign_kinds_follow_the_zodiac_in_threes() {
        for s in [Aries, Cancer, Libra, Capricorn] {
            assert_eq!(sign_kind(s), SignKind::Movable, "{s:?}");
        }
        for s in [Taurus, Leo, Scorpio, Aquarius] {
            assert_eq!(sign_kind(s), SignKind::Fixed, "{s:?}");
        }
        for s in [Gemini, Virgo, Sagittarius, Pisces] {
            assert_eq!(sign_kind(s), SignKind::Dual, "{s:?}");
        }
    }

    #[test]
    fn rashi_drishti_matches_the_hand_table() {
        for (from, targets) in HAND_TABLE {
            assert_eq!(rashi_drishti_signs(from), targets, "{from:?}");
        }
        let matrix = rashi_drishti_matrix();
        for (from, targets) in HAND_TABLE {
            for to in Sign::ALL {
                assert_eq!(
                    matrix[from.index()][to.index()],
                    targets.contains(&to),
                    "{from:?} → {to:?}"
                );
            }
        }
    }

    #[test]
    fn rashi_drishti_is_symmetric_and_skips_the_adjacent_sign() {
        let matrix = rashi_drishti_matrix();
        for from in Sign::ALL {
            let row = matrix[from.index()];
            assert_eq!(row.iter().filter(|&&b| b).count(), 3, "{from:?}");
            assert!(!row[from.index()], "{from:?} does not aspect itself");
            for to in Sign::ALL {
                assert_eq!(
                    matrix[from.index()][to.index()],
                    matrix[to.index()][from.index()],
                    "{from:?} ↔ {to:?}"
                );
                if !row[to.index()] {
                    continue;
                }
                match sign_kind(from) {
                    SignKind::Movable => assert_eq!(sign_kind(to), SignKind::Fixed),
                    SignKind::Fixed => assert_eq!(sign_kind(to), SignKind::Movable),
                    SignKind::Dual => assert_eq!(sign_kind(to), SignKind::Dual),
                }
            }
        }
        // Bitişik burca bakış yoktur: yürük burç için sonraki, sabit burç için önceki burç.
        for s in [Aries, Cancer, Libra, Capricorn] {
            assert!(!matrix[s.index()][(s.index() + 1) % 12], "{s:?}");
        }
        for s in [Taurus, Leo, Scorpio, Aquarius] {
            assert!(!matrix[s.index()][(s.index() + 11) % 12], "{s:?}");
        }
    }

    #[test]
    fn counting_direction_and_directed_count() {
        let forward: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| counts_forward(s))
            .collect();
        assert_eq!(
            forward,
            [Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius]
        );
        // Akrep'ten ileri: Akrep 1, Yay 2, Oğlak 3.
        assert_eq!(directed_count(Scorpio, Scorpio), 1);
        assert_eq!(directed_count(Scorpio, Capricorn), 3);
        assert_eq!(directed_count(Scorpio, Libra), 12);
        // Kova'dan geri: Kova 1, Oğlak 2, Yay 3; Balık 12.
        assert_eq!(directed_count(Aquarius, Aquarius), 1);
        assert_eq!(directed_count(Aquarius, Sagittarius), 3);
        assert_eq!(directed_count(Aquarius, Pisces), 12);
    }

    #[test]
    fn grahas_aspect_the_signs_their_sign_aspects() {
        // Güneş ve Merkür Koç'ta, Ay Yengeç'te, Mars Aslan'da, Jüpiter Yay'da, Venüs Boğa'da,
        // Satürn Kova'da, Rahu Akrep'te, Ketu Boğa'da.
        let p = Positions::synthetic(
            None,
            [10.0, 95.0, 130.0, 20.0, 250.0, 40.0, 300.0, 220.0, 40.0],
        );
        let aspects = graha_rashi_drishti(&p);
        assert_eq!(aspects.len(), 27);
        for (i, a) in aspects.iter().enumerate() {
            assert_eq!(a.graha, Graha::ALL[i / 3]);
            assert_eq!(a.target, rashi_drishti_signs(p.sign_of(a.graha))[i % 3]);
            assert_eq!(a.occupants, p.occupants(a.target));
        }
        // Güneş Koç'tan Aslan'a (Mars), Akrep'e (Rahu) ve Kova'ya (Satürn) bakar.
        assert_eq!(aspects[0].target, Leo);
        assert_eq!(aspects[0].occupants, [Graha::Mars]);
        assert_eq!(aspects[1].occupants, [Graha::Rahu]);
        assert_eq!(aspects[2].occupants, [Graha::Saturn]);
        // Jüpiter Yay'dan Balık, İkizler ve Başak'a bakar: üçü de boş.
        assert!(aspects[12..15].iter().all(|a| a.occupants.is_empty()));
    }
}
