//! Shadbala toplayıcısı (ileri modüller spec'i K3, §7 "Veri modeli ve IPC"): temel haritayı
//! hesaplar, saf hesabın astronomik girdisini (`ShadbalaAstro`) ephemeris'ten toplar ve
//! `shadbala`'ya verir.

use ephemeris::{Body, Ephemeris, HouseSystem, Zodiac};

use crate::chart::{compute_chart_for, finite, Chart, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{following_sunrise, vara_at};
use crate::positions::{ChartInput, Positions};
use crate::zodiac::normalize;
use crate::CoreError;

use super::cheshta::mean_longitudes;
use super::{shadbala, ShadbalaAstro, ShadbalaReport, SHADBALA_GRAHAS};

/// Giriş aramasının sabit adım sayısı (Newton). Güneş'in hızı %3,4'ten az değişir: ilk tahmin
/// en çok ~12° şaşar ve her adım hatanın karesiyle küçülür; sekiz adım fazlasıyla yeter.
const ENTRY_STEPS: usize = 8;

/// Güneş'in `jd_ut`'den önceki son anı ki `zodiac` çerçevesindeki boylamı `target_deg` olsun
/// (burç girişi). `jd_ut` anında Güneş tam hedefteyse o an döner.
pub fn sun_entry_before<E: Ephemeris + ?Sized>(
    eph: &mut E,
    zodiac: Zodiac,
    jd_ut: f64,
    target_deg: f64,
) -> Result<f64, CoreError> {
    let now = eph.body(jd_ut, Body::Sun, zodiac)?;
    let speed = finite(now.lon_speed, "sun_entry")?;
    if speed <= 0.0 {
        return Err(CoreError::NonFiniteResult("sun_entry"));
    }
    // Hedeften bu yana kat edilen yay, [0, 360).
    let travelled = normalize(finite(now.lon, "sun_entry")? - target_deg);
    let mut t = jd_ut - travelled / speed;
    for _ in 0..ENTRY_STEPS {
        let p = eph.body(t, Body::Sun, zodiac)?;
        let speed = finite(p.lon_speed, "sun_entry")?;
        if speed <= 0.0 {
            return Err(CoreError::NonFiniteResult("sun_entry"));
        }
        // Hedefe işaretli uzaklık, [-180, 180).
        let off = normalize(finite(p.lon, "sun_entry")? - target_deg + 180.0) - 180.0;
        t -= off / speed;
    }
    finite(t, "sun_entry")
}

/// Saf hesabın astronomik girdisi. `chart`, `input` ile hesaplanmış temel haritadır. Doğum
/// gününün gün doğumu, batımı ya da sonraki gün doğumu yoksa (kutup günü ya da gecesi)
/// `None`.
pub fn compute_shadbala_astro<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    chart: &Chart,
) -> Result<Option<ShadbalaAstro>, CoreError> {
    let settings = &input.settings;
    let disc = settings.sunrise_def.sun_disc();
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let (Some(sunrise), Some(sunset)) = (chart.panchang.sunrise_jd_ut, chart.panchang.sunset_jd_ut)
    else {
        return Ok(None);
    };
    let Some(next_sunrise) = following_sunrise(eph, sunrise, input.lat, input.lon, disc)? else {
        return Ok(None);
    };
    // MC astronomik açıdır; hangi ev sistemi sorulursa sorulsun aynıdır.
    let houses = eph.houses(
        input.jd_ut,
        input.lat,
        input.lon,
        HouseSystem::WholeSign,
        zodiac,
    )?;
    let mc_lon = finite(houses.mc, "mc")?;

    // Tropikal boylam: sidereal + ayanamsa (v1'in sidereal dönüşümünün tersi).
    let tropical_lons = std::array::from_fn(|i| {
        normalize(chart.grahas[SHADBALA_GRAHAS[i] as usize].sidereal_lon + chart.ayanamsa_deg)
    });

    // Yıl girişi: sidereal 0° (Koç); ay girişi: Güneş'in bulunduğu sidereal burcun başı.
    let sun = chart.grahas[Graha::Sun as usize].sidereal_lon;
    let month_start = (sun / 30.0).floor().clamp(0.0, 11.0) * 30.0;
    let year_entry = sun_entry_before(eph, zodiac, input.jd_ut, 0.0)?;
    let month_entry = sun_entry_before(eph, zodiac, input.jd_ut, month_start)?;
    let year_entry_weekday = vara_at(eph, year_entry, input.lat, input.lon, disc)?;
    let month_entry_weekday = vara_at(eph, month_entry, input.lat, input.lon, disc)?;

    let astro = ShadbalaAstro {
        jd_ut: input.jd_ut,
        sunrise_jd_ut: sunrise,
        sunset_jd_ut: sunset,
        next_sunrise_jd_ut: next_sunrise,
        mc_lon,
        tropical_lons,
        mean: mean_longitudes(input.jd_ut),
        weekday: chart.panchang.vara,
        year_entry_weekday,
        month_entry_weekday,
    };
    Ok(astro.is_consistent().then_some(astro))
}

