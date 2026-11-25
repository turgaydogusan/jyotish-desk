//! Komut gövdesini gerçek `Engine` aktörüyle uçtan uca dener.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{AyanamsaSystem, Bhavas, CalcSettings, Graha, KaranaName, Sign, Vara, Varga};
use jyotish_desk_lib::commands::{compute_chart_blocking, compute_transits_blocking};

#[test]
fn compute_chart_runs_on_the_engine_thread_and_maps_errors() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
    let engine = Engine::spawn(dir).expect("spawn engine");

    // tests/golden/new-delhi-2000-01-01.json (swetest 2.10.03).
    let chart = compute_chart_blocking(
        &engine,
        2_451_545.0,
        28.6139,
        77.2090,
        CalcSettings::default(),
    )
    .expect("chart");
    assert_eq!(chart.grahas.len(), 9);
    let sun = chart.grahas.iter().find(|p| p.graha == Graha::Sun).unwrap();
    assert!(
        (sun.lon - 256.5156960).abs() * 3600.0 <= 2.0,
        "sun {}",
        sun.lon
    );
    assert!(
        (chart.lagna.lon - 76.3380905).abs() * 3600.0 <= 5.0,
        "lagna {}",
        chart.lagna.lon
    );
    assert_eq!(chart.meta.swe_version, SwissEph::version());
    assert_eq!(chart.meta.settings, CalcSettings::default());
    assert!(chart.meta.warnings.is_empty());

    // Plan 06: vargalar ve dasha (tests/golden/new-delhi-2000-01-01.json türetilmiş beklentileri).
    assert_eq!(chart.vargas.len(), 16);
    let d9 = chart
        .vargas
        .iter()
        .find(|v| v.varga == Varga::D9)
        .expect("D9");
    assert_eq!(d9.lagna_sign, Sign::Aquarius);
    let sun_d9 = d9
        .placements
        .iter()
        .find(|p| p.graha == Graha::Sun)
        .unwrap();
    assert_eq!(sun_d9.sign, Sign::Leo);
    assert_eq!(chart.dasha.periods.len(), 9 + 81 + 729);
    let first = &chart.dasha.periods[0];
    assert_eq!(first.lords, [Graha::Rahu]);
    assert!(
        (first.start_jd_ut - 2_445_231.575_532).abs() <= 1.0,
        "maha start {}",
        first.start_jd_ut
    );

    // Plan 07: panchang ve SAV (tests/golden/new-delhi-2000-01-01.json, swetest türetmesi).
    let p = &chart.panchang;
    let sec = |a: f64, b: f64| (a - b).abs() * 86_400.0;
    assert!(sec(p.sunrise_jd_ut.unwrap(), 2_451_544.574_985) <= 60.0);
    assert!(sec(p.sunset_jd_ut.unwrap(), 2_451_545.000_560) <= 60.0);
    assert_eq!(p.vara, Vara::Saturday);
    assert_eq!(p.tithi.no, 26);
    assert!(sec(p.tithi.ends_jd_ut, 2_451_545.826_235) <= 60.0);
    assert_eq!((p.nakshatra.no, p.yoga.no), (15, 8));
    assert_eq!((p.karana.no, p.karana.name), (51, KaranaName::Bava));
    assert_eq!(
        chart.ashtakavarga.sav,
        [29, 21, 23, 26, 34, 33, 30, 30, 28, 21, 32, 30]
    );

    // Transit: doğum anında haritayla aynı; bir gün sonra Ay ≈ 12–15° ilerler.
    let settings = chart.meta.settings.clone();
    let at_birth =
        compute_transits_blocking(&engine, 2_451_545.0, settings.clone(), chart.bhavas.clone())
            .expect("transits at birth");
    assert_eq!(at_birth.grahas.len(), 9);
    for (t, g) in at_birth.grahas.iter().zip(&chart.grahas) {
        assert_eq!((t.graha, t.sign, t.natal_house), (g.graha, g.sign, g.house));
        assert!((t.lon - g.lon).abs() < 1e-9, "{:?}", t.graha);
    }
    let next =
        compute_transits_blocking(&engine, 2_451_546.0, settings.clone(), chart.bhavas.clone())
            .expect("transits next day");
    let step = (next.grahas[1].lon - at_birth.grahas[1].lon).rem_euclid(360.0);
    assert!((11.0..16.0).contains(&step), "moon step {step}");
    let broken = Bhavas {
        cusps: vec![0.0; 3],
        ..chart.bhavas.clone()
    };
    let err = compute_transits_blocking(&engine, 2_451_545.0, settings, broken).unwrap_err();
    assert_eq!(err.code, "invalid_transit_input");
    assert_eq!(err.params["field"], "natal_bhavas");

    let err = compute_chart_blocking(&engine, 2_451_545.0, 91.0, 0.0, CalcSettings::default())
        .unwrap_err();
    assert_eq!(err.code, "invalid_location");
    assert_eq!(err.params["lat"], "91");

    let old = CalcSettings {
        schema_version: 2,
        ..CalcSettings::default()
    };
    let err = compute_chart_blocking(&engine, 2_451_545.0, 0.0, 0.0, old).unwrap_err();
    assert_eq!(err.code, "unsupported_schema_version");
    assert_eq!(err.params["found"], "2");

    let absurd = CalcSettings {
        ayanamsa: AyanamsaSystem::Custom {
            t0_jd: 1e300,
            deg: 0.0,
        },
        ..CalcSettings::default()
    };
    let err = compute_chart_blocking(&engine, 2_451_545.0, 41.0, 29.0, absurd).unwrap_err();
    assert_eq!(err.code, "invalid_settings");
    assert_eq!(err.params["field"], "ayanamsa");
}
