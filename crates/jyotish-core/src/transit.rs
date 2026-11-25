//! Transit: verilen andaki dokuz grahanın konumu ve natal bhavalardaki yeri (spec §3, §4).
//!
//! v1 kapsamı (UI spec §3.3): transit halkası ve tablosu. Lagna, dignity, yanma, vedha
//! yoktur. "Şimdi" anını çekirdek bilmez; `jd_ut`'yi arayüz verir.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame, raw_grahas, to_sidereal, ChartWarning};
use crate::graha::Graha;
use crate::houses::{house_from_cusps, Bhavas};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};
use crate::CoreError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitGraha {
    pub graha: Graha,
    /// Rasi çerçevesinde boylam (natal haritayla aynı ayarlar verilmelidir).
    pub lon: f64,
    pub sidereal_lon: f64,
    pub speed: f64,
    pub retrograde: bool,
    pub sign: Sign,
    pub deg_in_sign: f64,
    pub nakshatra: u8,
    pub pada: u8,
    /// Natal bhava başlangıçlarına göre ev, 1..=12.
    pub natal_house: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitSnapshot {
    pub jd_ut: f64,
    pub ayanamsa_deg: f64,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub grahas: Vec<TransitGraha>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
}

/// `jd_ut` anının transit konumları. `settings` ve `natal_bhavas` natal haritadan gelir
/// (`chart.meta.settings`, `chart.bhavas`); evler natal bhava başlangıçlarına göre bulunur
/// (Whole Sign başlangıçları burç başlarıdır, sonuç burç sayımıyla aynıdır).
pub fn compute_transits<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    settings: &CalcSettings,
    natal_bhavas: &Bhavas,
) -> Result<TransitSnapshot, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if natal_bhavas.cusps.len() != 12 || !natal_bhavas.cusps.iter().all(|c| c.is_finite()) {
        return Err(CoreError::InvalidTransitInput("natal_bhavas"));
    }
    let ayanamsa = eph.ayanamsa(jd_ut, settings.ayanamsa.sidereal_mode())?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let (raw, low_precision) = raw_grahas(eph, jd_ut, rasi_frame(settings), settings.node)?;
    let grahas = raw
        .iter()
        .map(|r| {
            let sidereal_lon = to_sidereal(settings.zodiac, r.lon, ayanamsa.deg);
            let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
            TransitGraha {
                graha: r.graha,
                lon: r.lon,
                sidereal_lon,
                speed: r.speed,
                retrograde: r.speed < 0.0,
                sign: Sign::of(r.lon),
                deg_in_sign: deg_in_sign(r.lon),
                nakshatra,
                pada,
                natal_house: house_from_cusps(r.lon, &natal_bhavas.cusps),
            }
        })
        .collect();
    let warnings = if low_precision {
        vec![ChartWarning::LowPrecision]
    } else {
        Vec::new()
    };
    Ok(TransitSnapshot {
        jd_ut,
        ayanamsa_deg: ayanamsa.deg,
        grahas,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use ephemeris::Precision;

    use super::*;
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::settings::{HouseSystem, ZodiacMode};
    use crate::zodiac::normalize;

    const JD: f64 = 2_451_545.0;

    fn natal(s: &CalcSettings) -> crate::chart::Chart {
        compute_chart(&mut FakeEph::default(), JD, 41.0, 29.0, s).unwrap()
    }

    #[test]
    fn transit_at_birth_matches_the_chart() {
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            for house_system in [HouseSystem::WholeSign, HouseSystem::Placidus] {
                let s = CalcSettings {
                    zodiac,
                    house_system,
                    ..CalcSettings::default()
                };
                let c = natal(&s);
                let t = compute_transits(&mut FakeEph::default(), JD, &s, &c.bhavas).unwrap();
                assert_eq!((t.jd_ut, t.ayanamsa_deg), (JD, c.ayanamsa_deg));
                assert!(t.warnings.is_empty());
                assert_eq!(t.grahas.len(), 9);
                for (tg, cg) in t.grahas.iter().zip(&c.grahas) {
                    let ctx = format!("{:?} {zodiac:?} {house_system:?}", tg.graha);
                    assert_eq!(tg.graha, cg.graha, "{ctx}");
                    assert_eq!(
                        (tg.lon, tg.sidereal_lon),
                        (cg.lon, cg.sidereal_lon),
                        "{ctx}"
                    );
                    assert_eq!(
                        (tg.speed, tg.retrograde),
                        (cg.speed, cg.retrograde),
                        "{ctx}"
                    );
                    assert_eq!(
                        (tg.sign, tg.deg_in_sign),
                        (cg.sign, cg.deg_in_sign),
                        "{ctx}"
                    );
                    assert_eq!((tg.nakshatra, tg.pada), (cg.nakshatra, cg.pada), "{ctx}");
                    assert_eq!(tg.natal_house, cg.house, "{ctx}");
                }
            }
        }
    }

    #[test]
    fn transits_move_and_fall_in_natal_houses() {
        let s = CalcSettings::default();
        let c = natal(&s);
        // 10 gün sonra (sahte hareket doğrusal): Ay sidereal 71 + 132 = 203° (Terazi),
        // Güneş 346 + 9.8 = 355.8° (Balık). Natal lagna Boğa.
        let t = compute_transits(&mut FakeEph::default(), JD + 10.0, &s, &c.bhavas).unwrap();
        let (sun, moon) = (&t.grahas[0], &t.grahas[1]);
        assert!((moon.lon - 203.0).abs() < 1e-9, "{}", moon.lon);
        assert_eq!((moon.sign, moon.natal_house), (Sign::Libra, 6));
        assert!((sun.lon - 355.8).abs() < 1e-9, "{}", sun.lon);
        assert_eq!((sun.sign, sun.natal_house), (Sign::Pisces, 11));
        assert_eq!(t.grahas[8].lon, normalize(t.grahas[7].lon + 180.0));

        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let t = compute_transits(&mut eph, JD, &s, &c.bhavas).unwrap();
        assert_eq!(t.warnings, [ChartWarning::LowPrecision]);
    }

    #[test]
    fn invalid_input_is_rejected() {
        let s = CalcSettings::default();
        let c = natal(&s);
        let mut eph = FakeEph::default();
        let bad_bhavas = CoreError::InvalidTransitInput("natal_bhavas");
        let short = Bhavas {
            cusps: c.bhavas.cusps[..11].to_vec(),
            ..c.bhavas.clone()
        };
        assert_eq!(
            compute_transits(&mut eph, JD, &s, &short),
            Err(bad_bhavas.clone())
        );
        let mut nan = c.bhavas.clone();
        nan.cusps[3] = f64::NAN;
        assert_eq!(compute_transits(&mut eph, JD, &s, &nan), Err(bad_bhavas));
        assert_eq!(
            compute_transits(&mut eph, 1e300, &s, &c.bhavas),
            Err(CoreError::InvalidJulianDay(1e300))
        );
        let old = CalcSettings {
            schema_version: 2,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_transits(&mut eph, JD, &old, &c.bhavas),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1
            })
        );
    }
}
