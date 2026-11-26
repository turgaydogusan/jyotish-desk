//! Yerleşime dayalı yogalar (spec §8): Pancha Mahapurusha, Neecha bhanga, nabhasa sayı ve
//! ashraya yogaları ve Kuja dosha.

use super::{
    in_houses_from, occupied_houses, YogaMatch, COND_DISPOSITOR_ASPECT, COND_EXALTATION,
    COND_EXALTED_GRAHA_IN_KENDRA, COND_FROM_LAGNA, COND_FROM_MOON, COND_FROM_VENUS, COND_OWN_SIGN,
    COND_SIGN_LORD_IN_KENDRA, KENDRA_HOUSES, YOGA_GRAHAS,
};
use crate::dignity::{debilitation_point, exaltation_point, own_signs, sign_lord};
use crate::drishti::graha_aspect_signs;
use crate::graha::Graha;
use crate::jaimini::{sign_kind, SignKind};
use crate::nature::kuja_dosha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Pancha Mahapurusha: graha kendi ya da yücelme burcunda ve lagnadan kendrada. Yalnızca
/// lagnadan sayılır. Koşullar grahanın burcunun niteliğini yazar (Merkür Başak'ta ikisini de).
pub(super) fn mahapurusha(positions: &Positions, graha: Graha) -> Vec<YogaMatch> {
    let Some(house) = positions.house_of(graha) else {
        return Vec::new();
    };
    if !KENDRA_HOUSES.contains(&house) {
        return Vec::new();
    }
    let sign = positions.sign_of(graha);
    let mut conditions = Vec::new();
    if own_signs(graha).contains(&sign) {
        conditions.push(COND_OWN_SIGN);
    }
    if exaltation_point(graha).is_some_and(|(s, _)| s == sign) {
        conditions.push(COND_EXALTATION);
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: vec![graha],
        houses: vec![house],
        conditions,
        ..YogaMatch::default()
    }]
}

/// `sign` burcunda yücelen graha (yedi graha içinde); Akrep'te yoktur.
pub(crate) fn exalted_in(sign: Sign) -> Option<Graha> {
    YOGA_GRAHAS
        .into_iter()
        .find(|&g| exaltation_point(g).is_some_and(|(s, _)| s == sign))
}

/// `graha` lagnadan ya da Ay'dan kendrada mı. Ay kendisinden sayılmaz: Ay için yalnızca lagna
/// referansı geçerlidir.
fn in_kendra_from_lagna_or_moon(positions: &Positions, graha: Graha) -> bool {
    let from_lagna = positions
        .house_of(graha)
        .is_some_and(|h| KENDRA_HOUSES.contains(&h));
    let from_moon = graha != Graha::Moon
        && in_houses_from(
            positions,
            graha,
            positions.sign_of(Graha::Moon),
            &KENDRA_HOUSES,
        );
    from_lagna || from_moon
}

/// Neecha bhanga: düşüş burcundaki her graha için sağlanan koşullar. Hiçbir koşul sağlanmıyorsa
/// eşleşme yoktur. Düşüş burcu derecesinden bağımsızdır (burcun tamamı).
pub(super) fn neecha_bhanga(positions: &Positions) -> Vec<YogaMatch> {
    let mut out = Vec::new();
    for graha in YOGA_GRAHAS {
        let Some((fall, _)) = debilitation_point(graha) else {
            continue;
        };
        if positions.sign_of(graha) != fall {
            continue;
        }
        // Grahanın dispozitörü, bulunduğu (düşüş) burcun lordudur.
        let lord = sign_lord(fall);
        let mut conditions = Vec::new();
        if in_kendra_from_lagna_or_moon(positions, lord) {
            conditions.push(COND_SIGN_LORD_IN_KENDRA);
        }
        if exalted_in(fall).is_some_and(|g| in_kendra_from_lagna_or_moon(positions, g)) {
            conditions.push(COND_EXALTED_GRAHA_IN_KENDRA);
        }
        if graha_aspect_signs(lord, positions.sign_of(lord)).contains(&fall) {
            conditions.push(COND_DISPOSITOR_ASPECT);
        }
        if !conditions.is_empty() {
            out.push(YogaMatch {
                grahas: vec![graha],
                houses: occupied_houses(positions, &[graha]),
                conditions,
                ..YogaMatch::default()
            });
        }
    }
    out
}

/// Yedi grahanın bulunduğu farklı burç sayısı, 1..=7.
pub fn occupied_sign_count(positions: &Positions) -> u8 {
    let mut signs: Vec<Sign> = YOGA_GRAHAS
        .into_iter()
        .map(|g| positions.sign_of(g))
        .collect();
    signs.sort_unstable();
    signs.dedup();
    signs.len() as u8
}

