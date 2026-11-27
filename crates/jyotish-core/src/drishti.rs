//! Graha bakışları (ileri modüller spec'i §4): burç temelli tam bakış ve derece temelli bakış
//! değeri (sphuta drishti).
//!
//! Kaynak: BPHS, bakışlar bölümü. Her graha bulunduğu burçtan 7. burca bakar; ek tam bakışlar
//! Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10'dur. Rahu ve Ketu'ya graha bakışı atanmaz.
//! Sphuta drishti parçalı doğrusaldır (virupa, 0–60); özel bakışlar eklenir ve sonuç 60'ta
//! kesilir. Evler ve burçlar rasi çerçevesindedir (K4, K5); açı sidereal boylamlardan alınır.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::zodiac::{normalize, Sign};
use crate::CoreError;

/// `DrishtiTable::method`: bu dosyadaki yöntemin kimliği.
pub const DRISHTI_METHOD: &str = "graha_drishti.bphs_v1";

/// Tam bakış değeri, virupa.
pub const FULL_DRISHTI: f64 = 60.0;

/// Bakan grahalar: `DrishtiTable::sphuta` satır sırası.
pub const ASPECTING_GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Grahanın tam bakışla baktığı evler (kendi burcundan sayılır), artan sırada. Düğümler için
/// boştur.
pub fn graha_aspect_houses(graha: Graha) -> &'static [u8] {
    match graha {
        Graha::Sun | Graha::Moon | Graha::Mercury | Graha::Venus => &[7],
        Graha::Mars => &[4, 7, 8],
        Graha::Jupiter => &[5, 7, 9],
        Graha::Saturn => &[3, 7, 10],
        Graha::Rahu | Graha::Ketu => &[],
    }
}

/// `from` burcundaki grahanın tam bakışla baktığı burçlar, ev sırasıyla.
pub fn graha_aspect_signs(graha: Graha, from: Sign) -> Vec<Sign> {
    graha_aspect_houses(graha)
        .iter()
        .map(|&h| Sign::from_index(from.index() + usize::from(h) - 1))
        .collect()
}

/// Özel bakış eklemesi: (açı aralığının başı, eklenen virupa). Aralık 30° genişliğindedir.
fn special_bonus(aspecting: Graha) -> &'static [(f64, f64)] {
    match aspecting {
        Graha::Mars => &[(90.0, 15.0), (210.0, 15.0)],
        Graha::Jupiter => &[(120.0, 30.0), (240.0, 30.0)],
        Graha::Saturn => &[(60.0, 45.0), (270.0, 45.0)],
        _ => &[],
    }
}

/// Özel bakışsız temel değer: `angle` = (bakılan − bakan) mod 360, derece.
fn base_drishti(angle: f64) -> f64 {
    let a = angle;
    if a < 30.0 {
        0.0
    } else if a < 60.0 {
        (a - 30.0) / 2.0
    } else if a < 90.0 {
        (a - 60.0) + 15.0
    } else if a < 120.0 {
        (120.0 - a) / 2.0 + 30.0
    } else if a < 150.0 {
        150.0 - a
    } else if a < 180.0 {
        (a - 150.0) * 2.0
    } else if a < 300.0 {
        (300.0 - a) / 2.0
    } else {
        0.0
    }
}

/// `aspecting` grahasının `from_lon` boylamından `to_lon` boylamına bakış değeri, virupa,
/// [0, 60]. Boylamlar sidereal derecedir; bakılan nokta herhangi bir boylam olabilir (graha ya
/// da bhava ortası). Rahu ve Ketu bakan olarak 0 verir.
pub fn sphuta_drishti(aspecting: Graha, from_lon: f64, to_lon: f64) -> f64 {
    if aspecting.is_node() {
        return 0.0;
    }
    let angle = normalize(to_lon - from_lon);
    let bonus: f64 = special_bonus(aspecting)
        .iter()
        .filter(|&&(start, _)| angle >= start && angle < start + 30.0)
        .map(|&(_, add)| add)
        .sum();
    (base_drishti(angle) + bonus).min(FULL_DRISHTI)
}

/// Bir grahanın bir burca tam bakışı ve o burçtaki grahalar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignAspect {
    pub graha: Graha,
    pub target: Sign,
    /// Hedef burçtaki grahalar, `Graha::ALL` sırasında (dokuz graha da sayılır).
    pub occupants: Vec<Graha>,
}