/// `compute_shadbala` komutunun hesabı. Doğum saati bilinmiyorsa (`input.time_known` yanlış)
/// ya da doğum yerinde o gün gün doğumu ya da batımı yoksa `None` döner; ayar, an ve konum
/// denetimi (`compute_chart`) her durumda yapılır. Temel haritanın konum duyarlılığı uyarısı
/// (`LowPrecision`) rapora taşınır.
pub fn compute_shadbala<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<ShadbalaReport>, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    if !input.time_known {
        return Ok(None);
    }
    let Some(astro) = compute_shadbala_astro(eph, input, &chart)? else {
        return Ok(None);
    };
    let Some(mut report) = shadbala(&Positions::from(&chart), &astro) else {
        return Ok(None);
    };
    report.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::panchang::Vara;
    use crate::settings::{CalcSettings, ZodiacMode};
    use ephemeris::{Precision, SiderealMode};

    const JD: f64 = 2_451_545.0;
    const LAHIRI: Zodiac = Zodiac::Sidereal(SiderealMode::Lahiri);

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

    fn astro_of(eph: &mut FakeEph, input: &ChartInput) -> Option<ShadbalaAstro> {
        let chart = compute_chart_for(eph, input).unwrap();
        compute_shadbala_astro(eph, input, &chart).unwrap()
    }

    #[test]
    fn sun_entry_is_found_on_a_linear_sun() {
        // FakeEph: Güneş tropikal 10°, ayanamsa 24° → sidereal 346°; hız 0,98°/gün.
        let mut eph = FakeEph::default();
        // Ay girişi (Balık, 330°): 16° geride → 16 / 0,98 gün önce.
        let month = sun_entry_before(&mut eph, LAHIRI, JD, 330.0).unwrap();
        assert!((month - (JD - 16.0 / 0.98)).abs() < 1e-9, "{month}");
        // Yıl girişi (Koç, 0°): 346° geride.
        let year = sun_entry_before(&mut eph, LAHIRI, JD, 0.0).unwrap();
        assert!((year - (JD - 346.0 / 0.98)).abs() < 1e-9, "{year}");
        // Giriş anında boylam hedeftedir.
        let at = eph.body(year, Body::Sun, LAHIRI).unwrap().lon;
        assert!(at.min(360.0 - at) < 1e-9, "{at}");
        // Güneş tam hedefteyse an doğumun kendisidir.
        let now = sun_entry_before(&mut eph, LAHIRI, JD, 346.0).unwrap();
        assert!((now - JD).abs() < 1e-9);
        // Geri giden ya da duran bir "Güneş" hata verir (yalnızca bozuk ephemeris'te).
        let mut still = FakeEph::default().with(Body::Sun, 10.0, 0.0);
        assert_eq!(
            sun_entry_before(&mut still, LAHIRI, JD, 0.0),
            Err(CoreError::NonFiniteResult("sun_entry"))
        );
    }

    #[test]
    fn astro_is_collected_from_the_chart_and_the_ephemeris() {
        let mut eph = FakeEph::default();
        let a = astro_of(&mut eph, &input(CalcSettings::default(), true)).expect("astro");
        // FakeEph: doğuş her gün n + 0,67, batış n + 0,15.
        assert_eq!(a.jd_ut, JD);
        assert!((a.sunrise_jd_ut - 2_451_544.67).abs() < 1e-9);
        assert!((a.sunset_jd_ut - 2_451_545.15).abs() < 1e-9);
        assert!((a.next_sunrise_jd_ut - 2_451_545.67).abs() < 1e-9);
        // MC: tropikal lagna 70° + 270° = 340°; sidereal 316°.
        assert!((a.mc_lon - 316.0).abs() < 1e-9);
        // Tropikal boylamlar ephemeris'in tropikal değerleridir (sidereal + ayanamsa).
        let want = [10.0, 95.0, 200.0, 20.0, 250.0, 40.0, 300.0];
        for (got, want) in a.tropical_lons.into_iter().zip(want) {
            assert!((got - want).abs() < 1e-9, "{got} {want}");
        }
        assert_eq!(a.mean, mean_longitudes(JD));
        // JD 2451545,0 (2000-01-01 12:00 UT) Cumartesi'dir; doğuş 2451544,67 aynı yerel günde.
        assert_eq!(a.weekday, Vara::Saturday);
        // Ay girişi JD − 16,3265 = 2451528,6735: o günün doğuşu 2451528,67; 16 gün önce
        // (Cumartesi − 16 gün) Perşembe.
        assert_eq!(a.month_entry_weekday, Vara::Thursday);
        // Yıl girişi JD − 353,0612 = 2451191,9388: doğuş 2451191,67; 353 gün önce
        // (353 = 50 × 7 + 3; Cumartesi − 3 gün) Çarşamba.
        assert_eq!(a.year_entry_weekday, Vara::Wednesday);
        assert!(a.is_consistent());
    }

    #[test]
    fn report_is_the_pure_function_on_the_collected_input() {
        let mut eph = FakeEph::default();
        let i = input(CalcSettings::default(), true);
        let report = compute_shadbala(&mut eph, &i).unwrap().expect("report");
        let chart = compute_chart(&mut eph, i.jd_ut, i.lat, i.lon, &i.settings).unwrap();
        let astro = compute_shadbala_astro(&mut eph, &i, &chart)
            .unwrap()
            .expect("astro");
        assert_eq!(
            Some(&report),
            shadbala(&Positions::from(&chart), &astro).as_ref()
        );
        assert!(report.warnings.is_empty());
        // Cumartesi; doğum doğuştan 0,33 gün sonra: 8. saat dilimi (sıra 7) → Cumartesi
        // dizisinde Satürn. Gündüz 0,48 gün; 0,33 / 0,16 = 2,06 → üçüncü dilim: Satürn.
        let lords = report.time_lords;
        assert_eq!(
            (lords.weekday, lords.hora, lords.tribhaga),
            (Graha::Saturn, Graha::Saturn, Graha::Saturn)
        );
        assert_eq!((lords.year, lords.month), (Graha::Mercury, Graha::Jupiter));
        // Sidereal lagna 46° (Boğa 16°): 1. bhava dört ayaklı, güçsüz bhavası 4. → 30.
        assert_eq!(report.bhavas[0].dig, 30.0);
    }

    #[test]
    fn tropical_rasi_mode_uses_tropical_signs_and_sidereal_angles() {
        let mut eph = FakeEph::default();
        let settings = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let report = compute_shadbala(&mut eph, &input(settings, true))
            .unwrap()
            .expect("report");
        // Güneş rasi çerçevesinde Koç 10°: yücelme noktası, uccha 60.
        assert!((report.grahas[0].sthana.uccha - 60.0).abs() < 1e-9);
        // Lagna rasi çerçevesinde İkizler 10° (tropikal 70°): 1. bhava insan burcu → 60.
        assert_eq!(report.bhavas[0].dig, 60.0);
        assert_eq!(report.bhavas[0].lord, Graha::Mercury);
        // Açı farkları çerçeveden bağımsızdır: dig bala sidereal kipteki ile aynıdır.
        let sidereal = compute_shadbala(&mut eph, &input(CalcSettings::default(), true))
            .unwrap()
            .expect("report");
        for (a, b) in report.grahas.iter().zip(&sidereal.grahas) {
            assert!((a.dig - b.dig).abs() < 1e-9, "{:?}", a.graha);
            assert!((a.drik - b.drik).abs() < 1e-9, "{:?}", a.graha);
            assert!((a.cheshta - b.cheshta).abs() < 1e-9, "{:?}", a.graha);
            assert_eq!(a.temporal, b.temporal, "{:?}", a.graha);
        }
    }

    #[test]
    fn no_report_without_a_birth_time_or_a_sunrise_but_errors_are_still_reported() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        assert_eq!(
            compute_shadbala(&mut eph, &input(s.clone(), false)),
            Ok(None)
        );
        // Kutup günü ya da gecesi: doğuş ve batış yok.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        assert_eq!(
            compute_shadbala(&mut polar, &input(s.clone(), true)),
            Ok(None)
        );
        // Doğum gününün doğuşu var, sonraki doğuş arama ufkunda yok.
        let mut last = FakeEph {
            sunrises: Some(vec![2_451_544.67]),
            ..FakeEph::default()
        };
        assert_eq!(astro_of(&mut last, &input(s.clone(), true)), None);
        assert_eq!(
            compute_shadbala(&mut last, &input(s.clone(), true)),
            Ok(None)
        );
        // Ayar, an ve konum denetimi saat bilinmese de yapılır.
        let mut bad = input(s.clone(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_shadbala(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..s
        };
        assert_eq!(
            compute_shadbala(&mut eph, &input(old, false)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }

    #[test]
    fn only_the_low_precision_warning_is_carried() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            porphyry_fallback: true,
            ..FakeEph::default()
        };
        let settings = CalcSettings {
            house_system: crate::settings::HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let report = compute_shadbala(&mut eph, &input(settings, true))
            .unwrap()
            .expect("report");
        assert_eq!(report.warnings, [ChartWarning::LowPrecision]);
    }
}
