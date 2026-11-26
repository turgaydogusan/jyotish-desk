//! Ev lordlarına dayalı yogalar (spec §8): raja (kendra–trikona, dharma-karmadhipati,
//! yogakaraka), dhana, değişim sınıfları ve viparita raja. Hepsi lagna ister: lordlar lagna
//! burcundan sayılan rasi evlerinin lordlarıdır (K4; `lordship::house_lord`).

use super::{
    is_exchange, yoga_relations, YogaMatch, YogaRelation, DUSTHANA_HOUSES, KENDRA_HOUSES,
    TRIKONA_HOUSES, YOGA_GRAHAS,
};
use crate::graha::Graha;
use crate::lordship::{house_lord, houses_owned};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Dhana yogasında sayılan evler.
pub const DHANA_HOUSES: [u8; 5] = [1, 2, 5, 9, 11];

/// Değişim sınıfı; kural kimlikleri `exchange.maha`, `exchange.third_lord`, `exchange.dainya`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeKind {
    Maha,
    ThirdLord,
    Dainya,
}

/// Değişen iki evin sınıfı: biri 6., 8. ya da 12. evse Dainya; değilse biri 3. evse "3. ev
/// lordu değişimi"; değilse Maha. `a` ve `b` farklı evlerdir.
pub fn exchange_kind(a: u8, b: u8) -> ExchangeKind {
    if DUSTHANA_HOUSES.contains(&a) || DUSTHANA_HOUSES.contains(&b) {
        ExchangeKind::Dainya
    } else if a == 3 || b == 3 {
        ExchangeKind::ThirdLord
    } else {
        ExchangeKind::Maha
    }
}

/// `set` içinden grahanın lordu olduğu evler, artan sırada.
fn owned_in(lagna: Sign, graha: Graha, set: &[u8]) -> Vec<u8> {
    houses_owned(lagna, graha)
        .into_iter()
        .filter(|h| set.contains(h))
        .collect()
}

fn merged(mut a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    a.extend(b);
    a.sort_unstable();
    a.dedup();
    a
}

/// Yedi grahanın sırasız çiftleri, `Graha::ALL` sırasında.
fn pairs() -> impl Iterator<Item = (Graha, Graha)> {
    YOGA_GRAHAS
        .into_iter()
        .enumerate()
        .flat_map(|(i, a)| YOGA_GRAHAS[i + 1..].iter().map(move |&b| (a, b)))
}

/// Kendra–trikona raja yogası: bir kendra lordu ile bir trikona lordu farklı grahalardır ve
/// ilişkilidir. Çift başına tek eşleşme; evler iki grahanın lordu olduğu kendra ve trikonalardır.
pub(super) fn kendra_trikona(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    pairs()
        .filter_map(|(a, b)| {
            let (a_kendra, a_trikona) = (
                owned_in(lagna, a, &KENDRA_HOUSES),
                owned_in(lagna, a, &TRIKONA_HOUSES),
            );
            let (b_kendra, b_trikona) = (
                owned_in(lagna, b, &KENDRA_HOUSES),
                owned_in(lagna, b, &TRIKONA_HOUSES),
            );
            let pairs_up = (!a_kendra.is_empty() && !b_trikona.is_empty())
                || (!a_trikona.is_empty() && !b_kendra.is_empty());
            let relations = yoga_relations(a, b, positions);
            (pairs_up && !relations.is_empty()).then(|| YogaMatch {
                grahas: vec![a, b],
                houses: merged(merged(a_kendra, a_trikona), merged(b_kendra, b_trikona)),
                relations,
                conditions: Vec::new(),
            })
        })
        .collect()
}

/// Dharma-Karmadhipati: 9. ve 10. lord farklı grahalardır ve ilişkilidir.
pub(super) fn dharma_karmadhipati(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let (ninth, tenth) = (house_lord(lagna, 9), house_lord(lagna, 10));
    let relations = yoga_relations(ninth, tenth, positions);
    if relations.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: super::in_graha_order(vec![ninth, tenth]),
        houses: vec![9, 10],
        relations,
        conditions: Vec::new(),
    }]
}

/// Yogakaraka: tek graha, lagna dışında hem bir kendranın (4, 7, 10) hem bir trikonanın (5, 9)
/// lordu. Yalnızca lagna burcuna bağlıdır.
pub(super) fn yogakaraka(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    YOGA_GRAHAS
        .into_iter()
        .filter_map(|g| {
            let kendra = owned_in(lagna, g, &[4, 7, 10]);
            let trikona = owned_in(lagna, g, &[5, 9]);
            (!kendra.is_empty() && !trikona.is_empty()).then(|| YogaMatch {
                grahas: vec![g],
                houses: merged(kendra, trikona),
                ..YogaMatch::default()
            })
        })
        .collect()
}

