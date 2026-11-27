//! Sankranti listesi (yol haritası §4.8): bir aralıkta Güneş'in sidereal burç girişleri ve her
//! giriş anının tithisi, varası ve Ay nakshatrası.
//!
//! Sankranti sidereal bir kavramdır: liste her zodyak kipinde, seçili ayanamsayla sidereal
//! boylamdan kurulur. Tropikal-rasi kipinde olay aramasının Güneş burç girişleri (rasi çerçevesi,
//! K5) başka anlardır; bu liste onlardan bilerek ayrılır. Giriş anı `timeline` yürüyücüsüyle
//! bulunur (çözünürlük 0,05 sn); giriş anının panchangı v1'in `compute_panchang`'ıdır: vara gün
//! doğumunda değişir ve yerin gün doğumuna göre okunur. Girişin kutlu süresi hesaplanmaz.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::ChartWarning;
use crate::panchang::{compute_panchang, Paksha, Vara};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal, walk, GRAHA_STEP_DAYS};
use crate::zodiac::Sign;
use crate::CoreError;

/// Yöntem kimliği (K7, Y5): sidereal giriş, giriş anının panchangı.
pub const SANKRANTI_METHOD: &str = "sankranti.sidereal_v1";
/// Bir çağrının en uzun aralığı, gün: bir takvim yılı ve pay.
pub const SANKRANTI_MAX_RANGE_DAYS: f64 = 400.0;

/// Güneş'in bir sidereal burca girişi ve o anın olguları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Sankranti {
    pub jd_ut: f64,
    /// Girilen sidereal burç.
    pub sign: Sign,
    /// Giriş anının tithisi, 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Giriş anını içeren günün varası (gün, yerin gün doğumunda başlar).
    pub vara: Vara,
    /// Giriş anında Ay'ın nakshatrası (1..=27) ve padası (1..=4).
    pub nakshatra: u8,
    pub pada: u8,
}

/// Bir aralığın Sankranti listesi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SankrantiList {
    /// Zaman sırasıyla.
    pub rows: Vec<Sankranti>,
    /// `LowPrecision`: bir giriş Moshier yedeğiyle hesaplandı. `SunriseUnavailable`: bir giriş
    /// gününde gün doğumu yok; o satırın varası 06:00 yerel ortalama saatte başlayan gündendir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidTransitInput(field)
}

/// `[from_jd_ut, to_jd_ut]` aralığında Güneş'in sidereal burç girişleri: (an, girilen burç),
/// zaman sırasıyla. Aralığın ilk anındaki burç giriş sayılmaz.
pub fn sankranti_moments<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<(f64, Sign)>, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    let in_range = JD_UT_RANGE.contains(&from_jd_ut) && JD_UT_RANGE.contains(&to_jd_ut);
    if !in_range || from_jd_ut >= to_jd_ut || to_jd_ut - from_jd_ut > SANKRANTI_MAX_RANGE_DAYS {
        return Err(invalid("range"));
    }
    let zodiac = sidereal(settings);
    let series = walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        Ok(Sign::of(lon_of(eph, jd, Body::Sun, zodiac)?))
    })?;
    Ok(series
        .windows(2)
        .map(|w| (w[1].start_jd_ut, w[1].value))
        .collect())
}

