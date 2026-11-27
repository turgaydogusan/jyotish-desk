//! Varshaphala raporu ve toplayıcısı (K1, K3): doğum haritası, dönüş anı, yıllık harita, muntha
//! ve mudda dasa. Rapor istek üzerine hesaplanır ve saklanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{check_moment, compute_chart, compute_chart_for, Chart, ChartWarning};
use crate::graha::Graha;
use crate::positions::ChartInput;
use crate::CoreError;

use super::mudda::{mudda_dasha, MuddaDasha};
use super::muntha::{year_facts, YearFacts};
use super::solar_return::{completed_years_at, varsha_span};
use super::VARSHA_METHOD;

/// İstenen yıl.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarshaYear {
    /// Tamamlanan yıl: 0 doğum yılıdır, 1 ilk dönüşle başlayan yıldır.
    Age(u8),
    /// Bu anda (UT Jülyen günü) süren yıl.
    At(f64),
}

/// `compute_varsha` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VarshaReport {
    /// Tamamlanan yıl: yıl, `age`. dönüşle başlar.
    pub age: u8,
    /// Yılın başı: Güneş'in doğumdaki sidereal boylamına döndüğü an.
    pub start_jd_ut: f64,
    /// Yılın sonu: sonraki dönüş.
    pub end_jd_ut: f64,
    /// Yanlışsa doğum saati bilinmiyordur: yıllık harita ve lagnaya bağlı olgular verilmez;
    /// dönüş anı ve mudda dasa yaklaşıktır (Güneş'in ve Ay'ın doğumdaki yeri gün içinde değişir).
    pub time_known: bool,
    /// Yıllık harita: dönüş anında, istenen yerde, doğum haritasının ayarlarıyla. Doğum saati
    /// bilinmiyorsa `None`.
    pub chart: Option<Chart>,
    /// Yılın lagnaya bağlı olguları (yıllık lagna, doğum lagnası, muntha); doğum saati
    /// bilinmiyorsa `None`.
    pub facts: Option<YearFacts>,
    pub mudda: MuddaDasha,
    /// Doğum haritasının ya da yıllık haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `compute_varsha` komutunun hesabı. Doğum haritası `compute_chart_for` ile hesaplanır (sayı