/// Dhana: {1, 2, 5, 9, 11} lordlarından, en az biri 2. ya da 11. lord olan iki farklı graha
/// ilişkili. Çift başına tek eşleşme; evler iki grahanın bu kümeden lordu olduğu evlerdir.
pub(super) fn dhana(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    pairs()
        .filter_map(|(a, b)| {
            let (wa, wb) = (
                owned_in(lagna, a, &DHANA_HOUSES),
                owned_in(lagna, b, &DHANA_HOUSES),
            );
            if wa.is_empty() || wb.is_empty() {
                return None;
            }
            let houses = merged(wa, wb);
            let relations = yoga_relations(a, b, positions);
            (houses.iter().any(|h| [2, 11].contains(h)) && !relations.is_empty()).then(|| {
                YogaMatch {
                    grahas: vec![a, b],
                    houses,
                    relations,
                    conditions: Vec::new(),
                }
            })
        })
        .collect()
}

/// Değişim yogaları: birbirinin burcundaki iki graha; evler değişen iki evdir (grahaların
/// bulunduğu evler). Eşleşme yalnızca `kind` sınıfındaki çiftleri verir.
pub(super) fn exchange(positions: &Positions, kind: ExchangeKind) -> Vec<YogaMatch> {
    if positions.lagna_sign.is_none() {
        return Vec::new();
    }
    pairs()
        .filter(|&(a, b)| is_exchange(a, b, positions))
        .filter_map(|(a, b)| {
            let (ha, hb) = (positions.house_of(a)?, positions.house_of(b)?);
            (exchange_kind(ha, hb) == kind).then(|| YogaMatch {
                grahas: vec![a, b],
                houses: merged(vec![ha], vec![hb]),
                relations: yoga_relations(a, b, positions)
                    .into_iter()
                    .filter(|r| *r == YogaRelation::Exchange)
                    .collect(),
                conditions: Vec::new(),
            })
        })
        .collect()
}