/// `compute_drishti` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DrishtiTable {
    /// Yedi grahanın tam bakışları: `ASPECTING_GRAHAS` sırasında, her graha için ev sırasıyla.
    /// Her zaman 13 kayıt: Güneş, Ay, Merkür ve Venüs birer; Mars, Jüpiter ve Satürn üçer.
    pub sign_aspects: Vec<SignAspect>,
    /// `sphuta[i][j]`: `ASPECTING_GRAHAS[i]`'nin `Graha::ALL[j]`'ye bakış değeri, virupa.
    /// Grahanın kendisine bakışı 0'dır.
    pub sphuta: [[f64; 9]; 7],
    pub method: String,
}

/// Saf hesap: konumlardan bakış tablosu. Lagnaya bağlı değildir.
pub fn drishti_table(positions: &Positions) -> DrishtiTable {
    let sign_aspects = ASPECTING_GRAHAS
        .into_iter()
        .flat_map(|graha| {
            graha_aspect_signs(graha, positions.sign_of(graha))
                .into_iter()
                .map(move |target| SignAspect {
                    graha,
                    target,
                    occupants: positions.occupants(target),
                })
        })
        .collect();
    let sphuta = std::array::from_fn(|i| {
        let from = positions.point(ASPECTING_GRAHAS[i]);
        std::array::from_fn(|j| {
            let to = &positions.grahas[j];
            if to.graha == from.graha {
                0.0
            } else {
                sphuta_drishti(from.graha, from.sidereal_lon, to.sidereal_lon)
            }
        })
    });
    DrishtiTable {
        sign_aspects,
        sphuta,
        method: DRISHTI_METHOD.to_owned(),
    }
}