/// prashnasında doğum lagnası sayıdan gelir); yıl Güneş'in doğumdaki sidereal boylamına
/// dönüşüyle başlar; yıllık harita dönüş anında ve `place` yerinde (`None`: doğum yeri; `Some`:
/// kuzey + enlem, doğu + boylam) aynı ayarlarla hesaplanır. `input.time_known` yanlışsa yıllık
/// harita ve `facts` boştur. Ayar, an ve konum denetimi `compute_chart`'ınkidir; yaş
/// `InvalidDashaInput("age")`, yer `InvalidLocation` ile reddedilir.
pub fn compute_varsha<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    year: VarshaYear,
    place: Option<(f64, f64)>,
) -> Result<VarshaReport, CoreError> {
    let natal = compute_chart_for(eph, input)?;
    // Yer, doğum saati bilinmese de denetlenir: geçersiz yer sessizce geçmez.
    let (lat, lon) = place.unwrap_or((input.lat, input.lon));
    check_moment(input.jd_ut, lat, lon)?;
    let sun = natal.grahas[Graha::Sun as usize].sidereal_lon;
    let age = match year {
        VarshaYear::Age(age) => age,
        VarshaYear::At(jd) => completed_years_at(eph, &input.settings, sun, input.jd_ut, jd)?,
    };
    let (start_jd_ut, end_jd_ut) = varsha_span(eph, &input.settings, sun, input.jd_ut, age)?;
    let mudda = mudda_dasha(
        natal.grahas[Graha::Moon as usize].nakshatra,
        age,
        start_jd_ut,
        end_jd_ut,
    )?;
    let low = |chart: &Chart| chart.meta.warnings.contains(&ChartWarning::LowPrecision);
    let mut low_precision = low(&natal);
    let (chart, facts) = if input.time_known {
        let annual = compute_chart(eph, start_jd_ut, lat, lon, &input.settings)?;
        low_precision |= low(&annual);
        let facts = year_facts(natal.lagna.sign, annual.lagna.sign, age);
        (Some(annual), Some(facts))
    } else {
        (None, None)
    };
    Ok(VarshaReport {
        age,
        start_jd_ut,
        end_jd_ut,
        time_known: input.time_known,
        chart,
        facts,
        mudda,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: VARSHA_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::varsha::{solar_return_jd, MAX_VARSHA_AGE, SIDEREAL_YEAR_DAYS};
    use crate::zodiac::{angular_distance, Sign};
    use ephemeris::{Body, Precision};

    const JD: f64 = 2_451_545.0;
    const ARCSEC: f64 = 1.0 / 3600.0;

    /// Güneş tam bir sidereal yılda döner; lagna günde bir tur atar (yıllık lagna doğum
    /// lagnasından ayrışsın diye).
    fn eph() -> FakeEph {
        FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default().with(Body::Sun, 10.0, 360.0 / SIDEREAL_YEAR_DAYS)
        }
    }

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    #[test]
    fn the_report_carries_the_year_the_annual_chart_muntha_and_mudda() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let r = compute_varsha(&mut eph, &i, VarshaYear::Age(3), None).unwrap();
        assert_eq!((r.age, r.time_known), (3, true));
        assert_eq!(r.method, VARSHA_METHOD);
        assert!(r.warnings.is_empty());
        // Yıl üçüncü dönüşte başlar ve bir sidereal yıl sürer.
        assert!((r.start_jd_ut - (JD + 3.0 * SIDEREAL_YEAR_DAYS)).abs() < 1e-6);
        assert!((r.end_jd_ut - r.start_jd_ut - SIDEREAL_YEAR_DAYS).abs() < 1e-6);
        // Yıllık harita dönüş anının ve doğum yerinin haritasıdır; Güneş doğum boylamındadır.
        let natal = compute_chart_for(&mut eph, &i).unwrap();
        let chart = r.chart.as_ref().expect("annual chart");
        assert_eq!(
            (chart.jd_ut, chart.geo_lat, chart.geo_lon),
            (r.start_jd_ut, 41.0, 29.0)
        );
        assert_eq!(chart.meta.settings, i.settings);
        assert!(
            angular_distance(chart.grahas[0].sidereal_lon, natal.grahas[0].sidereal_lon) < ARCSEC
        );
        // Doğum lagnası sidereal 46° Boğa; yıl 3: muntha Aslan. Yıllık lagna: 70° + 360° ×
        // 0,769089 gün = 346,87° tropikal, 322,87° sidereal: Kova. Aslan, Kova'dan 7. evdir.
        let facts = r.facts.expect("facts");
        assert_eq!(facts, year_facts(Sign::Taurus, Sign::Aquarius, 3));
        assert_eq!(chart.lagna.sign, Sign::Aquarius);
        assert_eq!(
            (
                facts.muntha.sign,
                facts.muntha.natal_house,
                facts.muntha.annual_house
            ),
            (Sign::Leo, 4, 7)
        );
        // Ay sidereal 71°: 6. nakshatra (Ardra). 6 + 3 − 2 = 7 → Merkür; yıl dokuz dönemle dolar.
        assert_eq!(
            (r.mudda.birth_nakshatra, r.mudda.age, r.mudda.first_lord),
            (6, 3, Graha::Mercury)
        );
        assert_eq!(r.mudda.periods[0].start_jd_ut, r.start_jd_ut);
        assert_eq!(r.mudda.periods.last().unwrap().end_jd_ut, r.end_jd_ut);
    }

    #[test]
    fn the_place_moves_the_annual_chart_only_and_is_validated() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let birth = compute_varsha(&mut eph, &i, VarshaYear::Age(3), None).unwrap();
        let moved = compute_varsha(&mut eph, &i, VarshaYear::Age(3), Some((-33.9, 151.2))).unwrap();
        let chart = moved.chart.as_ref().expect("annual chart");
        assert_eq!((chart.geo_lat, chart.geo_lon), (-33.9, 151.2));
        // Dönüş anı ve mudda dasa yere bağlı değildir.
        assert_eq!(
            (moved.start_jd_ut, moved.end_jd_ut),
            (birth.start_jd_ut, birth.end_jd_ut)
        );
        assert_eq!(moved.mudda, birth.mudda);
        for time_known in [true, false] {
            let i = input(CalcSettings::default(), time_known);
            assert_eq!(
                compute_varsha(&mut eph, &i, VarshaYear::Age(3), Some((91.0, 0.0))),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 0.0
                }),
                "time_known {time_known}"
            );
        }
        assert_eq!(
            compute_varsha(&mut eph, &i, VarshaYear::Age(MAX_VARSHA_AGE + 1), None),
            Err(CoreError::InvalidDashaInput("age"))
        );
    }

    #[test]
    fn without_a_birth_time_there_is_no_annual_chart_and_no_muntha() {
        let mut eph = eph();
        let known = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), true),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        let r = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), false),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        assert!(!r.time_known);
        assert_eq!((&r.chart, r.facts), (&None, None));
        // Yıl ve mudda dasa yine döner (yaklaşık sayılır; işareti arayüz koyar).
        assert_eq!(
            (r.age, r.start_jd_ut, r.end_jd_ut),
            (3, known.start_jd_ut, known.end_jd_ut)
        );
        assert_eq!(r.mudda, known.mudda);
    }

    #[test]
    fn the_running_year_is_found_from_a_moment() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let sun = compute_chart_for(&mut eph, &i).unwrap().grahas[0].sidereal_lon;
        let fifth = solar_return_jd(&mut eph, &i.settings, sun, JD, 5).unwrap();
        let mut age_at = |jd: f64| {
            compute_varsha(&mut eph, &i, VarshaYear::At(jd), None)
                .unwrap()
                .age
        };
        assert_eq!(age_at(JD - 10.0), 0, "before birth: the birth year");
        assert_eq!(age_at(fifth - 0.5), 4);
        assert_eq!(age_at(fifth), 5);
        assert_eq!(age_at(fifth + 100.0), 5);
        // `At` ile bulunan yılın raporu, aynı yaşın raporudur.
        let by_moment = compute_varsha(&mut eph, &i, VarshaYear::At(fifth + 100.0), None).unwrap();
        let by_age = compute_varsha(&mut eph, &i, VarshaYear::Age(5), None).unwrap();
        assert_eq!(by_moment, by_age);
    }

    #[test]
    fn tropical_rasi_mode_reads_the_muntha_in_the_rasi_frame_and_the_return_sidereally() {
        // Tropikal lagna 70°: İkizler; yıl 3: muntha Başak. Yıllık lagna tropikal 346,87°: Balık;
        // Başak, Balık'tan 7. evdir. Dönüş anı ve doğum nakshatrası sidereal boylamdandır:
        // sidereal kipin raporuyla aynıdır.
        let mut eph = eph();
        let sidereal = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), true),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        let settings = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let r = compute_varsha(&mut eph, &input(settings, true), VarshaYear::Age(3), None).unwrap();
        assert_eq!(r.facts, Some(year_facts(Sign::Gemini, Sign::Pisces, 3)));
        assert_eq!(r.facts.unwrap().muntha.sign, Sign::Virgo);
        assert_eq!(
            (r.start_jd_ut, r.end_jd_ut),
            (sidereal.start_jd_ut, sidereal.end_jd_ut)
        );
        assert_eq!(r.mudda, sidereal.mudda);
    }

    #[test]
    fn number_prashna_counts_the_muntha_from_the_number_lagna_and_low_precision_is_carried() {
        // Sayı 30 → doğum lagnası Yengeç (`number_lagna_lon`); yıl 3: muntha Terazi. Yıllık harita
        // dönüş anının kendi lagnasıyla hesaplanır (Kova): sayı yıllık haritaya taşınmaz.
        let mut eph = eph();
        let numbered = ChartInput {
            lagna_number: Some(30),
            ..input(CalcSettings::default(), true)
        };
        let r = compute_varsha(&mut eph, &numbered, VarshaYear::Age(3), None).unwrap();
        assert_eq!(r.facts, Some(year_facts(Sign::Cancer, Sign::Aquarius, 3)));
        assert_eq!(r.facts.unwrap().muntha.sign, Sign::Libra);
        assert_eq!(
            compute_varsha(
                &mut eph,
                &ChartInput {
                    lagna_number: Some(0),
                    ..numbered
                },
                VarshaYear::Age(3),
                None
            ),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
        // Moshier yedeği: uyarı rapora taşınır.
        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..self::eph()
        };
        let r = compute_varsha(
            &mut moshier,
            &input(CalcSettings::default(), false),
            VarshaYear::Age(1),
            None,
        )
        .unwrap();
        assert_eq!(r.warnings, [ChartWarning::LowPrecision]);
    }
}
