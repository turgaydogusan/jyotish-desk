//! Harita hesabı: ayanamsa, dokuz graha, lagna, bhavalar ve graha başına
//! burç, nakshatra, pada, yanma, dignity ve ev.

use ephemeris::{Body, Ephemeris, Precision, Zodiac};
use serde::{Deserialize, Serialize};

use crate::combustion::is_combust;
use crate::dasha::{vimshottari, Dasha, DashaSeed};
use crate::dignity::{dignity, sign_lord};
use crate::graha::{Graha, GrahaPosition};
use crate::houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
use crate::settings::{CalcSettings, HouseSystem, ZodiacMode, JD_UT_RANGE};
use crate::varga::{all_vargas, is_vargottama, VargaChart};
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// `Chart.meta.engine_version`: bu crate'in sürümü.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Ölümcül olmayan durumlar (spec §7); arayüz kodları i18n ile çevirir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ChartWarning {
    /// Güneş veya Ay Moshier yedeğiyle hesaplandı (paketlenen ephemeris aralığı dışında).
    LowPrecision,
    /// Placidus kutup dairesinde hesaplanamadı; bhava başlangıçları Porphyry.
    PlacidusPolarFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartMeta {
    pub engine_version: String,
    pub swe_version: String,
    /// Hesapta kullanılan ayarların tam kopyası.
    pub settings: CalcSettings,
    pub warnings: Vec<ChartWarning>,
}

/// M3 haritası (Plan 06): vargalar ve Vimshottari dahil. Panchang ve ashtakavarga
/// Plan 07'de eklenir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Chart {
    pub meta: ChartMeta,
    pub jd_ut: f64,
    pub geo_lat: f64,
    pub geo_lon: f64,
    /// Nütasyon dahil (gerçek) ayanamsa, derece.
    pub ayanamsa_deg: f64,
    pub lagna: Lagna,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub grahas: Vec<GrahaPosition>,
    pub bhavas: Bhavas,
    /// Her zaman 16 eleman, `Varga::ALL` sırasında; rasi çerçevesindeki boylamlardan.
    pub vargas: Vec<VargaChart>,
    /// Vimshottari, 1–3. seviyeler. Ay'ın sidereal boylamından (her zodyak kipinde).
    pub dasha: Dasha,
}

const PLANETS: [(Graha, Body); 7] = [
    (Graha::Sun, Body::Sun),
    (Graha::Moon, Body::Moon),
    (Graha::Mars, Body::Mars),
    (Graha::Mercury, Body::Mercury),
    (Graha::Jupiter, Body::Jupiter),
    (Graha::Venus, Body::Venus),
    (Graha::Saturn, Body::Saturn),
];

/// Ephemeris çıktısı sonlu değilse hata verir; yanlış harita başarı gibi dönmesin.
fn finite(v: f64, what: &'static str) -> Result<f64, CoreError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(CoreError::NonFiniteResult(what))
    }
}

fn finite_body(p: &ephemeris::BodyPosition) -> Result<(), CoreError> {
    finite(p.lon, "graha")?;
    finite(p.lat, "graha")?;
    finite(p.lon_speed, "graha")?;
    Ok(())
}

/// Hesap anı ve konum denetimi; `compute_chart` ve `compute_panchang` ortak kullanır.
pub(crate) fn check_moment(jd_ut: f64, geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(CoreError::InvalidLocation {
            lat: geo_lat,
            lon: geo_lon,
        });
    }
    Ok(())
}

/// Rasi çerçevesinde ham konum.
struct Raw {
    graha: Graha,
    lon: f64,
    lat: f64,
    speed: f64,
}