fn all_seven(positions: &Positions) -> Vec<YogaMatch> {
    vec![YogaMatch {
        grahas: YOGA_GRAHAS.to_vec(),
        houses: occupied_houses(positions, &YOGA_GRAHAS),
        ..YogaMatch::default()
    }]
}

/// Nabhasa sayı yogası: yedi graha tam `count` burca dağılmış.
pub(super) fn nabhasa_count(positions: &Positions, count: u8) -> Vec<YogaMatch> {
    if occupied_sign_count(positions) != count {
        return Vec::new();
    }
    all_seven(positions)
}

/// Nabhasa ashraya yogası: yedi grahanın hepsi `kind` türünden burçlarda.
pub(super) fn nabhasa_ashraya(positions: &Positions, kind: SignKind) -> Vec<YogaMatch> {
    if !YOGA_GRAHAS
        .into_iter()
        .all(|g| sign_kind(positions.sign_of(g)) == kind)
    {
        return Vec::new();
    }
    all_seven(positions)
}

/// Kuja dosha: 3.1'deki `kuja_dosha` olgusu; sağlayan referanslar koşul olarak listelenir.
/// İptal kuralı uygulanmaz.
pub(super) fn kuja(positions: &Positions) -> Vec<YogaMatch> {
    let dosha = kuja_dosha(positions);
    let mut conditions = Vec::new();
    if dosha.from_lagna == Some(true) {
        conditions.push(COND_FROM_LAGNA);
    }
    if dosha.from_moon {
        conditions.push(COND_FROM_MOON);
    }
    if dosha.from_venus {
        conditions.push(COND_FROM_VENUS);
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: vec![Graha::Mars],
        houses: occupied_houses(positions, &[Graha::Mars]),
        conditions,
        ..YogaMatch::default()
    }]
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::*;
    use Graha::*;
    use Sign::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): mahapurusha yogası, grahası, kendi
    /// burçları ve yücelme burcu.
    const HAND_MAHAPURUSHA: [(&str, Graha, &[Sign], Sign); 5] = [
        ("mahapurusha.ruchaka", Mars, &[Aries, Scorpio], Capricorn),
        ("mahapurusha.bhadra", Mercury, &[Gemini, Virgo], Virgo),
        ("mahapurusha.hamsa", Jupiter, &[Sagittarius, Pisces], Cancer),
        ("mahapurusha.malavya", Venus, &[Taurus, Libra], Pisces),
        ("mahapurusha.shasha", Saturn, &[Capricorn, Aquarius], Libra),
    ];

    #[test]
    fn mahapurusha_matches_the_hand_table_for_every_lagna_and_sign() {
        for (id, graha, own, exalted) in HAND_MAHAPURUSHA {
            for lagna in 0..12usize {
                for sign in 0..12usize {
                    let mut signs = [0usize; 9];
                    signs[graha as usize] = sign;
                    let p = chart(Some(lagna), signs);
                    let house = (sign + 12 - lagna) % 12 + 1;
                    let s = Sign::from_index(sign);
                    let strong = own.contains(&s) || s == exalted;
                    let want = strong && [1, 4, 7, 10].contains(&house);
                    let got = matches(id, &p);
                    assert_eq!(!got.is_empty(), want, "{id}: lagna {lagna}, sign {sign}");
                    if let Some(m) = got.first() {
                        assert_eq!(m.grahas, [graha]);
                        assert_eq!(m.houses, [house as u8]);
                        let mut conditions = Vec::new();
                        if own.contains(&s) {
                            conditions.push(COND_OWN_SIGN);
                        }
                        if s == exalted {
                            conditions.push(COND_EXALTATION);
                        }
                        assert_eq!(m.conditions, conditions, "{id}");
                    }
                    // Lagna yoksa değerlendirilmez (yalnızca lagnadan sayılır).
                    assert!(!forms(id, &p.clone().without_lagna()));
                }
            }
        }
    }

    /// Elle yazılmış tablo: graha, düşüş burcu, o burcun lordu, o burçta yücelen graha.
    const HAND_FALL: [(Graha, Sign, Graha, Option<Graha>); 7] = [
        (Sun, Libra, Venus, Some(Saturn)),
        (Moon, Scorpio, Mars, None),
        (Mars, Cancer, Moon, Some(Jupiter)),
        (Mercury, Pisces, Jupiter, Some(Venus)),
        (Jupiter, Capricorn, Saturn, Some(Mars)),
        (Venus, Virgo, Mercury, Some(Mercury)),
        (Saturn, Aries, Mars, Some(Sun)),
    ];

    #[test]
    fn fall_signs_lords_and_exalted_grahas_match_the_hand_table() {
        for (graha, fall, lord, exalted) in HAND_FALL {
            assert_eq!(debilitation_point(graha).map(|(s, _)| s), Some(fall));
            assert_eq!(sign_lord(fall), lord, "{graha:?}");
            assert_eq!(exalted_in(fall), exalted, "{graha:?}");
        }
        // Yedi yücelme burcu dışındaki burçlarda yücelen graha yoktur.
        let exalting: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| exalted_in(s).is_some())
            .collect();
        assert_eq!(
            exalting,
            [Aries, Taurus, Cancer, Virgo, Libra, Capricorn, Pisces]
        );
    }

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: hiçbir graha düşüş burcunda değil. Güneş Aslan, Ay Boğa, Mars İkizler, Merkür
    /// Aslan, Jüpiter Kova, Venüs Yengeç, Satürn Yay.
    const BASE: [usize; 9] = [4, 1, 2, 4, 10, 3, 8, 5, 11];

    fn with(lagna: Option<usize>, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(lagna, signs)
    }

    const NB: &str = "neecha_bhanga.neecha_bhanga";

    #[test]
    fn neecha_bhanga_lists_the_conditions_that_hold() {
        assert!(!forms(NB, &with(Some(0), &[])));
        // Güneş Terazi'de (düşüş). Lord Venüs Yengeç'te: Koç lagnadan 4. ev (kendra). Terazi'de
        // yücelen Satürn Yay'da: lagnadan 9., Ay'dan (Boğa) 8. Venüs Yengeç'ten Terazi'ye
        // bakmaz (4. burç).
        let p = with(Some(0), &[(Sun, 6)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].grahas, [Sun]);
        assert_eq!(m[0].houses, [7]);
        assert_eq!(m[0].conditions, [COND_SIGN_LORD_IN_KENDRA]);
        // Satürn Aslan'a alınır: Ay'dan (Boğa) 4. ev. Venüs Koç'a alınır: lagnadan 1. ev ve
        // Terazi'ye 7. bakış. Üç koşul birden.
        let p = with(Some(0), &[(Sun, 6), (Saturn, 4), (Venus, 0)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [
                COND_SIGN_LORD_IN_KENDRA,
                COND_EXALTED_GRAHA_IN_KENDRA,
                COND_DISPOSITOR_ASPECT
            ]
        );
        // Venüs Başak'a alınır (lagnadan 6., Ay'dan 5. ev). Güneş için hiçbir koşul yok: lord
        // Venüs kendrada değil ve Başak'tan Terazi'ye bakmıyor (2. burç); Satürn Yay'da kendrada
        // değil. Güneş listelenmez. Venüs ise Başak'ta düşüştedir ve kendi koşulları sağlanır.
        let p = with(Some(0), &[(Sun, 6), (Venus, 5)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 1, "only Venus has a cancelling condition");
        assert_eq!(m[0].grahas, [Venus]);
        // Başak'ın lordu da Başak'ta yücelen graha da Merkür'dür: Aslan'da, Ay'dan 4. evde.
        // Merkür Aslan'dan Başak'a bakmaz (2. burç).
        assert_eq!(
            m[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_EXALTED_GRAHA_IN_KENDRA]
        );
    }

    #[test]
    fn neecha_bhanga_special_cases() {
        // Ay Akrep'te: Akrep'te yücelen graha yok, ikinci koşul uygulanmaz. Lord Mars İkizler'de:
        // Koç lagnadan 3., Ay'dan 8.; Akrep'e bakmaz (6. burç). Koşul yok.
        assert!(!forms(NB, &with(Some(0), &[(Moon, 7)])));
        // Mars Aslan'a alınır: Akrep Aslan'dan 4. burçtur (Mars'ın özel bakışı); Aslan Ay'dan
        // 10. evdir.
        let p = with(Some(0), &[(Moon, 7), (Mars, 4)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_DISPOSITOR_ASPECT]
        );
        // Mars Yengeç'te: lord Ay'dır. Ay kendisinden sayılmaz; Ay Boğa'da, Koç lagnadan 2.
        // evde → ilk koşul yok. Yengeç'te yücelen Jüpiter Kova'da: lagnadan 11., Ay'dan 10. →
        // ikinci koşul. Ay Boğa'dan Yengeç'e bakmaz.
        let p = with(Some(0), &[(Mars, 3)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_EXALTED_GRAHA_IN_KENDRA]
        );
        // Ay lagnadan kendradaysa ilk koşul sağlanır: Boğa lagna, Ay 1. evde.
        let p = with(Some(1), &[(Mars, 3)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_EXALTED_GRAHA_IN_KENDRA]
        );
        // Lagnasız konumda yalnızca Ay referansı ve bakış: aynı konumda ilk koşul düşer.
        let p = with(None, &[(Mars, 3)]);
        let m = &matches(NB, &p)[0];
        assert_eq!(m.conditions, [COND_EXALTED_GRAHA_IN_KENDRA]);
        assert!(m.houses.is_empty());
        // İki graha düşüşteyse iki ayrı eşleşme: Güneş Terazi'de (lord Venüs Koç'ta, bakıyor),
        // Satürn Koç'ta (lord Mars İkizler'de: lagnadan 3., Ay'dan 2.; Koç'ta yücelen Güneş
        // Terazi'de: lagnadan 7.).
        let p = with(Some(0), &[(Sun, 6), (Venus, 0), (Saturn, 0)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 2);
        assert_eq!(
            (m[0].grahas.as_slice(), m[1].grahas.as_slice()),
            (&[Sun][..], &[Saturn][..])
        );
        assert_eq!(m[1].conditions, [COND_EXALTED_GRAHA_IN_KENDRA]);
    }

    /// Elle yazılmış tablo: dolu burç sayısı → sayı yogası.
    const HAND_COUNT: [(u8, &str); 7] = [
        (7, "nabhasa_count.vallaki"),
        (6, "nabhasa_count.dama"),
        (5, "nabhasa_count.pasha"),
        (4, "nabhasa_count.kedara"),
        (3, "nabhasa_count.shula"),
        (2, "nabhasa_count.yuga"),
        (1, "nabhasa_count.gola"),
    ];

    #[test]
    fn exactly_one_count_yoga_forms_and_nodes_do_not_count() {
        // Yedi graha ilk `n` burca dağıtılır; düğümler başka burçlardadır ve sayılmaz.
        for (n, id) in HAND_COUNT {
            let mut signs = [0usize; 9];
            for (i, s) in signs.iter_mut().take(7).enumerate() {
                *s = i.min(usize::from(n) - 1);
            }
            signs[7] = 10;
            signs[8] = 4;
            let p = chart(Some(2), signs);
            assert_eq!(occupied_sign_count(&p), n);
            let formed: Vec<&str> = HAND_COUNT
                .into_iter()
                .map(|(_, other)| other)
                .filter(|other| forms(other, &p))
                .collect();
            assert_eq!(formed, [id]);
            let m = &matches(id, &p)[0];
            assert_eq!(m.grahas, YOGA_GRAHAS);
            assert_eq!(m.houses.len(), usize::from(n));
            assert!(matches(id, &p.clone().without_lagna())[0].houses.is_empty());
        }
    }

    #[test]
    fn ashraya_yogas_need_all_seven_in_one_kind_of_sign() {
        let ids = [
            "nabhasa_ashraya.rajju",
            "nabhasa_ashraya.musala",
            "nabhasa_ashraya.nala",
        ];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Yürük burçlar: Koç, Yengeç, Terazi, Oğlak. Düğümler başka türden burçlarda.
        let p = chart(None, [0, 3, 6, 9, 0, 3, 6, 1, 7]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.rajju"]);
        // Sabit burçlar: Boğa, Aslan, Akrep, Kova.
        let p = chart(None, [1, 4, 7, 10, 1, 4, 7, 0, 6]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.musala"]);
        // Çift doğalı burçlar: İkizler, Başak, Yay, Balık.
        let p = chart(None, [2, 5, 8, 11, 2, 5, 8, 0, 6]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.nala"]);
        // Tek graha başka türden burçtaysa hiçbiri oluşmaz.
        let p = chart(None, [2, 5, 8, 11, 2, 5, 7, 0, 6]);
        assert!(formed(&p).is_empty());
    }

    #[test]
    fn kuja_dosha_lists_the_references_that_hold() {
        // Mars İkizler'de. Koç lagnadan 3., Ay'dan (Boğa) 2., Venüs'ten (Yengeç) 12.
        let p = with(Some(0), &[]);
        let m = &matches("dosha.kuja", &p)[0];
        assert_eq!(m.grahas, [Mars]);
        assert_eq!(m.houses, [3]);
        assert_eq!(m.conditions, [COND_FROM_MOON, COND_FROM_VENUS]);
        // İkizler lagna: lagnadan 1. ev de.
        let p = with(Some(2), &[]);
        assert_eq!(
            matches("dosha.kuja", &p)[0].conditions,
            [COND_FROM_LAGNA, COND_FROM_MOON, COND_FROM_VENUS]
        );
        // Lagnasız konumda lagna referansı yoktur.
        let p = with(None, &[]);
        assert_eq!(
            matches("dosha.kuja", &p)[0].conditions,
            [COND_FROM_MOON, COND_FROM_VENUS]
        );
        // Oluşmaz: Mars Terazi'de, Venüs Aslan'da, Boğa lagna. Mars lagnadan 6., Ay'dan (Boğa)
        // 6., Venüs'ten 3. evdedir; hiçbiri 1, 2, 4, 7, 8, 12 değildir.
        let p = with(Some(1), &[(Mars, 6), (Venus, 4)]);
        assert!(!forms("dosha.kuja", &p));
    }
}
