//! Ay ve Güneş yogaları (spec §8 "Ay" ve "Güneş" satırları). Sayılan grahalar Mars, Merkür,
//! Jüpiter, Venüs ve Satürn'dür (`COUNTED_GRAHAS`); Güneş, Ay ve düğümler sayılmaz. Evler
//! Ay'ın ya da Güneş'in burcundan sayılan rasi evleridir.

use super::{
    in_graha_order, in_houses_from, occupied_houses, YogaMatch, YogaRelation, COND_FROM_LAGNA,
    COND_FROM_MOON, COUNTED_GRAHAS, KENDRA_HOUSES, UPACHAYA_HOUSES,
};
use crate::graha::Graha;
use crate::lordship::house_sign;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Bir ışığın (Ay ya da Güneş) iki yanındaki evlerin doluluğu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Flank {
    /// Sayılan graha yalnızca 2. evde.
    SecondOnly,
    /// Sayılan graha yalnızca 12. evde.
    TwelfthOnly,
    /// İki evde de sayılan graha var.
    Both,
}

/// `center`'ın burcundan sayılan `house`. evdeki sayılan grahalar.
fn counted_in(positions: &Positions, center: Graha, house: u8) -> Vec<Graha> {
    let reference = positions.sign_of(center);
    COUNTED_GRAHAS
        .into_iter()
        .filter(|&g| positions.house_from(g, reference) == house)
        .collect()
}

fn listed(positions: &Positions, grahas: Vec<Graha>) -> YogaMatch {
    let grahas = in_graha_order(grahas);
    YogaMatch {
        houses: occupied_houses(positions, &grahas),
        grahas,
        ..YogaMatch::default()
    }
}

/// Sunapha / Anapha / Durudhara (Ay) ve Vesi / Vasi / Ubhayachari (Güneş): üçü birbirini
/// dışlar. Eşleşme ışığı ve iki yandaki sayılan grahaları listeler.
pub(super) fn flank(positions: &Positions, center: Graha, want: Flank) -> Vec<YogaMatch> {
    let second = counted_in(positions, center, 2);
    let twelfth = counted_in(positions, center, 12);
    let found = match (second.is_empty(), twelfth.is_empty()) {
        (false, true) => Flank::SecondOnly,
        (true, false) => Flank::TwelfthOnly,
        (false, false) => Flank::Both,
        (true, true) => return Vec::new(),
    };
    if found != want {
        return Vec::new();
    }
    let mut grahas = vec![center];
    grahas.extend(second);
    grahas.extend(twelfth);
    vec![listed(positions, grahas)]
}

/// Kemadruma: Ay'dan 2. ve 12. evde sayılan graha yok, Ay'ın burcunda sayılan graha yok (Ay
/// yalnız) ve Ay'dan ve lagnadan kendralarda sayılan graha yok. Son koşul iptal kuralıdır;
/// lagna ister.
pub(super) fn kemadruma(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let moon = positions.sign_of(Graha::Moon);
    let near_moon = COUNTED_GRAHAS
        .into_iter()
        .any(|g| in_houses_from(positions, g, moon, &[12, 1, 2]));
    let in_a_kendra = COUNTED_GRAHAS.into_iter().any(|g| {
        in_houses_from(positions, g, moon, &KENDRA_HOUSES)
            || in_houses_from(positions, g, lagna, &KENDRA_HOUSES)
    });
    if near_moon || in_a_kendra {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon])]
}

/// Gajakesari: Jüpiter Ay'dan kendrada (Ay'la aynı burç dahil).
pub(super) fn gajakesari(positions: &Positions) -> Vec<YogaMatch> {
    let moon = positions.sign_of(Graha::Moon);
    if !in_houses_from(positions, Graha::Jupiter, moon, &KENDRA_HOUSES) {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon, Graha::Jupiter])]
}

fn together(positions: &Positions, a: Graha, b: Graha) -> Vec<YogaMatch> {
    if positions.sign_of(a) != positions.sign_of(b) {
        return Vec::new();
    }
    vec![YogaMatch {
        relations: vec![YogaRelation::Conjunction],
        ..listed(positions, vec![a, b])
    }]
}

