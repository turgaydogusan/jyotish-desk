//! Sthana bala (konum gücü): uccha, saptavargaja, ojayugma, kendradi ve drekkana kalemleri
//! (ileri modüller spec'i §7, "1. Sthana bala"). Kaynak: BPHS, graha gücü bölümü.
//!
//! Burca ve burç içi dereceye dayalı her kural rasi çerçevesindedir (K5); evler lagna
//! burcundan sayılan rasi evleridir (K4). Birim virupadır.

use crate::dignity::{
    compound_relation, debilitation_point, moolatrikona, natural_relation, own_signs, sign_lord,
    temporal_friend, Dignity,
};
use crate::graha::Graha;
use crate::positions::{GrahaPoint, Positions};
use crate::varga::{varga_position, Varga};
use crate::zodiac::{angular_distance, Sign};

use super::SthanaBala;

/// Saptavargaja balada kullanılan yedi varga.
pub const SAPTAVARGA: [Varga; 7] = [
    Varga::D1,
    Varga::D2,
    Varga::D3,
    Varga::D7,
    Varga::D9,
    Varga::D12,
    Varga::D30,
];

/// Grahanın rasi çerçevesindeki boylamı, [0, 360): burç başı + burç içi derece. `Sidereal`
/// kipinde sidereal boylamın, tropikal-rasi kipinde tropikal boylamın kendisidir.
pub fn rasi_lon(point: &GrahaPoint) -> f64 {
    point.rasi_sign.index() as f64 * 30.0 + point.deg_in_sign
}

/// Tek (oja) burç: Koç, İkizler, Aslan, Terazi, Yay, Kova.
fn is_odd_sign(sign: Sign) -> bool {
    sign.index().is_multiple_of(2)
}

/// Uccha bala: düşüş noktasından açısal uzaklık (0°–180°) / 3. Yücelme noktasında 60, düşüş
/// noktasında 0. `rasi_lon` rasi çerçevesindeki boylamdır. Düğümler 0 verir.
pub fn uccha_bala(graha: Graha, rasi_lon: f64) -> f64 {
    match debilitation_point(graha) {
        Some((sign, deg)) => angular_distance(rasi_lon, sign.index() as f64 * 30.0 + deg) / 3.0,
        None => 0.0,
    }
}

/// Bileşik ilişkinin saptavargaja değeri. `compound_relation` yalnızca bu beş değeri üretir.
fn relation_points(relation: Dignity) -> f64 {
    match relation {
        Dignity::GreatFriend => 20.0,
        Dignity::Friend => 15.0,
        Dignity::Neutral => 10.0,
        Dignity::Enemy => 4.0,
        Dignity::GreatEnemy => 2.0,
        Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign | Dignity::Debilitated => 0.0,
    }
}

/// Grahanın tek bir vargadaki saptavargaja değeri: moolatrikona 45 (yalnızca D1; v1'in
/// moolatrikona derece aralığı), kendi burcu 30, sonra varga burcunun lorduyla bileşik ilişki:
/// çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2. Geçici ilişki her vargada D1
/// burçlarından alınır. Düğümler 0 verir.
pub fn saptavargaja_points(graha: Graha, varga: Varga, positions: &Positions) -> f64 {
    if graha.is_node() {
        return 0.0;
    }
    let point = positions.point(graha);
    let (sign, _) = varga_position(varga, rasi_lon(point));
    if varga == Varga::D1 {
        if let Some((mt_sign, from, to)) = moolatrikona(graha) {
            if sign == mt_sign && point.deg_in_sign >= from && point.deg_in_sign < to {
                return 45.0;
            }
        }
    }
    if own_signs(graha).contains(&sign) {
        return 30.0;
    }
    let lord = sign_lord(sign);
    relation_points(compound_relation(
        natural_relation(graha, lord),
        temporal_friend(point.rasi_sign, positions.sign_of(lord)),
    ))
}

