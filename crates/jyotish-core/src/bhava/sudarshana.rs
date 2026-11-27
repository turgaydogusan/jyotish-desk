//! Sudarshana chakra (yol haritası §4.6): lagna, Ay ve Güneş'i birinci ev sayan üç halka. Her
//! halkada on iki ev vardır; ev, o halkanın başlangıç burcundan sayılan burç evidir (K4) ve
//! içindeki grahalar rasi çerçevesindeki burçlarından okunur. Halka sırası içten dışadır: lagna,
//! Ay, Güneş. Sudarshana dashası hesaplanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::zodiac::Sign;
use crate::CoreError;

/// Sonucun yöntem kimliği (K7).
pub const SUDARSHANA_METHOD: &str = "sudarshana.v1";

/// Halkanın birinci evini veren nokta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SudarshanaRef {
    Lagna,
    Moon,
    Sun,
}

impl SudarshanaRef {
    /// İçten dışa halka sırası.
    pub const ALL: [SudarshanaRef; 3] = [
        SudarshanaRef::Lagna,
        SudarshanaRef::Moon,
        SudarshanaRef::Sun,
    ];
}

/// Bir halkanın bir evi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaHouse {
    /// 1..=12.
    pub house: u8,
    pub sign: Sign,
    /// Bu burçtaki grahalar, `Graha::ALL` sırasında.
    pub grahas: Vec<Graha>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaRing {
    pub reference: SudarshanaRef,
    /// On iki ev, 1. evden başlayarak. Lagna halkasında doğum saati bilinmiyorsa `None` (K2).
    pub houses: Option<Vec<SudarshanaHouse>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaChakra {
    pub method: String,
    /// Üç halka, `SudarshanaRef::ALL` sırasında (içten dışa).
    pub rings: Vec<SudarshanaRing>,
    /// Doğum saati bilinmiyor: Ay'ın burcu o gün değişmiş olabilir.
    pub approximate: bool,
}

fn ring_houses(positions: &Positions, first: Sign) -> Vec<SudarshanaHouse> {
    (0..12)
        .map(|i| {
            let sign = Sign::from_index(first.index() + i);
            SudarshanaHouse {
                house: (i + 1) as u8,
                sign,
                grahas: positions.occupants(sign),
            }
        })
        .collect()
}

/// Saf hesap: üç halka. Lagna yoksa (doğum saati bilinmiyor) lagna halkasının evleri `None`
/// döner ve sonuç `approximate` işaretlenir.
pub fn sudarshana(positions: &Positions) -> SudarshanaChakra {
    let rings = SudarshanaRef::ALL
        .into_iter()
        .map(|reference| {
            let first = match reference {
                SudarshanaRef::Lagna => positions.lagna_sign,
                SudarshanaRef::Moon => Some(positions.sign_of(Graha::Moon)),
                SudarshanaRef::Sun => Some(positions.sign_of(Graha::Sun)),
            };
            SudarshanaRing {
                reference,
                houses: first.map(|sign| ring_houses(positions, sign)),
            }
        })
        .collect();
    SudarshanaChakra {
        method: SUDARSHANA_METHOD.to_owned(),
        rings,
        approximate: positions.lagna_sign.is_none(),
    }
}

/// `compute_sudarshana` komutunun hesabı: temel haritayı hesaplar ve `sudarshana`'ya verir.
/// Doğum saati bilinmiyorsa lagna halkası boştur; sayı prashnasında lagna sayıdan gelir.
pub fn compute_sudarshana<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<SudarshanaChakra, CoreError> {
    Ok(sudarshana(&compute_positions(eph, input)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::CalcSettings;

    /// El örneği: lagna Akrep 5°; Güneş Koç 10°, Ay Yengeç 5°, Mars Akrep 20°, Merkür Koç
    /// 20°, Jüpiter Yay 10°, Venüs Boğa 10°, Satürn Kova 0°, Rahu Aslan 0°, Ketu Kova 0°.
    fn sample() -> Positions {
        Positions::synthetic(
            Some(215.0),
            [10.0, 95.0, 230.0, 20.0, 250.0, 40.0, 300.0, 120.0, 300.0],
        )
    }

    fn house(ring: &SudarshanaRing, house: u8) -> &SudarshanaHouse {
        &ring.houses.as_ref().expect("houses")[usize::from(house) - 1]
    }

    #[test]
    fn three_rings_count_from_the_lagna_the_moon_and_the_sun() {
        let c = sudarshana(&sample());
        assert_eq!(c.method, SUDARSHANA_METHOD);
        assert!(!c.approximate);
        let refs: Vec<SudarshanaRef> = c.rings.iter().map(|r| r.reference).collect();
        assert_eq!(refs, SudarshanaRef::ALL);
        let (lagna, moon, sun) = (&c.rings[0], &c.rings[1], &c.rings[2]);
        // Lagna halkası: 1. ev Akrep (Mars); 2. ev Yay (Jüpiter); 4. ev Kova (Satürn, Ketu);
        // 6. ev Koç (Güneş, Merkür); 7. ev Boğa (Venüs); 9. ev Yengeç (Ay); 10. ev Aslan (Rahu).
        assert_eq!(house(lagna, 1).sign, Sign::Scorpio);
        assert_eq!(house(lagna, 1).grahas, [Graha::Mars]);
        assert_eq!(house(lagna, 2).grahas, [Graha::Jupiter]);
        assert_eq!(house(lagna, 4).grahas, [Graha::Saturn, Graha::Ketu]);
        assert_eq!(house(lagna, 6).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(lagna, 7).grahas, [Graha::Venus]);
        assert_eq!(house(lagna, 9).grahas, [Graha::Moon]);
        assert_eq!(house(lagna, 10).grahas, [Graha::Rahu]);
        // Ay halkası: 1. ev Yengeç (Ay); 2. ev Aslan (Rahu); 5. ev Akrep (Mars); 6. ev Yay
        // (Jüpiter); 8. ev Kova (Satürn, Ketu); 10. ev Koç (Güneş, Merkür); 11. ev Boğa (Venüs).
        assert_eq!(house(moon, 1).sign, Sign::Cancer);
        assert_eq!(house(moon, 1).grahas, [Graha::Moon]);
        assert_eq!(house(moon, 2).grahas, [Graha::Rahu]);
        assert_eq!(house(moon, 5).grahas, [Graha::Mars]);
        assert_eq!(house(moon, 6).grahas, [Graha::Jupiter]);
        assert_eq!(house(moon, 8).grahas, [Graha::Saturn, Graha::Ketu]);
        assert_eq!(house(moon, 10).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(moon, 11).grahas, [Graha::Venus]);
        // Güneş halkası: 1. ev Koç (Güneş, Merkür); 2. ev Boğa (Venüs); 4. ev Yengeç (Ay);
        // 5. ev Aslan (Rahu); 8. ev Akrep (Mars); 9. ev Yay (Jüpiter); 11. ev Kova (Satürn, Ketu).
        assert_eq!(house(sun, 1).sign, Sign::Aries);
        assert_eq!(house(sun, 1).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(sun, 2).grahas, [Graha::Venus]);
        assert_eq!(house(sun, 4).grahas, [Graha::Moon]);
        assert_eq!(house(sun, 5).grahas, [Graha::Rahu]);
        assert_eq!(house(sun, 8).grahas, [Graha::Mars]);
        assert_eq!(house(sun, 9).grahas, [Graha::Jupiter]);
        assert_eq!(house(sun, 11).grahas, [Graha::Saturn, Graha::Ketu]);
        // Her halkada on iki ev, on iki ayrı burç ve dokuz grahanın her biri tam bir kez.
        for ring in &c.rings {
            let houses = ring.houses.as_ref().expect("houses");
            assert_eq!(houses.len(), 12);
            for (i, h) in houses.iter().enumerate() {
                assert_eq!(usize::from(h.house), i + 1);
                assert_eq!(h.sign, Sign::from_index(houses[0].sign.index() + i));
            }
            let mut all: Vec<Graha> = houses.iter().flat_map(|h| h.grahas.clone()).collect();
            all.sort();
            assert_eq!(all, Graha::ALL);
        }
    }

    #[test]
    fn without_a_birth_time_the_lagna_ring_is_empty_and_the_result_is_approximate() {
        let c = sudarshana(&sample().without_lagna());
        assert!(c.approximate);
        assert_eq!(c.rings.len(), 3);
        assert_eq!(c.rings[0].reference, SudarshanaRef::Lagna);
        assert_eq!(c.rings[0].houses, None);
        assert_eq!(house(&c.rings[1], 1).sign, Sign::Cancer);
        assert_eq!(house(&c.rings[2], 1).sign, Sign::Aries);
    }

    #[test]
    fn collector_reads_the_chart_positions() {
        let mut eph = FakeEph::default();
        let mut input = ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings: CalcSettings::default(),
            time_known: true,
            lagna_number: None,
        };
        let positions = compute_positions(&mut eph, &input).unwrap();
        let c = compute_sudarshana(&mut eph, &input).unwrap();
        assert_eq!(c, sudarshana(&positions));
        // FakeEph: lagna sidereal 46° (Boğa), Ay 71° (İkizler), Güneş 346° (Balık).
        assert_eq!(house(&c.rings[0], 1).sign, Sign::Taurus);
        assert_eq!(house(&c.rings[1], 1).sign, Sign::Gemini);
        assert_eq!(house(&c.rings[2], 1).sign, Sign::Pisces);
        // Sayı prashnası: lagna halkası sayının lagnasından (30 → Yengeç).
        input.lagna_number = Some(30);
        let numbered = compute_sudarshana(&mut eph, &input).unwrap();
        assert_eq!(house(&numbered.rings[0], 1).sign, Sign::Cancer);
        input.time_known = false;
        let unknown = compute_sudarshana(&mut eph, &input).unwrap();
        assert!(unknown.approximate);
        assert_eq!(unknown.rings[0].houses, None);
        input.lat = 91.0;
        assert_eq!(
            compute_sudarshana(&mut eph, &input),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }
}