/// Viparita raja: `house` (6, 8 ya da 12) lordu 6., 8. ya da 12. evde. Evler lordu olduğu ev ve
/// bulunduğu evdir.
pub(super) fn viparita(positions: &Positions, house: u8) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let lord = house_lord(lagna, house);
    match positions.house_of(lord) {
        Some(at) if DUSTHANA_HOUSES.contains(&at) => vec![YogaMatch {
            grahas: vec![lord],
            houses: merged(vec![house], vec![at]),
            ..YogaMatch::default()
        }],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::super::YOGA_CATALOG;
    use super::*;
    use Graha::*;
    use YogaRelation::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): lagna burcu (Koç 0 … Balık 11) →
    /// yogakaraka ve lordu olduğu kendra ile trikona. Altı lagnada yogakaraka yoktur.
    const HAND_YOGAKARAKA: [Option<(Graha, [u8; 2])>; 12] = [
        None,                    // Koç
        Some((Saturn, [9, 10])), // Boğa: Oğlak 9., Kova 10.
        None,                    // İkizler
        Some((Mars, [5, 10])),   // Yengeç: Akrep 5., Koç 10.
        Some((Mars, [4, 9])),    // Aslan: Akrep 4., Koç 9.
        None,                    // Başak
        Some((Saturn, [4, 5])),  // Terazi: Oğlak 4., Kova 5.
        None,                    // Akrep
        None,                    // Yay
        Some((Venus, [5, 10])),  // Oğlak: Boğa 5., Terazi 10.
        Some((Venus, [4, 9])),   // Kova: Boğa 4., Terazi 9.
        None,                    // Balık
    ];

    #[test]
    fn yogakaraka_matches_the_hand_table_whatever_the_placements() {
        for (lagna, want) in HAND_YOGAKARAKA.into_iter().enumerate() {
            for shift in [0usize, 5] {
                let p = chart(Some(lagna), [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|s| s + shift));
                let got = matches("raja.yogakaraka", &p);
                match want {
                    Some((graha, houses)) => {
                        assert_eq!(got.len(), 1, "lagna {lagna}");
                        assert_eq!(got[0].grahas, [graha], "lagna {lagna}");
                        assert_eq!(got[0].houses, houses, "lagna {lagna}");
                        assert!(got[0].relations.is_empty() && got[0].conditions.is_empty());
                    }
                    None => assert!(got.is_empty(), "lagna {lagna}"),
                }
            }
        }
    }

    /// Elle yazılmış tablo: değişen iki evin sınıfı. Satır küçük ev, sütun büyük ev; `M` Maha,
    /// `3` 3. ev lordu değişimi, `D` Dainya. Köşegen ve altı kullanılmaz (`.`).
    const HAND_EXCHANGE: [&str; 12] = [
        ".M3MMDMDMMMD", // 1
        "..3MMDMDMMMD", // 2
        "...33D3D333D", // 3
        "....MDMDMMMD", // 4
        ".....DMDMMMD", // 5
        "......DDDDDD", // 6
        ".......DMMMD", // 7
        "........DDDD", // 8
        ".........MMD", // 9
        "..........MD", // 10
        "...........D", // 11
        "............", // 12
    ];

    #[test]
    fn exchange_kind_matches_the_hand_table_and_is_symmetric() {
        for a in 1..=12u8 {
            for b in (a + 1)..=12 {
                let cell = HAND_EXCHANGE[usize::from(a) - 1].as_bytes()[usize::from(b) - 1];
                let want = match cell {
                    b'M' => ExchangeKind::Maha,
                    b'3' => ExchangeKind::ThirdLord,
                    b'D' => ExchangeKind::Dainya,
                    other => panic!("bad cell {other} at {a},{b}"),
                };
                assert_eq!(exchange_kind(a, b), want, "{a} and {b}");
                assert_eq!(exchange_kind(b, a), want, "{b} and {a}");
            }
        }
    }

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban, Koç lagna: Güneş ve Merkür Koç'ta (1. ev), Ay Boğa'da (2), Satürn Yengeç'te (4),
    /// Jüpiter Aslan'da (5), Mars Yay'da (9), Venüs Balık'ta (12). Altı dolu burcun hiçbiri
    /// diğerinin karşısında değildir; Satürn Mars'tan 4. burçta değildir; değişim yoktur. Tek
    /// ilişkili çift Güneş–Merkür'dür (birlikte) ve Merkür (3. ve 6. lord) kendra, trikona ya
    /// da dhana evlerinin lordu değildir.
    /// Koç lagnanın lordları: 1 Mars, 2 Venüs, 3 Merkür, 4 Ay, 5 Güneş, 6 Merkür, 7 Venüs,
    /// 8 Mars, 9 Jüpiter, 10 Satürn, 11 Satürn, 12 Jüpiter.
    const BASE: [usize; 9] = [0, 1, 8, 0, 4, 11, 3, 5, 11];

    fn with(lagna: usize, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(Some(lagna), signs)
    }

    fn rows(id: &str, p: &Positions) -> Vec<(Vec<Graha>, Vec<u8>, Vec<YogaRelation>)> {
        matches(id, p)
            .into_iter()
            .map(|m| {
                assert!(m.conditions.is_empty(), "{id}");
                (m.grahas, m.houses, m.relations)
            })
            .collect()
    }

    #[test]
    fn kendra_trikona_pairs_a_kendra_lord_with_a_trikona_lord() {
        assert!(rows("raja.kendra_trikona", &with(0, &[])).is_empty());
        // Ay (4. lord) Aslan'a, Jüpiter'in (9. lord) yanına: birlikte.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Moon, 4)])),
            [(vec![Moon, Jupiter], vec![4, 9], vec![Conjunction])]
        );
        // Lagna lordu hem kendra hem trikona lordudur: Mars (1) Yay'da, Venüs (7) İkizler'e
        // alınır: karşılıklı 7. bakış. Venüs yalnızca kendra lordudur; Mars trikona lordu sayılır.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Venus, 2)])),
            [(vec![Mars, Venus], vec![1, 7], vec![MutualAspect])]
        );
        // İki kendra lordu (Ay 4, Venüs 7) birlikte olsa da yoga oluşmaz: Ay Balık'a alınır.
        assert!(rows("raja.kendra_trikona", &with(0, &[(Moon, 11)])).is_empty());
        // Değişim: Güneş (5) Yengeç'e, Ay (4) Aslan'a. Güneş orada Satürn'le (10), Ay Jüpiter'le
        // (9) birliktedir: üç çift.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Sun, 3), (Moon, 4)])),
            [
                (vec![Sun, Moon], vec![4, 5], vec![Exchange]),
                (vec![Sun, Saturn], vec![5, 10], vec![Conjunction]),
                (vec![Moon, Jupiter], vec![4, 9], vec![Conjunction]),
            ]
        );
        // Birden çok ilişki tek eşleşmede: Mars (1) Oğlak'a, Satürn (10) Koç'a: değişim ve
        // karşılıklı bakış (Koç Oğlak'tan 4., Oğlak Koç'tan 10. burç). Satürn Koç'ta Güneş'le
        // (5) birliktedir: o çift de oluşur.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Mars, 9), (Saturn, 0)])),
            [
                (vec![Sun, Saturn], vec![5, 10], vec![Conjunction]),
                (
                    vec![Mars, Saturn],
                    vec![1, 10],
                    vec![MutualAspect, Exchange]
                ),
            ]
        );
        assert!(!forms(
            "raja.kendra_trikona",
            &with(0, &[(Moon, 4)]).without_lagna()
        ));
    }

    #[test]
    fn dharma_karmadhipati_is_the_ninth_and_tenth_lord_case() {
        // Koç lagna: 9. lord Jüpiter (Aslan), 10. lord Satürn (Yengeç): ilişkili değiller.
        assert!(!forms("raja.dharma_karmadhipati", &with(0, &[])));
        // Satürn Kova'ya: Jüpiter'le karşılıklı 7. bakış.
        let p = with(0, &[(Saturn, 10)]);
        assert_eq!(
            rows("raja.dharma_karmadhipati", &p),
            [(vec![Jupiter, Saturn], vec![9, 10], vec![MutualAspect])]
        );
        // Aynı çift kendra–trikona kaydı olarak da döner.
        assert_eq!(
            rows("raja.kendra_trikona", &p),
            [(vec![Jupiter, Saturn], vec![9, 10], vec![MutualAspect])]
        );
        // Boğa lagnada 9. ve 10. lord aynı grahadır (Satürn): yoga oluşmaz, yogakaraka oluşur.
        for shift in 0..12usize {
            let p = chart(Some(1), BASE.map(|s| s + shift));
            assert!(!forms("raja.dharma_karmadhipati", &p), "shift {shift}");
            assert!(forms("raja.yogakaraka", &p));
        }
        // Grahalar `Graha::ALL` sırasındadır: Kova lagnada 9. lord Venüs, 10. lord Mars; ikisi
        // İkizler'de.
        let p = with(10, &[(Mars, 2), (Venus, 2)]);
        assert_eq!(
            rows("raja.dharma_karmadhipati", &p),
            [(vec![Mars, Venus], vec![9, 10], vec![Conjunction])]
        );
    }

    #[test]
    fn dhana_needs_the_second_or_eleventh_lord_in_the_pair() {
        // Koç lagna: 1 Mars, 2 Venüs, 5 Güneş, 9 Jüpiter, 11 Satürn.
        assert!(rows("dhana.dhana", &with(0, &[])).is_empty());
        // Güneş (5) Aslan'a, Jüpiter'in (9) yanına: ikisi de kümede ama 2. ya da 11. lord yok.
        assert!(rows("dhana.dhana", &with(0, &[(Sun, 4)])).is_empty());
        // Venüs (2) Aslan'a, Jüpiter'in (9) yanına.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Venus, 4)])),
            [(vec![Jupiter, Venus], vec![2, 9], vec![Conjunction])]
        );
        // Satürn (11) Koç'a, Güneş'in (5) yanına; Merkür de oradadır ama kümede değildir.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Saturn, 0)])),
            [(vec![Sun, Saturn], vec![5, 11], vec![Conjunction])]
        );
        // 2. ve 11. lord birbiriyle: Venüs (2) Yengeç'e, Satürn'ün (11) yanına.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Venus, 3)])),
            [(vec![Venus, Saturn], vec![2, 11], vec![Conjunction])]
        );
        // Bir graha kümeden iki evin lorduysa ikisi de yazılır: Boğa lagnada Merkür 2 ve 5'in,
        // Jüpiter 11'in lordudur. Merkür Kova'ya alınır: Aslan'daki Jüpiter'le karşılıklı bakış.
        assert_eq!(
            rows("dhana.dhana", &with(1, &[(Mercury, 10)])),
            [(vec![Mercury, Jupiter], vec![2, 5, 11], vec![MutualAspect])]
        );
    }

    #[test]
    fn exchanges_are_classified_by_the_two_houses() {
        let ids = ["exchange.maha", "exchange.third_lord", "exchange.dainya"];
        let all = |p: &Positions| -> Vec<(&str, Vec<Graha>, Vec<u8>)> {
            ids.into_iter()
                .flat_map(|id| {
                    rows(id, p).into_iter().map(move |(g, h, r)| {
                        assert_eq!(r, [Exchange], "{id}");
                        (id, g, h)
                    })
                })
                .collect()
        };
        assert!(all(&with(0, &[])).is_empty());
        // Koç lagna. Maha: Güneş Yengeç'te (4. ev), Ay Aslan'da (5. ev).
        assert_eq!(
            all(&with(0, &[(Sun, 3), (Moon, 4)])),
            [("exchange.maha", vec![Sun, Moon], vec![4, 5])]
        );
        // 3. ev lordu değişimi: Merkür Boğa'da (2. ev), Venüs İkizler'de (3. ev).
        assert_eq!(
            all(&with(0, &[(Mercury, 1), (Venus, 2)])),
            [("exchange.third_lord", vec![Mercury, Venus], vec![2, 3])]
        );
        // Dainya: Mars Başak'ta (6. ev), Merkür Koç'ta (1. ev). Mars 1. evin de lordudur; sınıfı
        // değişen evler (1 ve 6) belirler.
        assert_eq!(
            all(&with(0, &[(Mars, 5)])),
            [("exchange.dainya", vec![Mars, Mercury], vec![1, 6])]
        );
        // Aynı iki graha, başka evler: Mars İkizler'de (3), Merkür Akrep'te (8). 6, 8 ve 12
        // önce gelir: Dainya.
        assert_eq!(
            all(&with(0, &[(Mars, 2), (Mercury, 7)])),
            [("exchange.dainya", vec![Mars, Mercury], vec![3, 8])]
        );
        // İki değişim birden: Güneş–Ay (4 ve 5) ve Mars Terazi'de, Venüs Koç'ta (7 ve 1).
        // İkinci çift karşılıklı bakışta da olsa değişim kaydı yalnızca değişimi taşır.
        assert_eq!(
            all(&with(0, &[(Sun, 3), (Moon, 4), (Mars, 6), (Venus, 0)])),
            [
                ("exchange.maha", vec![Sun, Moon], vec![4, 5]),
                ("exchange.maha", vec![Mars, Venus], vec![1, 7]),
            ]
        );
        assert!(!forms(
            "exchange.maha",
            &with(0, &[(Sun, 3), (Moon, 4)]).without_lagna()
        ));
    }

    #[test]
    fn viparita_names_follow_the_lorded_house() {
        // Koç lagna: 6 Merkür, 8 Mars, 12 Jüpiter. Taban: Merkür 1., Mars 9., Jüpiter 5. evde.
        for id in ["viparita.harsha", "viparita.sarala", "viparita.vimala"] {
            assert!(!forms(id, &with(0, &[])), "{id}");
        }
        // Harsha: 6. lord Merkür 8. evde (Akrep).
        let p = with(0, &[(Mercury, 7)]);
        assert_eq!(
            rows("viparita.harsha", &p),
            [(vec![Mercury], vec![6, 8], vec![])]
        );
        // Kendi evinde: 6. lord 6. evde → tek ev.
        let p = with(0, &[(Mercury, 5)]);
        assert_eq!(
            rows("viparita.harsha", &p),
            [(vec![Mercury], vec![6], vec![])]
        );
        // Sarala: 8. lord Mars 12. evde (Balık).
        let p = with(0, &[(Mars, 11)]);
        assert_eq!(
            rows("viparita.sarala", &p),
            [(vec![Mars], vec![8, 12], vec![])]
        );
        assert!(!forms("viparita.harsha", &p) && !forms("viparita.vimala", &p));
        // Vimala: 12. lord Jüpiter 6. evde (Başak).
        let p = with(0, &[(Jupiter, 5)]);
        assert_eq!(
            rows("viparita.vimala", &p),
            [(vec![Jupiter], vec![6, 12], vec![])]
        );
        assert!(!forms("viparita.vimala", &p.without_lagna()));
    }

    /// Elle yazılmış liste: lagna isteyen kurallar (doğum saati bilinmiyorsa atlananlar).
    const HAND_NEEDS_LAGNA: [&str; 17] = [
        "mahapurusha.ruchaka",
        "mahapurusha.bhadra",
        "mahapurusha.hamsa",
        "mahapurusha.malavya",
        "mahapurusha.shasha",
        "lunar.kemadruma",
        "lunar.shakata",
        "raja.kendra_trikona",
        "raja.dharma_karmadhipati",
        "raja.yogakaraka",
        "dhana.dhana",
        "exchange.maha",
        "exchange.third_lord",
        "exchange.dainya",
        "viparita.harsha",
        "viparita.sarala",
        "viparita.vimala",
    ];

    #[test]
    fn the_catalog_has_forty_one_rules_and_seventeen_need_the_lagna() {
        assert_eq!(YOGA_CATALOG.len(), 41);
        let needing: Vec<&str> = YOGA_CATALOG
            .iter()
            .filter(|r| r.needs_lagna)
            .map(|r| r.id)
            .collect();
        assert_eq!(needing, HAND_NEEDS_LAGNA);
    }
}