/// Saptavargaja bala: yedi varganın değerlerinin toplamı (14–225).
pub fn saptavargaja_bala(graha: Graha, positions: &Positions) -> f64 {
    SAPTAVARGA
        .into_iter()
        .map(|varga| saptavargaja_points(graha, varga, positions))
        .sum()
}

/// Ojayugma bala: Ay ve Venüs çift burçta, diğer beş graha tek burçta 15; aynı kural navamsa
/// burcu için ayrıca 15 (0, 15 ya da 30). Düğümler 0 verir.
pub fn ojayugma_bala(graha: Graha, rasi_lon: f64) -> f64 {
    if graha.is_node() {
        return 0.0;
    }
    let wants_even = matches!(graha, Graha::Moon | Graha::Venus);
    [Varga::D1, Varga::D9]
        .into_iter()
        .filter(|&varga| is_odd_sign(varga_position(varga, rasi_lon).0) != wants_even)
        .count() as f64
        * 15.0
}

/// Kendradi bala: kendra (1, 4, 7, 10) 60; panaphara (2, 5, 8, 11) 30; apoklima (3, 6, 9, 12)
/// 15. `house` lagnadan sayılan rasi evidir, 1..=12.
pub fn kendradi_bala(house: u8) -> f64 {
    match house % 3 {
        1 => 60.0,
        2 => 30.0,
        _ => 15.0,
    }
}