/// Chandra-Mangala: Ay ile Mars birlikte.
pub(super) fn chandra_mangala(positions: &Positions) -> Vec<YogaMatch> {
    together(positions, Graha::Moon, Graha::Mars)
}

/// Budha-Aditya: Güneş ile Merkür birlikte.
pub(super) fn budha_aditya(positions: &Positions) -> Vec<YogaMatch> {
    together(positions, Graha::Sun, Graha::Mercury)
}

const NATURAL_BENEFIC_TRIO: [Graha; 3] = [Graha::Mercury, Graha::Jupiter, Graha::Venus];

/// Adhi: Merkür, Jüpiter ve Venüs'ün üçü de Ay'dan 6., 7. ya da 8. evde.
pub(super) fn adhi(positions: &Positions) -> Vec<YogaMatch> {
    let moon = positions.sign_of(Graha::Moon);
    if !NATURAL_BENEFIC_TRIO
        .into_iter()
        .all(|g| in_houses_from(positions, g, moon, &[6, 7, 8]))
    {
        return Vec::new();
    }
    let mut grahas = vec![Graha::Moon];
    grahas.extend(NATURAL_BENEFIC_TRIO);
    vec![listed(positions, grahas)]
}

/// Shakata: Ay Jüpiter'den 6., 8. ya da 12. evde. Ay lagnadan kendradaysa oluşmaz (iptal);
/// lagna ister.
pub(super) fn shakata(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let jupiter = positions.sign_of(Graha::Jupiter);
    if !in_houses_from(positions, Graha::Moon, jupiter, &[6, 8, 12])
        || in_houses_from(positions, Graha::Moon, lagna, &KENDRA_HOUSES)
    {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon, Graha::Jupiter])]
}

/// Lagna (varsa) ve Ay: iki referanslı kuralların referansları, koşul kimlikleriyle.
fn references(positions: &Positions) -> Vec<(&'static str, Sign)> {
    let mut out = Vec::new();
    if let Some(lagna) = positions.lagna_sign {
        out.push((COND_FROM_LAGNA, lagna));
    }
    out.push((COND_FROM_MOON, positions.sign_of(Graha::Moon)));
    out
}

/// Amala: lagnadan ya da Ay'dan 10. evde en az bir doğal iyicil (K6) var ve o evde kötücül
/// yok. Sağlayan referanslar koşul olarak, o evlerdeki grahalar eşleşmede listelenir.
pub(super) fn amala(positions: &Positions) -> Vec<YogaMatch> {
    let mut conditions = Vec::new();
    let mut grahas = Vec::new();
    for (condition, reference) in references(positions) {
        let occupants = positions.occupants(house_sign(reference, 10));
        let all_benefic = occupants
            .iter()
            .all(|&g| nature(g, positions) == Nature::Benefic);
        if !occupants.is_empty() && all_benefic {
            conditions.push(condition);
            grahas.extend(occupants);
        }
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        conditions,
        ..listed(positions, grahas)
    }]
}