/// `jd_ut` anı ve (`geo_lat` kuzey +, `geo_lon` doğu +) konumu için haritayı hesaplar.
///
/// Rasi çerçevesi `settings.zodiac`'a göre seçilir: `Sidereal` kipinde ephemeris sidereal
/// sorulur, `TropicalRasiSiderealNakshatra` kipinde tropikal sorulur ve sidereal boylam
/// `tropikal − ayanamsa` ile bulunur. SE'nin sidereal hesabı da aynı çıkarmayı yapar.
pub fn compute_chart<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
) -> Result<Chart, CoreError> {
    settings.validate()?;
    check_moment(jd_ut, geo_lat, geo_lon)?;

    let mode = settings.ayanamsa.sidereal_mode();
    let ayanamsa = eph.ayanamsa(jd_ut, mode)?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let frame = match settings.zodiac {
        ZodiacMode::Sidereal => Zodiac::Sidereal(mode),
        ZodiacMode::TropicalRasiSiderealNakshatra => Zodiac::Tropical,
    };
    let to_sidereal = |frame_lon: f64| match settings.zodiac {
        ZodiacMode::Sidereal => frame_lon,
        ZodiacMode::TropicalRasiSiderealNakshatra => normalize(frame_lon - ayanamsa.deg),
    };

    let mut warnings = Vec::new();
    let mut raw = Vec::with_capacity(9);
    for (graha, body) in PLANETS {
        let p = eph.body(jd_ut, body, frame)?;
        finite_body(&p)?;
        // Hassasiyet Güneş/Ay'dan alınır (Plan 01 notu: ayanamsa bayrağı güvenilmez).
        if matches!(graha, Graha::Sun | Graha::Moon)
            && p.precision == Precision::Moshier
            && !warnings.contains(&ChartWarning::LowPrecision)
        {
            warnings.push(ChartWarning::LowPrecision);
        }
        raw.push(Raw {
            graha,
            lon: p.lon,
            lat: p.lat,
            speed: p.lon_speed,
        });
    }
    let rahu = eph.body(jd_ut, settings.node.body(), frame)?;
    finite_body(&rahu)?;
    raw.push(Raw {
        graha: Graha::Rahu,
        lon: rahu.lon,
        lat: rahu.lat,
        speed: rahu.lon_speed,
    });
    // Ketu tanım gereği Rahu'nun tam karşısıdır; hareketi Rahu ile aynıdır.
    raw.push(Raw {
        graha: Graha::Ketu,
        lon: normalize(rahu.lon + 180.0),
        lat: -rahu.lat,
        speed: rahu.lon_speed,
    });

    let houses = eph.houses(
        jd_ut,
        geo_lat,
        geo_lon,
        settings.house_system.ephemeris_system(),
        frame,
    )?;
    finite(houses.asc, "lagna")?;
    for c in houses.cusps {
        finite(c, "bhavas")?;
    }
    if houses.porphyry_fallback {
        warnings.push(ChartWarning::PlacidusPolarFallback);
    }
    let lagna = Lagna::new(houses.asc, to_sidereal(houses.asc));
    let cusps = match settings.house_system {
        HouseSystem::WholeSign => whole_sign_cusps(lagna.sign),
        HouseSystem::Sripati | HouseSystem::Placidus => houses.cusps.to_vec(),
    };

    let sun_lon = raw[0].lon;
    // Burç lordları her zaman ilk 7 grahadan biridir; bulunamaması mümkün değildir.
    let sign_of = |g: Graha| raw.iter().find(|r| r.graha == g).map(|r| Sign::of(r.lon));
    let grahas: Vec<GrahaPosition> = raw
        .iter()
        .map(|r| {
            let sign = Sign::of(r.lon);
            let deg = deg_in_sign(r.lon);
            let sidereal_lon = to_sidereal(r.lon);
            let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
            let retrograde = r.speed < 0.0;
            let lord_sign = sign_of(sign_lord(sign)).unwrap_or(sign);
            GrahaPosition {
                graha: r.graha,
                lon: r.lon,
                sidereal_lon,
                lat: r.lat,
                speed: r.speed,
                retrograde,
                sign,
                deg_in_sign: deg,
                nakshatra,
                pada,
                combust: is_combust(
                    r.graha,
                    r.lon,
                    sun_lon,
                    retrograde,
                    &settings.combustion_orbs,
                ),
                dignity: dignity(r.graha, sign, deg, lord_sign),
                house: match settings.house_system {
                    HouseSystem::WholeSign => whole_sign_house(sign, lagna.sign),
                    HouseSystem::Sripati | HouseSystem::Placidus => house_from_cusps(r.lon, &cusps),
                },
                vargottama: is_vargottama(r.lon),
            }
        })
        .collect();

    let points: Vec<(Graha, f64)> = raw.iter().map(|r| (r.graha, r.lon)).collect();
    let vargas = all_vargas(lagna.lon, &points);
    // raw[1] her zaman Ay'dır (PLANETS sırası); nakshatra her kipte sidereal'dir.
    let dasha = vimshottari(DashaSeed {
        birth_jd_ut: jd_ut,
        moon_sidereal_lon: to_sidereal(raw[1].lon),
        year_days: settings.dasha_year,
    })?;
    Ok(Chart {
        meta: ChartMeta {
            engine_version: ENGINE_VERSION.to_owned(),
            swe_version: eph.swe_version(),
            settings: settings.clone(),
            warnings,
        },
        jd_ut,
        geo_lat,
        geo_lon,
        ayanamsa_deg: ayanamsa.deg,
        lagna,
        grahas,
        bhavas: Bhavas {
            system: settings.house_system,
            cusps,
        },
        vargas,
        dasha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::DashaSeed;
    use crate::dignity::Dignity;
    use crate::fake::FakeEph;
    use crate::settings::NodeKind;
    use crate::varga::Varga;

    const JD: f64 = 2_451_545.0;

    fn chart(eph: &mut FakeEph, s: &CalcSettings) -> Chart {
        compute_chart(eph, JD, 41.0, 29.0, s).unwrap()
    }

    fn g(c: &Chart, graha: Graha) -> &GrahaPosition {
        c.grahas.iter().find(|p| p.graha == graha).unwrap()
    }

    fn tropical_rasi() -> CalcSettings {
        CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        }
    }

    #[test]
    fn nine_grahas_in_traditional_order() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let order: Vec<Graha> = c.grahas.iter().map(|p| p.graha).collect();
        assert_eq!(order, Graha::ALL);
    }

    #[test]
    fn ketu_is_exactly_opposite_rahu_and_shares_its_motion() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let (rahu, ketu) = (g(&c, Graha::Rahu), g(&c, Graha::Ketu));
        assert_eq!(rahu.lon, 96.0); // 120 − 24
        assert_eq!(ketu.lon, 276.0);
        assert_eq!(ketu.speed, rahu.speed);
        assert!(rahu.retrograde && ketu.retrograde);
        assert_eq!(rahu.dignity, None);
        assert_eq!(ketu.dignity, None);
        assert!(!rahu.combust && !ketu.combust);
    }

    #[test]
    fn sidereal_mode_uses_sidereal_longitude_everywhere() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let sun = g(&c, Graha::Sun);
        assert_eq!(sun.lon, 346.0);
        assert_eq!(sun.sidereal_lon, 346.0);
        assert_eq!(sun.sign, Sign::Pisces);
        assert!((sun.deg_in_sign - 16.0).abs() < 1e-12);
        assert_eq!((sun.nakshatra, sun.pada), (26, 4));
        assert_eq!(c.ayanamsa_deg, 24.0);
        assert_eq!(c.lagna.lon, 46.0);
        assert_eq!(c.lagna.sign, Sign::Taurus);
        assert_eq!(sun.house, 11);
    }

    #[test]
    fn tropical_rasi_takes_sign_from_tropical_and_nakshatra_from_sidereal() {
        let c = chart(&mut FakeEph::default(), &tropical_rasi());
        let sun = g(&c, Graha::Sun);
        assert_eq!(sun.lon, 10.0);
        assert_eq!(sun.sign, Sign::Aries);
        assert!((sun.deg_in_sign - 10.0).abs() < 1e-12);
        assert_eq!(sun.sidereal_lon, 346.0);
        assert_eq!((sun.nakshatra, sun.pada), (26, 4));
        assert_eq!(sun.dignity, Some(Dignity::Exalted));
        assert_eq!(c.lagna.lon, 70.0);
        assert_eq!(c.lagna.sign, Sign::Gemini);
        assert_eq!(c.lagna.sidereal_lon, 46.0);
        assert_eq!((c.lagna.nakshatra, c.lagna.pada), (4, 2));
        assert_eq!(sun.house, 11); // Koç, İkizler'den 11.
        assert_eq!(c.bhavas.cusps[0], 60.0);
    }

    #[test]
    fn whole_sign_and_placidus_place_by_different_rules() {
        // Ay sidereal 71° (İkizler 11°); lagna 46° (Boğa 16°).
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        assert_eq!(g(&c, Graha::Moon).house, 2);
        assert_eq!(c.bhavas.system, HouseSystem::WholeSign);
        assert_eq!(c.bhavas.cusps.len(), 12);
        assert_eq!(c.bhavas.cusps[0], 30.0);

        let mut eph = FakeEph::default();
        let s = CalcSettings {
            house_system: HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let c = chart(&mut eph, &s);
        assert_eq!(
            eph.last_house_system,
            Some(ephemeris::HouseSystem::Placidus)
        );
        assert_eq!(c.bhavas.cusps[0], 46.0);
        assert_eq!(g(&c, Graha::Moon).house, 1); // 71 − 46 = 25° → 1. bhava
    }

    #[test]
    fn true_node_selects_true_node_body() {
        let s = CalcSettings {
            node: NodeKind::True,
            ..CalcSettings::default()
        };
        let c = chart(&mut FakeEph::default(), &s);
        let rahu = g(&c, Graha::Rahu);
        assert_eq!(rahu.lon, 101.0); // 125 − 24
        assert!(!rahu.retrograde);
        assert_eq!(g(&c, Graha::Ketu).lon, 281.0);
    }

    #[test]
    fn combustion_dignity_and_retrograde_are_filled() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        // Merkür sidereal 356°, Güneş 346°: 10° ≤ 14°.
        assert!(g(&c, Graha::Mercury).combust);
        // Venüs sidereal 16°: Güneş'e 30°.
        assert!(!g(&c, Graha::Venus).combust);
        assert!(!g(&c, Graha::Sun).combust);
        // Güneş Balık'ta; lord Jüpiter sidereal 226° (Akrep), Balık'tan 9. → geçici düşman;
        // Güneş→Jüpiter doğal dost → Neutral.
        assert_eq!(g(&c, Graha::Sun).dignity, Some(Dignity::Neutral));
        assert!(g(&c, Graha::Jupiter).retrograde);
        assert!(!g(&c, Graha::Mars).retrograde);
    }

    #[test]
    fn low_precision_warning_comes_from_sun_and_moon() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let c = chart(&mut eph, &CalcSettings::default());
        assert_eq!(c.meta.warnings, [ChartWarning::LowPrecision]);
    }

    #[test]
    fn low_precision_ignores_bodies_other_than_sun_and_moon() {
        let warnings = |bodies: Vec<Body>| {
            let mut eph = FakeEph {
                moshier_bodies: bodies,
                ..FakeEph::default()
            };
            chart(&mut eph, &CalcSettings::default()).meta.warnings
        };
        assert_eq!(warnings(vec![Body::Mars]), []);
        assert_eq!(warnings(vec![Body::MeanNode]), []);
        assert_eq!(warnings(vec![Body::Moon]), [ChartWarning::LowPrecision]);
        assert_eq!(
            warnings(vec![Body::Sun, Body::Moon]),
            [ChartWarning::LowPrecision]
        );
    }

    #[test]
    fn non_finite_ephemeris_output_is_an_error() {
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            let s = CalcSettings {
                zodiac,
                ..CalcSettings::default()
            };
            let mut eph = FakeEph {
                ayanamsa: f64::NAN,
                ..FakeEph::default()
            };
            assert_eq!(
                compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap_err(),
                CoreError::NonFiniteResult("ayanamsa")
            );
            let mut eph = FakeEph::default().with(Body::Mars, f64::NAN, 1.0);
            assert_eq!(
                compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap_err(),
                CoreError::NonFiniteResult("graha")
            );
            let mut eph = FakeEph {
                asc: f64::NAN,
                ..FakeEph::default()
            };
            assert_eq!(
                compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap_err(),
                CoreError::NonFiniteResult("lagna")
            );
        }
    }

    #[test]
    fn placidus_polar_fallback_becomes_a_warning() {
        let mut eph = FakeEph {
            porphyry_fallback: true,
            ..FakeEph::default()
        };
        let s = CalcSettings {
            house_system: HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let c = chart(&mut eph, &s);
        assert_eq!(c.meta.warnings, [ChartWarning::PlacidusPolarFallback]);
    }

    #[test]
    fn meta_carries_versions_and_a_settings_copy() {
        let s = CalcSettings {
            ayanamsa: crate::settings::AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        let c = compute_chart(&mut FakeEph::default(), JD, -33.9, 151.2, &s).unwrap();
        assert_eq!(c.meta.engine_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(c.meta.swe_version, "fake-2.10");
        assert_eq!(c.meta.settings, s);
        assert!(c.meta.warnings.is_empty());
        assert_eq!((c.jd_ut, c.geo_lat, c.geo_lon), (JD, -33.9, 151.2));
    }

    #[test]
    fn rejects_invalid_input() {
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        assert_eq!(
            compute_chart(&mut eph, JD, 91.0, 0.0, &s),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 0.0
            })
        );
        assert_eq!(
            compute_chart(&mut eph, JD, 0.0, 180.5, &s),
            Err(CoreError::InvalidLocation {
                lat: 0.0,
                lon: 180.5
            })
        );
        assert!(matches!(
            compute_chart(&mut eph, f64::NAN, 0.0, 0.0, &s),
            Err(CoreError::InvalidJulianDay(_))
        ));
        // JD_UT_RANGE (D2): sonlu ama SE aralığı dışındaki anlar reddedilir.
        for jd in [1e300, -1e300, 625_001.0, 2_817_998.0] {
            assert_eq!(
                compute_chart(&mut eph, jd, 0.0, 0.0, &s),
                Err(CoreError::InvalidJulianDay(jd)),
                "{jd}"
            );
        }
        assert!(compute_chart(&mut eph, 625_002.0, 0.0, 0.0, &s).is_ok());
        assert!(compute_chart(&mut eph, 2_817_997.0, 0.0, 0.0, &s).is_ok());
        let bad = CalcSettings {
            schema_version: 2,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_chart(&mut eph, JD, 0.0, 0.0, &bad),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1
            })
        );
    }

    #[test]
    fn chart_serde_round_trip() {
        let c = chart(&mut FakeEph::default(), &tropical_rasi());
        let json = serde_json::to_string(&c).unwrap();
        let back: Chart = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn vargas_cover_all_sixteen_from_the_rasi_frame() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let order: Vec<Varga> = c.vargas.iter().map(|v| v.varga).collect();
        assert_eq!(order, Varga::ALL);
        let d1 = &c.vargas[0];
        assert_eq!(d1.lagna_sign, c.lagna.sign);
        for (p, g) in d1.placements.iter().zip(&c.grahas) {
            assert_eq!((p.graha, p.sign), (g.graha, g.sign));
            assert!((p.deg - g.deg_in_sign).abs() < 1e-12);
        }
        let d9 = c.vargas.iter().find(|v| v.varga == Varga::D9).unwrap();
        assert_eq!(d9.lagna_sign, Sign::Taurus);
        // Güneş sidereal 346°: floor(103.8) = 103 → Akrep.
        assert_eq!(d9.placements[0].sign, Sign::Scorpio);

        // Tropikal-rasi kipinde vargalar tropikal boylamdan: Güneş 10° → floor(3.0) = 3 → Yengeç.
        let t = chart(&mut FakeEph::default(), &tropical_rasi());
        let d9 = t.vargas.iter().find(|v| v.varga == Varga::D9).unwrap();
        assert_eq!(d9.placements[0].sign, Sign::Cancer);
    }

    #[test]
    fn vargottama_flag_compares_d1_and_d9() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        // Sidereal: Güneş 346, Ay 71, Mars 176, Merkür 356, Jüpiter 226, Venüs 16, Satürn 276,
        // Rahu 96, Ketu 276. Yalnızca Jüpiter (Akrep 16°) D9'da da Akrep'tir.
        let vargottama: Vec<Graha> = c
            .grahas
            .iter()
            .filter(|p| p.vargottama)
            .map(|p| p.graha)
            .collect();
        assert_eq!(vargottama, [Graha::Jupiter]);
    }

    #[test]
    fn dasha_comes_from_the_sidereal_moon_in_both_modes() {
        for s in [CalcSettings::default(), tropical_rasi()] {
            let c = chart(&mut FakeEph::default(), &s);
            // Ay sidereal 71°: Ardra (6.), lord Rahu; kesir (71 − 66.6̄) / 13.3̄ = 0.325.
            assert_eq!(
                c.dasha.seed,
                DashaSeed {
                    birth_jd_ut: JD,
                    moon_sidereal_lon: 71.0,
                    year_days: 365.25
                }
            );
            let first = &c.dasha.periods[0];
            assert_eq!(first.lords, [Graha::Rahu]);
            assert!((first.start_jd_ut - (JD - 0.325 * 18.0 * 365.25)).abs() < 1e-6);
            assert_eq!(c.dasha.periods.len(), 819);
        }
    }
}