/// Drekkana bala: eril grahalar (Güneş, Mars, Jüpiter) burcun ilk 10°'sinde, nötrler (Merkür,
/// Satürn) ikinci 10°'de, dişiller (Ay, Venüs) son 10°'de 15; değilse 0. Düğümler 0 verir.
pub fn drekkana_bala(graha: Graha, deg_in_sign: f64) -> f64 {
    let part = match graha {
        Graha::Sun | Graha::Mars | Graha::Jupiter => 0,
        Graha::Mercury | Graha::Saturn => 1,
        Graha::Moon | Graha::Venus => 2,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    let third = if deg_in_sign < 10.0 {
        0
    } else if deg_in_sign < 20.0 {
        1
    } else {
        2
    };
    if part == third {
        15.0
    } else {
        0.0
    }
}

/// Beş kalem ve toplamı. Kendradi lagnaya bağlıdır: doğum saati bilinmiyorsa `None`.
pub fn sthana_bala(graha: Graha, positions: &Positions) -> Option<SthanaBala> {
    let house = positions.house_of(graha)?;
    let point = positions.point(graha);
    let lon = rasi_lon(point);
    let uccha = uccha_bala(graha, lon);
    let saptavargaja = saptavargaja_bala(graha, positions);
    let ojayugma = ojayugma_bala(graha, lon);
    let kendradi = kendradi_bala(house);
    let drekkana = drekkana_bala(graha, point.deg_in_sign);
    Some(SthanaBala {
        uccha,
        saptavargaja,
        ojayugma,
        kendradi,
        drekkana,
        total: uccha + saptavargaja + ojayugma + kendradi + drekkana,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(Some(0.0), lons)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn uccha_is_sixty_at_exaltation_and_zero_at_debilitation() {
        // Yücelme noktaları: Güneş Koç 10°, Ay Boğa 3°, Mars Oğlak 28°, Merkür Başak 15°,
        // Jüpiter Yengeç 5°, Venüs Balık 27°, Satürn Terazi 20°.
        for (g, exalt) in [
            (Sun, 10.0),
            (Moon, 33.0),
            (Mars, 298.0),
            (Mercury, 165.0),
            (Jupiter, 95.0),
            (Venus, 357.0),
            (Saturn, 200.0),
        ] {
            assert!(close(uccha_bala(g, exalt), 60.0), "{g:?} exalted");
            assert!(
                close(uccha_bala(g, exalt + 180.0), 0.0),
                "{g:?} debilitated"
            );
            // Çeyrek uzaklıkta yarım değer; 30° ötede 50 ya da 10.
            assert!(close(uccha_bala(g, exalt + 90.0), 30.0), "{g:?} quarter");
            assert!(close(uccha_bala(g, exalt - 90.0), 30.0), "{g:?} quarter");
            assert!(close(uccha_bala(g, exalt + 30.0), 50.0), "{g:?} thirty");
            assert!(close(uccha_bala(g, exalt + 150.0), 10.0), "{g:?} near fall");
        }
        // Ara değer: Güneş Aslan 15° (135°); düşüş noktası 190°; uzaklık 55° → 18,333…
        assert!(close(uccha_bala(Sun, 135.0), 55.0 / 3.0));
        assert_eq!(uccha_bala(Rahu, 100.0), 0.0);
        assert_eq!(uccha_bala(Ketu, 100.0), 0.0);
    }

    #[test]
    fn saptavargaja_points_by_dignity_and_compound_relation() {
        // Moolatrikona yalnızca D1'de ve derece aralığında: Güneş Aslan 0–20°.
        let p = with(&[(Sun, 130.0)]);
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &p), 45.0);
        // Aslan 25°: aralığın dışı, kendi burcu.
        let p = with(&[(Sun, 145.0)]);
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &p), 30.0);
        // Ay Boğa 2°: moolatrikona aralığından (3–30°) önce; Boğa'nın lordu Venüs. Doğal: Ay →
        // Venüs nötr. Geçici: Venüs Boğa'da, Ay'la aynı burçta (1. burç) → düşman. Nötr +
        // düşman = düşman: 4.
        let p = with(&[(Moon, 32.0)]);
        assert_eq!(saptavargaja_points(Moon, Varga::D1, &p), 4.0);
        // Ay Boğa 10°: moolatrikona.
        let p = with(&[(Moon, 40.0)]);
        assert_eq!(saptavargaja_points(Moon, Varga::D1, &p), 45.0);
        // Mars Akrep'te (220°): kendi burcu, moolatrikona değil.
        assert_eq!(saptavargaja_points(Mars, Varga::D1, &with(&[])), 30.0);

        // Bileşik ilişkinin beş değeri, D1'de. Taban: Güneş Koç 10° (lordu Mars, Akrep'te).
        // Doğal: Güneş → Mars dost. Geçici: Akrep, Koç'tan 8. burç → düşman. Dost + düşman =
        // nötr: 10.
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &with(&[])), 10.0);
        // Mars Boğa'ya (2. burç) alınırsa geçici dost olur: dost + dost = çok dost: 20.
        assert_eq!(
            saptavargaja_points(Sun, Varga::D1, &with(&[(Mars, 40.0)])),
            20.0
        );
        // Merkür İkizler 5° (65°) kendi burcundadır; Yengeç 5°'ye (95°) alınırsa lord Ay'dır
        // (Yengeç 10°'de, aynı burçta): doğal Merkür → Ay düşman, geçici düşman → çok düşman: 2.
        assert_eq!(
            saptavargaja_points(Mercury, Varga::D1, &with(&[(Mercury, 95.0)])),
            2.0
        );
        // Venüs Yay'da (250°): lord Jüpiter de Yay'da (aynı burç → geçici düşman). Doğal Venüs
        // → Jüpiter nötr: düşman: 4.
        assert_eq!(
            saptavargaja_points(Venus, Varga::D1, &with(&[(Venus, 255.0)])),
            4.0
        );
        // Venüs Oğlak'ta (275°): lord Satürn Kova'da (2. burç → geçici dost). Doğal Venüs →
        // Satürn dost: çok dost: 20.
        assert_eq!(
            saptavargaja_points(Venus, Varga::D1, &with(&[(Venus, 275.0)])),
            20.0
        );
        // Jüpiter Boğa'da (45°): lord Venüs de Boğa'da (40°) → geçici düşman; doğal Jüpiter →
        // Venüs düşman: çok düşman: 2. Venüs İkizler'e (70°, 2. burç) alınırsa nötr: 10.
        assert_eq!(
            saptavargaja_points(Jupiter, Varga::D1, &with(&[(Jupiter, 45.0)])),
            2.0
        );
        assert_eq!(
            saptavargaja_points(Jupiter, Varga::D1, &with(&[(Jupiter, 45.0), (Venus, 70.0)])),
            10.0
        );
        // Mars Yengeç'te (95°): lord Ay Yengeç'te (aynı burç → geçici düşman); doğal Mars → Ay
        // dost: nötr 10. Ay Aslan'a (125°, 2. burç) alınırsa çok dost: 20.
        assert_eq!(
            saptavargaja_points(Mars, Varga::D1, &with(&[(Mars, 95.0)])),
            10.0
        );
        assert_eq!(
            saptavargaja_points(Mars, Varga::D1, &with(&[(Mars, 95.0), (Moon, 125.0)])),
            20.0
        );
        // Satürn Yay'da (245°): lord Jüpiter Yay'da → geçici düşman; doğal Satürn → Jüpiter
        // nötr: düşman 4. Jüpiter Oğlak'a (275°) alınırsa dost: 15.
        assert_eq!(
            saptavargaja_points(Saturn, Varga::D1, &with(&[(Saturn, 245.0)])),
            4.0
        );
        assert_eq!(
            saptavargaja_points(
                Saturn,
                Varga::D1,
                &with(&[(Saturn, 245.0), (Jupiter, 275.0)])
            ),
            15.0
        );
        assert_eq!(saptavargaja_points(Rahu, Varga::D1, &with(&[])), 0.0);
    }

    #[test]
    fn saptavargaja_uses_the_varga_sign_and_the_d1_temporal_relation() {
        let p = with(&[]);
        // Güneş Koç 10°: hora (D2) tek burcun ilk yarısı → Aslan: kendi burcu 30; D2'de
        // moolatrikona sayılmaz.
        assert_eq!(saptavargaja_points(Sun, Varga::D2, &p), 30.0);
        // Güneş Koç 10°: navamsa 4. parça (10°–13°20′) → Koç'tan 4.: Yengeç; lord Ay (D1'de
        // Yengeç, Koç'tan 4. burç → geçici dost). Doğal Güneş → Ay dost: çok dost 20.
        assert_eq!(saptavargaja_points(Sun, Varga::D9, &p), 20.0);
        // Güneş Koç 10°: D3 ikinci parça → Koç'tan 5.: Aslan: kendi burcu 30.
        assert_eq!(saptavargaja_points(Sun, Varga::D3, &p), 30.0);
        // Güneş Koç 10°: D12 5. parça (10°–12°30′) → Aslan: 30.
        assert_eq!(saptavargaja_points(Sun, Varga::D12, &p), 30.0);
        // Güneş Koç 10°: D7 3. parça (8°34′–12°51′) → tek burçta kendinden: İkizler; lord Merkür
        // (D1'de İkizler, Koç'tan 3. burç → geçici dost). Doğal Güneş → Merkür nötr: dost 15.
        assert_eq!(saptavargaja_points(Sun, Varga::D7, &p), 15.0);
        // Güneş Koç 10°: trimsamsa, tek burç 10°–18° → Yay; lord Jüpiter (D1'de Yay, Koç'tan
        // 9. burç → geçici düşman). Doğal dost: nötr 10.
        assert_eq!(saptavargaja_points(Sun, Varga::D30, &p), 10.0);
        // Toplam: D1 10 + D2 30 + D3 30 + D7 15 + D9 20 + D12 30 + D30 10 = 145.
        assert_eq!(saptavargaja_bala(Sun, &p), 145.0);
        assert_eq!(saptavargaja_bala(Ketu, &p), 0.0);
    }

    #[test]
    fn ojayugma_counts_the_sign_and_the_navamsa_sign() {
        // Koç 1° (tek burç; navamsa Koç, tek): eriller 30, Ay ve Venüs 0.
        for g in [Sun, Mars, Mercury, Jupiter, Saturn] {
            assert_eq!(ojayugma_bala(g, 1.0), 30.0, "{g:?}");
        }
        for g in [Moon, Venus] {
            assert_eq!(ojayugma_bala(g, 1.0), 0.0, "{g:?}");
        }
        // Koç 5° (tek burç; navamsa 2. parça: Boğa, çift): herkes 15.
        for g in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            assert_eq!(ojayugma_bala(g, 5.0), 15.0, "{g:?}");
        }
        // Boğa 1° (çift burç; navamsa Oğlak, çift): Ay ve Venüs 30, diğerleri 0.
        for g in [Moon, Venus] {
            assert_eq!(ojayugma_bala(g, 31.0), 30.0, "{g:?}");
        }
        for g in [Sun, Mars, Mercury, Jupiter, Saturn] {
            assert_eq!(ojayugma_bala(g, 31.0), 0.0, "{g:?}");
        }
        // Boğa 5° (çift burç; navamsa 2. parça: Kova, tek): herkes 15.
        for g in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            assert_eq!(ojayugma_bala(g, 35.0), 15.0, "{g:?}");
        }
        assert_eq!(ojayugma_bala(Rahu, 1.0), 0.0);
    }

    #[test]
    fn kendradi_by_house_group() {
        let got: Vec<f64> = (1..=12).map(kendradi_bala).collect();
        assert_eq!(
            got,
            [60.0, 30.0, 15.0, 60.0, 30.0, 15.0, 60.0, 30.0, 15.0, 60.0, 30.0, 15.0]
        );
    }

    #[test]
    fn drekkana_by_gender_and_third_of_the_sign() {
        // (burç içi derece, eril, nötr, dişil)
        for (deg, male, neuter, female) in [
            (0.0, 15.0, 0.0, 0.0),
            (9.999, 15.0, 0.0, 0.0),
            (10.0, 0.0, 15.0, 0.0),
            (19.999, 0.0, 15.0, 0.0),
            (20.0, 0.0, 0.0, 15.0),
            (29.999, 0.0, 0.0, 15.0),
        ] {
            for g in [Sun, Mars, Jupiter] {
                assert_eq!(drekkana_bala(g, deg), male, "{g:?} at {deg}");
            }
            for g in [Mercury, Saturn] {
                assert_eq!(drekkana_bala(g, deg), neuter, "{g:?} at {deg}");
            }
            for g in [Moon, Venus] {
                assert_eq!(drekkana_bala(g, deg), female, "{g:?} at {deg}");
            }
        }
        assert_eq!(drekkana_bala(Rahu, 5.0), 0.0);
    }

    #[test]
    fn sthana_sums_the_five_items_and_needs_a_lagna() {
        let p = with(&[]);
        // Güneş Koç 10°, lagna Koç: uccha 60, saptavargaja 145, ojayugma (Koç tek; navamsa
        // Yengeç çift) 15, kendradi (1. ev) 60, drekkana (ikinci 10°: eril için yok) 0.
        let s = sthana_bala(Sun, &p).expect("lagna");
        assert_eq!(
            (s.uccha, s.saptavargaja, s.ojayugma, s.kendradi, s.drekkana),
            (60.0, 145.0, 15.0, 60.0, 0.0)
        );
        assert_eq!(s.total, 280.0);
        assert_eq!(sthana_bala(Sun, &p.without_lagna()), None);
    }

    #[test]
    fn rasi_longitude_rebuilds_the_synthetic_longitude_exactly() {
        let p = with(&[]);
        for (g, lon) in Graha::ALL.into_iter().zip(BASE) {
            assert_eq!(rasi_lon(p.point(g)), lon, "{g:?}");
        }
    }
}
