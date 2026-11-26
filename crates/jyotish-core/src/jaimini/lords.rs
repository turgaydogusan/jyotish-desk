//! Jaimini eş lordları (ileri modüller spec'i §5): Akrep'in lordları Mars ile Ketu, Kova'nın
//! lordları Satürn ile Rahu'dur ve güçlü olan lord sayılır. Arudha pada ve chara dasha (Alt
//! proje 3.3) aynı seçimi kullanır.
//!
//! Güçlü lord sırayla altı ölçütle seçilir; ilk ayrım belirleyicidir:
//! 1. yalnızca biri burcun kendisindeyse, burcun dışındaki;
//! 2. bulunduğu burçta daha çok grahayla birlikte olan (diğer eş lord sayılmaz);
//! 3. yücelme burcunda olan; değilse düşüş burcunda olmayan (düğümlerin yücelme ve düşüş burcu
//!    yoktur: bu ölçüt yalnızca Mars'ın ya da Satürn'ün durumuna bakar);
//! 4. bulunduğu burcun türü: çift doğalı > sabit > yürük;
//! 5. söz konusu burçtan, o burcun sayım yönünde daha uzakta olan;
//! 6. Mars (Akrep) ya da Satürn (Kova).

use crate::dignity::{debilitation_point, exaltation_point, sign_lord};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::rashi::{directed_count, sign_kind, SignKind};

/// Burcun eş lordları: (graha lord, düğüm lord). Akrep ve Kova dışında `None`.
pub fn co_lords(sign: Sign) -> Option<(Graha, Graha)> {
    match sign {
        Sign::Scorpio => Some((Graha::Mars, Graha::Ketu)),
        Sign::Aquarius => Some((Graha::Saturn, Graha::Rahu)),
        _ => None,
    }
}

/// Eş lord seçimi: seçilen lord ve belirleyici ölçütün numarası (1..=6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoLordChoice {
    pub lord: Graha,
    pub step: u8,
}

/// Burç türünün 4. ölçütteki sırası.
fn kind_rank(kind: SignKind) -> u8 {
    match kind {
        SignKind::Dual => 3,
        SignKind::Fixed => 2,
        SignKind::Movable => 1,
    }
}

/// `sign` burcunun eş lordlarından güçlü olanı ve hangi ölçütün belirlediği. Akrep ve Kova
/// dışında `None`.
pub fn co_lord_choice(sign: Sign, positions: &Positions) -> Option<CoLordChoice> {
    let (planet, node) = co_lords(sign)?;
    let planet_sign = positions.sign_of(planet);
    let node_sign = positions.sign_of(node);
    let pick = |lord, step| Some(CoLordChoice { lord, step });
    // Büyük olan tarafın lordu; eşitse `None`.
    let stronger = |p: u8, n: u8| match p.cmp(&n) {
        std::cmp::Ordering::Greater => Some(planet),
        std::cmp::Ordering::Less => Some(node),
        std::cmp::Ordering::Equal => None,
    };

    // 1. Yalnızca biri burcun kendisindeyse, burcun dışındaki.
    match (planet_sign == sign, node_sign == sign) {
        (true, false) => return pick(node, 1),
        (false, true) => return pick(planet, 1),
        _ => {}
    }
    // 2. Daha çok grahayla birlikte olan; iki eş lord sayılmaz.
    let company = |s: Sign| {
        positions
            .occupants(s)
            .into_iter()
            .filter(|&g| g != planet && g != node)
            .count() as u8
    };
    if let Some(lord) = stronger(company(planet_sign), company(node_sign)) {
        return pick(lord, 2);
    }
    // 3. Yücelme ve düşüş: düğümlerin yücelme ve düşüş burcu yoktur.
    if exaltation_point(planet).is_some_and(|(s, _)| s == planet_sign) {
        return pick(planet, 3);
    }
    if debilitation_point(planet).is_some_and(|(s, _)| s == planet_sign) {
        return pick(node, 3);
    }
    // 4. Bulunduğu burcun türü.
    if let Some(lord) = stronger(
        kind_rank(sign_kind(planet_sign)),
        kind_rank(sign_kind(node_sign)),
    ) {
        return pick(lord, 4);
    }
    // 5. Burçtan, sayım yönünde daha uzakta olan.
    if let Some(lord) = stronger(
        directed_count(sign, planet_sign),
        directed_count(sign, node_sign),
    ) {
        return pick(lord, 5);
    }
    // 6. Mars ya da Satürn.
    pick(planet, 6)
}