/// Vasumati: Merkür, Jüpiter ve Venüs'ün üçü de lagnadan upachaya evlerinde ya da üçü de
/// Ay'dan upachaya evlerinde.
pub(super) fn vasumati(positions: &Positions) -> Vec<YogaMatch> {
    let conditions: Vec<&'static str> = references(positions)
        .into_iter()
        .filter(|&(_, reference)| {
            NATURAL_BENEFIC_TRIO
                .into_iter()
                .all(|g| in_houses_from(positions, g, reference, &UPACHAYA_HOUSES))
        })
        .map(|(condition, _)| condition)
        .collect();
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        conditions,
        ..listed(positions, NATURAL_BENEFIC_TRIO.to_vec())
    }]
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: Koç lagna; Ay Yengeç'te (4. ev), sayılan grahalar Ay'dan 3., 5., 6., 9. ve 11.
    /// evlerde, Güneş'ten 5., 7., 8., 11. ve 1. evlerde; hiçbiri lagnadan kendrada değil.
    /// Ay'dan: Mars Başak (3), Merkür Akrep (5), Jüpiter Yay (6), Venüs Balık (9), Satürn
    /// Boğa (11). Güneş Boğa'da.
    const BASE: [usize; 9] = [1, 3, 5, 7, 8, 11, 1, 2, 8];

    fn with(lagna: Option<usize>, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(lagna, signs)
    }

    #[test]
    fn moon_flank_yogas_exclude_each_other() {
        let ids = ["lunar.sunapha", "lunar.anapha", "lunar.durudhara"];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Taban: Ay'dan 2. (Aslan) ve 12. (İkizler) boş.
        assert!(formed(&with(Some(0), &[])).is_empty());
        // Mars Aslan'da (Ay'dan 2.): Sunapha.
        let p = with(Some(0), &[(Mars, 4)]);
        assert_eq!(formed(&p), ["lunar.sunapha"]);
        let m = &matches("lunar.sunapha", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars]);
        assert_eq!(m.houses, [4, 5]);
        assert!(m.relations.is_empty() && m.conditions.is_empty());
        // Venüs İkizler'de (Ay'dan 12.): Anapha.
        let p = with(Some(0), &[(Venus, 2)]);
        assert_eq!(formed(&p), ["lunar.anapha"]);
        assert_eq!(matches("lunar.anapha", &p)[0].grahas, [Moon, Venus]);
        // İkisi birden: yalnızca Durudhara.
        let p = with(Some(0), &[(Mars, 4), (Venus, 2), (Saturn, 4)]);
        assert_eq!(formed(&p), ["lunar.durudhara"]);
        let m = &matches("lunar.durudhara", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars, Venus, Saturn]);
        assert_eq!(m.houses, [3, 4, 5]);
        // Güneş ve düğümler sayılmaz: Güneş Aslan'da, Rahu İkizler'de.
        assert!(formed(&with(Some(0), &[(Sun, 4), (Rahu, 2)])).is_empty());
        // Lagnasız konumda evler boştur; yoga yine oluşur.
        let p = with(None, &[(Mars, 4)]);
        assert_eq!(matches("lunar.sunapha", &p)[0].houses, [] as [u8; 0]);
    }

    #[test]
    fn sun_flank_yogas_exclude_each_other() {
        let ids = ["solar.vesi", "solar.vasi", "solar.ubhayachari"];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Taban: Satürn Güneş'le aynı burçta (Boğa); Güneş'ten 2. (İkizler) ve 12. (Koç) boş.
        assert!(formed(&with(Some(0), &[])).is_empty());
        let p = with(Some(0), &[(Jupiter, 2)]);
        assert_eq!(formed(&p), ["solar.vesi"]);
        assert_eq!(matches("solar.vesi", &p)[0].grahas, [Sun, Jupiter]);
        let p = with(Some(0), &[(Mercury, 0)]);
        assert_eq!(formed(&p), ["solar.vasi"]);
        let p = with(Some(0), &[(Jupiter, 2), (Mercury, 0)]);
        assert_eq!(formed(&p), ["solar.ubhayachari"]);
        assert_eq!(
            matches("solar.ubhayachari", &p)[0].grahas,
            [Sun, Mercury, Jupiter]
        );
        // Ay sayılmaz: Ay İkizler'de.
        assert!(formed(&with(Some(0), &[(Moon, 2)])).is_empty());
    }

    #[test]
    fn kemadruma_needs_an_isolated_moon_and_empty_kendras() {
        // Taban: Ay'dan 2. ve 12. boş, Ay yalnız; Ay'dan kendralar (Yengeç, Terazi, Oğlak,
        // Koç) ve lagnadan kendralar (Koç, Yengeç, Terazi, Oğlak) sayılan graha taşımıyor.
        let p = with(Some(0), &[]);
        let m = matches("lunar.kemadruma", &p);
        assert_eq!(m.len(), 1);
        assert_eq!(
            (m[0].grahas.as_slice(), m[0].houses.as_slice()),
            (&[Moon][..], &[4][..])
        );
        // Güneş ve düğümler yalnızlığı bozmaz.
        assert!(forms(
            "lunar.kemadruma",
            &with(Some(0), &[(Sun, 3), (Rahu, 4), (Ketu, 2)])
        ));
        // Oluşmaz: Ay'dan 2. evde graha (Sunapha oluşur).
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Mars, 4)])));
        // Oluşmaz: Ay yalnız değil.
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Venus, 3)])));
        // İptal: Ay'dan kendrada graha (Jüpiter Terazi'de, Ay'dan 4.).
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Jupiter, 6)])));
        // İptal: lagnadan kendrada graha. İkizler lagna: Başak'taki Mars lagnadan 4. evdedir
        // ama Ay'dan 3. evdedir.
        assert!(!forms("lunar.kemadruma", &with(Some(2), &[])));
        // Lagna yoksa değerlendirilmez.
        assert!(!forms("lunar.kemadruma", &with(None, &[])));
    }

    #[test]
    fn gajakesari_counts_the_four_kendras_from_the_moon() {
        for (jupiter, want) in [
            (3, true),   // aynı burç
            (6, true),   // 4.
            (9, true),   // 7.
            (0, true),   // 10.
            (4, false),  // 2.
            (8, false),  // 6.
            (11, false), // 9.
        ] {
            let p = with(Some(0), &[(Jupiter, jupiter)]);
            assert_eq!(forms("lunar.gajakesari", &p), want, "Jupiter in {jupiter}");
        }
        let p = with(Some(0), &[(Jupiter, 9)]);
        let m = &matches("lunar.gajakesari", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.houses.as_slice()),
            (&[Moon, Jupiter][..], &[4, 10][..])
        );
    }

    #[test]
    fn conjunction_yogas_carry_the_relation() {
        assert!(!forms("lunar.chandra_mangala", &with(Some(0), &[])));
        let p = with(Some(0), &[(Mars, 3)]);
        let m = &matches("lunar.chandra_mangala", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars]);
        assert_eq!(m.houses, [4]);
        assert_eq!(m.relations, [YogaRelation::Conjunction]);
        assert!(!forms("solar.budha_aditya", &with(Some(0), &[])));
        let p = with(Some(0), &[(Mercury, 1)]);
        let m = &matches("solar.budha_aditya", &p)[0];
        assert_eq!(m.grahas, [Sun, Mercury]);
        assert_eq!(m.relations, [YogaRelation::Conjunction]);
    }

    #[test]
    fn adhi_needs_all_three_in_the_sixth_seventh_or_eighth_from_the_moon() {
        // Ay Yengeç'te: 6. Yay, 7. Oğlak, 8. Kova.
        let p = with(Some(0), &[(Mercury, 8), (Jupiter, 9), (Venus, 10)]);
        let m = &matches("lunar.adhi", &p)[0];
        assert_eq!(m.grahas, [Moon, Mercury, Jupiter, Venus]);
        assert_eq!(m.houses, [4, 9, 10, 11]);
        // Üçü aynı evde de olabilir.
        assert!(forms(
            "lunar.adhi",
            &with(Some(0), &[(Mercury, 9), (Jupiter, 9), (Venus, 9)])
        ));
        // Biri dışarıda (Venüs Balık'ta, 9. ev): oluşmaz.
        assert!(!forms(
            "lunar.adhi",
            &with(Some(0), &[(Mercury, 8), (Jupiter, 9)])
        ));
    }

    #[test]
    fn shakata_is_cancelled_by_a_moon_in_a_kendra_from_the_lagna() {
        // Taban: Jüpiter Yay'da, Ay Yengeç'te: Ay Jüpiter'den 8. evde. Boğa lagna: Ay 3. evde.
        let p = with(Some(1), &[]);
        let m = &matches("lunar.shakata", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.houses.as_slice()),
            (&[Moon, Jupiter][..], &[3, 8][..])
        );
        // 6. ve 12. de sayılır: Jüpiter Kova'da (Ay ondan 6.), Jüpiter Aslan'da (Ay ondan 12.).
        assert!(forms("lunar.shakata", &with(Some(1), &[(Jupiter, 10)])));
        assert!(forms("lunar.shakata", &with(Some(1), &[(Jupiter, 4)])));
        // Oluşmaz: Ay Jüpiter'den 7. evde.
        assert!(!forms("lunar.shakata", &with(Some(1), &[(Jupiter, 9)])));
        // İptal: Koç lagna, Ay 4. evde (kendra).
        assert!(!forms("lunar.shakata", &with(Some(0), &[])));
        assert!(!forms("lunar.shakata", &with(None, &[])));
    }

    #[test]
    fn amala_needs_a_benefic_and_no_malefic_in_the_tenth() {
        // Taban, Koç lagna: lagnadan 10. Oğlak (boş); Ay'dan 10. Koç (boş).
        assert!(!forms("lunar.amala", &with(Some(0), &[])));
        // Jüpiter Oğlak'ta: lagnadan.
        let p = with(Some(0), &[(Jupiter, 9)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(m.grahas, [Jupiter]);
        assert_eq!(m.houses, [10]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA]);
        // Venüs Koç'ta: Ay'dan. Lagnasız konumda da bulunur.
        let p = with(None, &[(Venus, 0)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.conditions.as_slice()),
            (&[Venus][..], &[COND_FROM_MOON][..])
        );
        // İki referans birden; grahalar birleşir.
        let p = with(Some(0), &[(Jupiter, 9), (Venus, 0)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(m.grahas, [Jupiter, Venus]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA, COND_FROM_MOON]);
        // Aynı evde kötücül varsa oluşmaz: Jüpiter ve Ketu Oğlak'ta.
        assert!(!forms(
            "lunar.amala",
            &with(Some(0), &[(Jupiter, 9), (Ketu, 9)])
        ));
        // Kötücülle tek başına duran Merkür kötücüldür (K6): Merkür ve Güneş Oğlak'ta.
        assert!(!forms(
            "lunar.amala",
            &with(Some(0), &[(Mercury, 9), (Sun, 9)])
        ));
        // Tek başına Merkür iyicildir.
        assert!(forms("lunar.amala", &with(Some(0), &[(Mercury, 9)])));
        // Büyüyen Ay iyicildir: Terazi lagna, Ay Yengeç'te (lagnadan 10.), Güneş Boğa'da.
        let p = with(Some(6), &[]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.conditions.as_slice()),
            (&[Moon][..], &[COND_FROM_LAGNA][..])
        );
        // Küçülen Ay kötücüldür: Güneş Akrep'te (uzanım 240°); Merkür de Akrep'tedir.
        assert!(!forms("lunar.amala", &with(Some(6), &[(Sun, 7)])));
    }

    #[test]
    fn vasumati_needs_all_three_in_upachayas_from_one_reference() {
        assert!(!forms("lunar.vasumati", &with(Some(0), &[])));
        // Koç lagna: 3. İkizler, 6. Başak, 10. Oğlak, 11. Kova.
        let p = with(Some(0), &[(Mercury, 2), (Jupiter, 5), (Venus, 10)]);
        let m = &matches("lunar.vasumati", &p)[0];
        assert_eq!(m.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(m.houses, [3, 6, 11]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA]);
        // Ay Yengeç'te: 3. Başak, 6. Yay, 10. Koç, 11. Boğa.
        let p = with(Some(0), &[(Mercury, 5), (Jupiter, 8), (Venus, 1)]);
        assert_eq!(
            matches("lunar.vasumati", &p)[0].conditions,
            [COND_FROM_MOON]
        );
        // Karışık referans sayılmaz: Merkür ve Jüpiter lagnadan, Venüs yalnızca Ay'dan.
        assert!(!forms(
            "lunar.vasumati",
            &with(Some(0), &[(Mercury, 2), (Jupiter, 10), (Venus, 1)])
        ));
        // Lagnasız konumda yalnızca Ay referansı.
        let p = with(None, &[(Mercury, 5), (Jupiter, 8), (Venus, 1)]);
        assert_eq!(
            matches("lunar.vasumati", &p)[0].conditions,
            [COND_FROM_MOON]
        );
        assert!(!forms(
            "lunar.vasumati",
            &with(None, &[(Mercury, 2), (Jupiter, 5), (Venus, 10)])
        ));
    }
}