/// Aralıktaki Sankranti listesi. `geo_lat` (kuzey +) ve `geo_lon` (doğu +) varanın okunduğu
/// yerdir. Girdi hatası yalnızca alan adını taşır (`range`, `place`): yer hataya girmez.
pub fn sankranti_list<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<SankrantiList, CoreError> {
    settings.validate()?;
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(invalid("place"));
    }
    let zodiac = sidereal(settings);
    let (mut low_precision, mut no_sunrise) = (false, false);
    let mut rows = Vec::new();
    for (jd_ut, sign) in sankranti_moments(eph, settings, from_jd_ut, to_jd_ut)? {
        let p = compute_panchang(eph, jd_ut, geo_lat, geo_lon, settings)?;
        no_sunrise |= p.sunrise_jd_ut.is_none();
        // Hassasiyet Güneş ve Ay'dan okunur (`compute_chart` ile aynı kural).
        for body in [Body::Sun, Body::Moon] {
            low_precision |= eph.body(jd_ut, body, zodiac)?.precision == Precision::Moshier;
        }
        rows.push(Sankranti {
            jd_ut,
            sign,
            tithi: p.tithi.no,
            paksha: p.tithi.paksha,
            vara: p.vara,
            nakshatra: p.nakshatra.no,
            pada: p.nakshatra.pada,
        });
    }
    let mut warnings = Vec::new();
    if low_precision {
        warnings.push(ChartWarning::LowPrecision);
    }
    if no_sunrise {
        warnings.push(ChartWarning::SunriseUnavailable);
    }
    Ok(SankrantiList {
        rows,
        warnings,
        method: SANKRANTI_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::settings::ZodiacMode;
    use crate::timeline::graha_sign_series;
    use crate::zodiac::normalize;

    const JD: f64 = 2_451_545.0;
    /// Sahte gök: Güneş `JD` anında tropikal 10°, günde 0,98°; ayanamsa 24° → sidereal 346°.
    /// Sidereal k. burç sınırına (k × 30°) varış: (14 + 30k) / 0,98 gün sonra.
    fn entry_day(k: u32) -> f64 {
        (14.0 + 30.0 * f64::from(k)) / 0.98
    }

    fn list(settings: &CalcSettings, from: f64, to: f64) -> Vec<Sankranti> {
        let list = sankranti_list(&mut FakeEph::default(), settings, 41.0, 29.0, from, to).unwrap();
        assert_eq!(list.method, "sankranti.sidereal_v1");
        assert!(list.warnings.is_empty());
        list.rows
    }

    #[test]
    fn a_year_has_twelve_entries_at_the_moments_the_arithmetic_gives() {
        // 366 günde on iki sınır: 14,29; 44,90; 75,51 … 351,02. gün (on üçüncüsü 381,63).
        let rows = list(&CalcSettings::default(), JD, JD + 366.0);
        assert_eq!(rows.len(), 12);
        for (k, row) in rows.iter().enumerate() {
            let expected = JD + entry_day(k as u32);
            assert!(
                (row.jd_ut - expected).abs() < 1e-6,
                "{k}: {} vs {expected}",
                row.jd_ut
            );
            // İlk giriş Koç'tur (sidereal 346° Balık'tadır); sıra burç sırasıdır.
            assert_eq!(row.sign, Sign::from_index(k % 12), "{k}");
        }
        assert_eq!(rows[0].sign, Sign::Aries);
        assert_eq!(rows[11].sign, Sign::Pisces);
        // Girişler zaman sırasındadır ve aralığın içindedir.
        assert!(rows.windows(2).all(|w| w[0].jd_ut < w[1].jd_ut));
        assert!(rows.iter().all(|r| JD < r.jd_ut && r.jd_ut < JD + 366.0));
    }

    #[test]
    fn an_entry_carries_the_tithi_vara_and_moon_nakshatra_of_its_moment() {
        // İlk giriş: t = 14 / 0,98 = 14,285714 gün. Ay `JD` anında sidereal 71°, günde 13,2°:
        // 71 + 13,2 × 14,285714 = 259,571°. Güneş 0°'dedir: uzanım 259,571°.
        // Tithi: 259,571 / 12 = 21,63 → 22. tithi, Krishna paksha (22 > 15).
        // Nakshatra: 259,571 / 13,3333 = 19,47 → 20. nakshatra (Purva Ashadha); nakshatranın
        // başı 253,333°, içinde 6,238°; 6,238 / 3,3333 = 1,87 → 2. pada.
        // Vara: an JD 2451559,2857. Sahte gün doğumu her gün n + 0,67'dedir: andan önceki son
        // doğuş 2451558,67. Yerel gün (29° D): floor(2451558,67 + 0,5 + 29/360) = 2451559;
        // 2451545 (2000-01-01) Cumartesi'dir ve 14 gün sonrası yine Cumartesi'dir.
        let first = &list(&CalcSettings::default(), JD, JD + 30.0)[0];
        assert_eq!(first.sign, Sign::Aries);
        assert_eq!((first.tithi, first.paksha), (22, Paksha::Krishna));
        assert_eq!((first.nakshatra, first.pada), (20, 2));
        assert_eq!(first.vara, Vara::Saturday);
        // İkinci giriş (Boğa): t = 44 / 0,98 = 44,897959 gün; Ay 71 + 592,653 = 663,653 →
        // 303,653°; Güneş 30°: uzanım 273,653° → 273,653 / 12 = 22,80 → 23. tithi.
        // Nakshatra: 303,653 / 13,3333 = 22,77 → 23. (Dhanishta); başı 293,333°, içinde
        // 10,320°; 10,320 / 3,3333 = 3,10 → 4. pada.
        // Vara: an 2451589,898; önceki doğuş 2451589,67; yerel gün floor(2451590,2506) =
        // 2451590 = 2451545 + 45; 45 = 6 × 7 + 3: Cumartesi + 3 = Salı.
        let rows = list(&CalcSettings::default(), JD, JD + 60.0);
        assert_eq!(rows.len(), 2);
        let second = &rows[1];
        assert_eq!(second.sign, Sign::Taurus);
        assert_eq!((second.tithi, second.paksha), (23, Paksha::Krishna));
        assert_eq!((second.nakshatra, second.pada), (23, 4));
        assert_eq!(second.vara, Vara::Tuesday);
    }

    #[test]
    fn the_sun_crosses_the_sidereal_sign_start_at_the_entry() {
        // İkinci yazım: giriş anının bir saniye öncesinde Güneş önceki burcun sonunda, bir
        // saniye sonrasında girilen burcun başındadır.
        let mut eph = FakeEph::default();
        let settings = CalcSettings::default();
        let zodiac = sidereal(&settings);
        let second = 1.0 / 86_400.0;
        for row in list(&settings, JD, JD + 366.0) {
            let before = lon_of(&mut eph, row.jd_ut - second, Body::Sun, zodiac).unwrap();
            let after = lon_of(&mut eph, row.jd_ut + second, Body::Sun, zodiac).unwrap();
            let start = 30.0 * row.sign.index() as f64;
            assert_eq!(Sign::of(after), row.sign);
            assert_eq!(
                Sign::of(before),
                Sign::from_index((row.sign.index() + 11) % 12)
            );
            assert!(normalize(after - start) < 1e-4, "{:?}", row.sign);
            assert!(normalize(start - before) < 1e-4, "{:?}", row.sign);
        }
    }

    #[test]
    fn the_list_is_sidereal_in_the_tropical_rasi_mode_too() {
        let sidereal_mode = CalcSettings::default();
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        // Liste zodyak kipine bakmaz.
        assert_eq!(
            list(&tropical, JD, JD + 366.0),
            list(&sidereal_mode, JD, JD + 366.0)
        );
        // Rasi çerçevesindeki (tropikal) Güneş burç girişleri başka anlardır: tropikal 10°'den
        // 30°'ye 20 / 0,98 = 20,41 gün; sidereal giriş 14,29. gündedir.
        let series = graha_sign_series(
            &mut FakeEph::default(),
            &tropical,
            Graha::Sun,
            JD,
            JD + 30.0,
        )
        .unwrap();
        assert_eq!(series.len(), 2);
        assert!((series[1].start_jd_ut - (JD + 20.0 / 0.98)).abs() < 1e-6);
        let first = &list(&tropical, JD, JD + 30.0)[0];
        assert!((first.jd_ut - (JD + 14.0 / 0.98)).abs() < 1e-6);
    }

    #[test]
    fn spans_join_without_a_gap_and_a_span_without_an_entry_is_empty() {
        let s = CalcSettings::default();
        assert!(list(&s, JD, JD + 10.0).is_empty());
        let mut joined = list(&s, JD, JD + 200.0);
        joined.extend(list(&s, JD + 200.0, JD + 366.0));
        assert_eq!(joined, list(&s, JD, JD + 366.0));
    }

    #[test]
    fn warnings_report_the_fallback_ephemeris_and_a_day_without_sunrise() {
        let s = CalcSettings::default();
        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let low = sankranti_list(&mut moshier, &s, 41.0, 29.0, JD, JD + 30.0).unwrap();
        assert_eq!(low.warnings, [ChartWarning::LowPrecision]);
        // Gün doğumu yok: gün 06:00 yerel ortalama saatte başlar. İlk giriş JD 2451559,2857:
        // 0,25 gün öncesi 2451559,0357; yerel gün floor(2451559,0357 + 0,5 + 29/360) =
        // 2451559, yine Cumartesi. Öteki olgular değişmez.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let dark = sankranti_list(&mut polar, &s, 41.0, 29.0, JD, JD + 30.0).unwrap();
        assert_eq!(dark.warnings, [ChartWarning::SunriseUnavailable]);
        assert_eq!(dark.rows, list(&s, JD, JD + 30.0));
        assert_eq!(dark.rows[0].vara, Vara::Saturday);
    }

    #[test]
    fn bad_input_names_the_field_and_never_carries_the_place() {
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        let range = Err(CoreError::InvalidTransitInput("range"));
        for (from, to) in [
            (JD, JD),
            (JD + 1.0, JD),
            (JD, JD + SANKRANTI_MAX_RANGE_DAYS + 1.0),
            (f64::NAN, JD),
            (JD, f64::INFINITY),
            (1.0, 2.0),
        ] {
            assert_eq!(
                sankranti_list(&mut eph, &s, 41.0, 29.0, from, to),
                range,
                "{from} {to}"
            );
        }
        // Yer aralıktan önce denetlenir ve hata koordinatı taşımaz.
        for (lat, lon) in [(91.0, 29.0), (41.0, 181.0), (f64::NAN, 29.0)] {
            assert_eq!(
                sankranti_list(&mut eph, &s, lat, lon, JD + 1.0, JD),
                Err(CoreError::InvalidTransitInput("place"))
            );
        }
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            sankranti_list(&mut eph, &old, 41.0, 29.0, JD, JD + 30.0),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