/// Eş lordlu burcun güçlü lordu; diğer burçlarda burcun tek lordu.
pub fn stronger_co_lord(sign: Sign, positions: &Positions) -> Graha {
    co_lord_choice(sign, positions).map_or_else(|| sign_lord(sign), |c| c.lord)
}

/// Burcun Jaimini lordu: `co_lords` açıksa Akrep ve Kova için güçlü eş lord, değilse (ve diğer
/// burçlarda) `dignity::sign_lord` (Akrep: Mars, Kova: Satürn).
pub fn jaimini_lord(sign: Sign, positions: &Positions, co_lords: bool) -> Graha {
    if co_lords {
        stronger_co_lord(sign, positions)
    } else {
        sign_lord(sign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası. Taban: Mars Koç 5°, Ketu Terazi 15°, Satürn Yengeç 5°, Rahu Koç
    /// 15°; diğer grahalar eş lordların burçlarında değil (Güneş İkizler, Ay Başak, Merkür
    /// İkizler, Jüpiter Yay, Venüs Boğa).
    const BASE: [f64; 9] = [65.0, 160.0, 5.0, 70.0, 250.0, 40.0, 95.0, 15.0, 195.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(None, lons)
    }

    fn choice(sign: Sign, p: &Positions) -> (Graha, u8) {
        let c = co_lord_choice(sign, p).expect("co-lorded sign");
        (c.lord, c.step)
    }

    #[test]
    fn only_scorpio_and_aquarius_have_co_lords() {
        let p = with(&[]);
        for s in Sign::ALL {
            match s {
                Sign::Scorpio => assert_eq!(co_lords(s), Some((Mars, Ketu))),
                Sign::Aquarius => assert_eq!(co_lords(s), Some((Saturn, Rahu))),
                _ => {
                    assert_eq!(co_lords(s), None);
                    assert_eq!(co_lord_choice(s, &p), None);
                    assert_eq!(stronger_co_lord(s, &p), sign_lord(s));
                }
            }
            assert_eq!(jaimini_lord(s, &p, false), sign_lord(s));
            assert_eq!(jaimini_lord(s, &p, true), stronger_co_lord(s, &p));
        }
    }

    #[test]
    fn step_1_the_lord_outside_the_sign_wins() {
        // Mars Akrep'te, Ketu Terazi'de: Ketu.
        assert_eq!(choice(Sign::Scorpio, &with(&[(Mars, 215.0)])), (Ketu, 1));
        // Ketu Akrep'te, Mars Koç'ta: Mars.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Ketu, 215.0), (Rahu, 35.0)])),
            (Mars, 1)
        );
        // Rahu Kova'da, Satürn Yengeç'te: Satürn.
        assert_eq!(
            choice(Sign::Aquarius, &with(&[(Rahu, 305.0), (Ketu, 125.0)])),
            (Saturn, 1)
        );
    }

    #[test]
    fn step_2_more_company_wins_and_the_other_co_lord_does_not_count() {
        // Taban: Mars Koç'ta Rahu'yla (1), Ketu Terazi'de yalnız (0): Mars.
        assert_eq!(choice(Sign::Scorpio, &with(&[])), (Mars, 2));
        // Ay da Terazi'ye, Jüpiter de Terazi'ye: Ketu 2, Mars 1.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Moon, 185.0), (Jupiter, 190.0)])),
            (Ketu, 2)
        );
        // Kova: Satürn Yengeç'te yalnız (0), Rahu Koç'ta Mars'la (1): Rahu.
        assert_eq!(choice(Sign::Aquarius, &with(&[])), (Rahu, 2));
        // İkisi aynı burçta (Koç): Mars ve Ketu birbirini saymaz, eşitlik sürer; Mars Koç'ta
        // ne yücelmiş ne düşmüş, burç türü ve uzaklık aynı: 6. ölçüt.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Ketu, 25.0), (Rahu, 205.0)])),
            (Mars, 6)
        );
    }

    #[test]
    fn step_3_exaltation_then_debilitation_of_the_planet_lord() {
        // Mars Oğlak'ta yalnız, Ketu Terazi'de yalnız (eşlik 0–0): Mars yücelmiş.
        assert_eq!(choice(Sign::Scorpio, &with(&[(Mars, 275.0)])), (Mars, 3));
        // Mars Yengeç'te (düşüş); Satürn Aslan'a alınır ki Mars yalnız olsun: Ketu.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Mars, 100.0), (Saturn, 125.0)])),
            (Ketu, 3)
        );
        // Kova: Satürn Terazi'de (yücelme) Ketu'yla; Ketu Kova'nın eş lordu değildir, sayılır
        // (1). Rahu Koç'ta Mars'la (1): eşit; Satürn yücelmiş.
        assert_eq!(
            choice(Sign::Aquarius, &with(&[(Saturn, 200.0)])),
            (Saturn, 3)
        );
        // Satürn Koç'ta (düşüş) Mars ve Rahu'yla: Satürn'ün eşliği Mars (Rahu sayılmaz),
        // Rahu'nun eşliği Mars (Satürn sayılmaz): 1–1; Satürn düşmüş: Rahu.
        assert_eq!(choice(Sign::Aquarius, &with(&[(Saturn, 20.0)])), (Rahu, 3));
    }

    #[test]
    fn step_4_sign_kind_dual_over_fixed_over_movable() {
        // Mars İkizler'de (çift doğalı); Güneş ve Merkür Aslan'a alınır ki Mars yalnız olsun.
        // Ketu Terazi'de (yürük) yalnız. Eşlik 0–0; yücelme ve düşüş yok; çift doğalı > yürük.
        let p = with(&[(Mars, 75.0), (Sun, 125.0), (Mercury, 130.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Mars, 4));
        // Ketu Boğa'da (sabit) Venüs'le (1); Mars Koç'ta (yürük) Güneş'le (1; Rahu Akrep'te):
        // sabit > yürük: Ketu.
        let p = with(&[(Ketu, 45.0), (Rahu, 225.0), (Sun, 12.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Ketu, 4));
    }

    #[test]
    fn step_5_the_lord_farther_in_the_counting_direction() {
        // Akrep ileri sayar: Mars Koç'ta (Akrep'ten 6.), Ketu Terazi'de (Akrep'ten 12.).
        // Eşlik: Mars Rahu'yla (Rahu Akrep'in eş lordu değildir, sayılır: 1), Ketu Ay'la (1).
        // Mars ne yücelmiş ne düşmüş; iki burç da yürük; 6 < 12: Ketu.
        let p = with(&[(Moon, 185.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Ketu, 5));
        // Kova geri sayar: Kova 1, Oğlak 2, Yay 3, Akrep 4, Terazi 5, Başak 6, Aslan 7,
        // Yengeç 8 (Satürn), İkizler 9, Boğa 10, Koç 11 (Rahu). Mars Boğa'ya alınır ki Rahu
        // yalnız olsun (eşlik 0–0); iki burç da yürük; 8 < 11: Rahu.
        let p = with(&[(Mars, 35.0)]);
        assert_eq!(choice(Sign::Aquarius, &p), (Rahu, 5));
        // İleri sayılsaydı Satürn (6.) Rahu'dan (3.) uzak olurdu: yön belirleyicidir.
        assert_eq!(Sign::Cancer.house_from(Sign::Aquarius), 6);
        assert_eq!(Sign::Aries.house_from(Sign::Aquarius), 3);
    }

    #[test]
    fn step_6_the_planet_lord_when_nothing_separates_them() {
        // İkisi de burcun kendisinde: 1. ölçüt ayırmaz, sonrakiler eşittir.
        let p = with(&[(Mars, 215.0), (Ketu, 225.0), (Rahu, 45.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Mars, 6));
        let p = with(&[(Saturn, 305.0), (Rahu, 320.0), (Ketu, 140.0)]);
        assert_eq!(choice(Sign::Aquarius, &p), (Saturn, 6));
    }
}