/// `compute_drishti` komutunun hesabı: temel harita, konumlar, bakış tablosu. Bakış tablosu
/// lagnaya bağlı değildir; `input.time_known` sonucu değiştirmez.
pub fn compute_drishti<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<DrishtiTable, CoreError> {
    Ok(drishti_table(&compute_positions(eph, input)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::CalcSettings;
    use Graha::*;
    use Sign::*;

    #[test]
    fn collector_builds_the_table_from_the_chart_positions() {
        let mut eph = FakeEph::default();
        let mut input = ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings: CalcSettings::default(),
            time_known: true,
            lagna_number: None,
        };
        let table = compute_drishti(&mut eph, &input).unwrap();
        let positions = compute_positions(&mut eph, &input).unwrap();
        assert_eq!(table, drishti_table(&positions));
        // FakeEph (ayanamsa 24°): Güneş sidereal 346°, Ay 71°: açı 85° → (85 − 60) + 15 = 40.
        assert!((table.sphuta[0][1] - 40.0).abs() < 1e-9);
        input.time_known = false;
        assert_eq!(compute_drishti(&mut eph, &input).unwrap(), table);
        input.lat = 91.0;
        assert_eq!(
            compute_drishti(&mut eph, &input),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }

    #[test]
    fn sign_aspects_match_the_hand_table_for_every_graha() {
        // Koç'tan sayım: 3 İkizler, 4 Yengeç, 5 Aslan, 7 Terazi, 8 Akrep, 9 Yay, 10 Oğlak.
        assert_eq!(graha_aspect_signs(Sun, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Moon, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Mercury, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Venus, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Mars, Aries), [Cancer, Libra, Scorpio]);
        assert_eq!(
            graha_aspect_signs(Jupiter, Aries),
            [Leo, Libra, Sagittarius]
        );
        assert_eq!(
            graha_aspect_signs(Saturn, Aries),
            [Gemini, Libra, Capricorn]
        );
        assert!(graha_aspect_signs(Rahu, Aries).is_empty());
        assert!(graha_aspect_signs(Ketu, Aries).is_empty());
        // Burç sonundan sarma: Oğlak'tan 4 Koç, 7 Yengeç, 8 Aslan; Balık'tan 3 Boğa, 10 Yay.
        assert_eq!(graha_aspect_signs(Mars, Capricorn), [Aries, Cancer, Leo]);
        assert_eq!(
            graha_aspect_signs(Saturn, Pisces),
            [Taurus, Virgo, Sagittarius]
        );
        assert_eq!(
            graha_aspect_signs(Jupiter, Sagittarius),
            [Aries, Gemini, Leo]
        );
    }

    #[test]
    fn seventh_aspect_is_symmetric_and_every_graha_has_it() {
        for from in Sign::ALL {
            for g in ASPECTING_GRAHAS {
                let seventh = Sign::from_index(from.index() + 6);
                let targets = graha_aspect_signs(g, from);
                assert!(targets.contains(&seventh), "{g:?} {from:?}");
                // Karşı burçtaki aynı graha geri bakar.
                assert!(graha_aspect_signs(g, seventh).contains(&from));
                assert_eq!(targets.len(), graha_aspect_houses(g).len());
                assert!(!targets.contains(&from));
            }
        }
    }

    #[test]
    fn base_values_at_the_table_points() {
        // Özel bakışı olmayan graha (Güneş) temel tabloyu verir.
        for (angle, want) in [
            (0.0, 0.0),
            (15.0, 0.0),
            (30.0, 0.0),
            (45.0, 7.5),
            (60.0, 15.0),
            (75.0, 30.0),
            (90.0, 45.0),
            (105.0, 37.5),
            (120.0, 30.0),
            (135.0, 15.0),
            (150.0, 0.0),
            (165.0, 30.0),
            (180.0, 60.0),
            (210.0, 45.0),
            (240.0, 30.0),
            (270.0, 15.0),
            (285.0, 7.5),
            (300.0, 0.0),
            (330.0, 0.0),
            (359.999, 0.0),
        ] {
            assert_eq!(sphuta_drishti(Sun, 0.0, angle), want, "angle {angle}");
            // Yalnızca fark önemlidir; 360°'den sarma da aynı sonucu verir.
            let got = sphuta_drishti(Venus, 350.0, 350.0 + angle);
            assert!((got - want).abs() < 1e-9, "wrapped {angle}: {got}");
        }
    }

    #[test]
    fn base_table_is_continuous_at_every_breakpoint() {
        for edge in [30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 300.0] {
            let below = sphuta_drishti(Moon, 0.0, edge - 1e-9);
            let at = sphuta_drishti(Moon, 0.0, edge);
            let above = sphuta_drishti(Moon, 0.0, edge + 1e-9);
            assert!((below - at).abs() < 1e-6, "below {edge}: {below} vs {at}");
            assert!((above - at).abs() < 1e-6, "above {edge}: {above} vs {at}");
        }
    }

    #[test]
    fn special_aspects_are_full_at_their_exact_points() {
        // Mars 4. ve 8. (90°, 210°), Jüpiter 5. ve 9. (120°, 240°), Satürn 3. ve 10. (60°, 270°).
        for (g, angle) in [
            (Mars, 90.0),
            (Mars, 210.0),
            (Jupiter, 120.0),
            (Jupiter, 240.0),
            (Saturn, 60.0),
            (Saturn, 270.0),
        ] {
            assert_eq!(sphuta_drishti(g, 0.0, angle), 60.0, "{g:?} {angle}");
        }
        // Herkes için 7. bakış (180°) tamdır.
        for g in ASPECTING_GRAHAS {
            assert_eq!(sphuta_drishti(g, 0.0, 180.0), 60.0, "{g:?}");
        }
    }

    #[test]
    fn special_bonus_applies_only_inside_its_interval() {
        // Mars [90, 120): taban + 15. 105° → 37,5 + 15 = 52,5. 120° aralık dışı → 30.
        assert_eq!(sphuta_drishti(Mars, 0.0, 105.0), 52.5);
        assert_eq!(sphuta_drishti(Mars, 0.0, 120.0), 30.0);
        assert!((sphuta_drishti(Mars, 0.0, 89.999) - 44.999).abs() < 1e-9);
        // Mars [210, 240): 225° → 37,5 + 15 = 52,5. 240° → 30.
        assert_eq!(sphuta_drishti(Mars, 0.0, 225.0), 52.5);
        assert_eq!(sphuta_drishti(Mars, 0.0, 240.0), 30.0);
        // Jüpiter [120, 150): 135° → 15 + 30 = 45. 150° → 0. [240, 270): 255° → 22,5 + 30.
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 135.0), 45.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 150.0), 0.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 255.0), 52.5);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 270.0), 15.0);
        // Satürn [60, 90): 75° → 30 + 45 = 75, 60'ta kesilir. 90° → 45.
        assert_eq!(sphuta_drishti(Saturn, 0.0, 75.0), 60.0);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 90.0), 45.0);
        // Satürn [270, 300): 285° → 7,5 + 45 = 52,5. 300° → 0.
        assert_eq!(sphuta_drishti(Saturn, 0.0, 285.0), 52.5);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 300.0), 0.0);
        // Başka grahanın özel aralığında ekleme yoktur.
        assert_eq!(sphuta_drishti(Mars, 0.0, 135.0), 15.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 105.0), 37.5);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 225.0), 37.5);
    }

    #[test]
    fn nodes_cast_no_graha_aspect() {
        for angle in [0.0, 90.0, 180.0, 270.0] {
            assert_eq!(sphuta_drishti(Rahu, 0.0, angle), 0.0);
            assert_eq!(sphuta_drishti(Ketu, 0.0, angle), 0.0);
        }
    }

    #[test]
    fn table_lists_thirteen_sign_aspects_with_occupants() {
        // Güneş Koç 10°, Ay Terazi 190°, Mars Koç 5°, Merkür Koç 20°, Jüpiter Aslan 125°,
        // Venüs Boğa 40°, Satürn Oğlak 280°, Rahu Yengeç 100°, Ketu Oğlak 280°.
        let p = Positions::synthetic(
            None,
            [10.0, 190.0, 5.0, 20.0, 125.0, 40.0, 280.0, 100.0, 280.0],
        );
        let t = drishti_table(&p);
        assert_eq!(t.method, "graha_drishti.bphs_v1");
        let got: Vec<(Graha, Sign, Vec<Graha>)> = t
            .sign_aspects
            .iter()
            .map(|a| (a.graha, a.target, a.occupants.clone()))
            .collect();
        assert_eq!(
            got,
            [
                (Sun, Libra, vec![Moon]),
                (Moon, Aries, vec![Sun, Mars, Mercury]),
                (Mars, Cancer, vec![Rahu]),
                (Mars, Libra, vec![Moon]),
                (Mars, Scorpio, vec![]),
                (Mercury, Libra, vec![Moon]),
                // Jüpiter Aslan: 5. Yay, 7. Kova, 9. Koç.
                (Jupiter, Sagittarius, vec![]),
                (Jupiter, Aquarius, vec![]),
                (Jupiter, Aries, vec![Sun, Mars, Mercury]),
                (Venus, Scorpio, vec![]),
                // Satürn Oğlak: 3. Balık, 7. Yengeç, 10. Terazi.
                (Saturn, Pisces, vec![]),
                (Saturn, Cancer, vec![Rahu]),
                (Saturn, Libra, vec![Moon]),
            ]
        );
    }

    #[test]
    fn table_sphuta_matrix_matches_the_function() {
        let p = Positions::synthetic(
            None,
            [10.0, 190.0, 5.0, 20.0, 125.0, 40.0, 280.0, 100.0, 280.0],
        );
        let t = drishti_table(&p);
        // Güneş (10°) → Ay (190°): 180° → 60. Ay → Güneş: 180° → 60.
        assert_eq!(t.sphuta[0][1], 60.0);
        assert_eq!(t.sphuta[1][0], 60.0);
        // Mars (5°) → Rahu (100°): 95° → (120 − 95)/2 + 30 = 42,5; özel +15 → 57,5.
        assert_eq!(t.sphuta[2][7], 57.5);
        // Jüpiter (125°) → Güneş (10°): 245° → (300 − 245)/2 = 27,5; özel +30 → 57,5.
        assert_eq!(t.sphuta[4][0], 57.5);
        // Satürn (280°) → Ay (190°): 270° → 15; özel +45 → 60.
        assert_eq!(t.sphuta[6][1], 60.0);
        // Satürn (280°) → Ketu (280°): 0° → 0. Köşegen 0.
        assert_eq!(t.sphuta[6][8], 0.0);
        for (i, g) in ASPECTING_GRAHAS.into_iter().enumerate() {
            assert_eq!(t.sphuta[i][g as usize], 0.0, "{g:?}");
            for (j, to) in Graha::ALL.into_iter().enumerate() {
                if to != g {
                    assert_eq!(
                        t.sphuta[i][j],
                        sphuta_drishti(g, p.point(g).sidereal_lon, p.point(to).sidereal_lon)
                    );
                }
            }
        }
    }
}
