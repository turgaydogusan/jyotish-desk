//! Komut gövdesini gerçek `Engine` aktörüyle uçtan uca dener.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{CalcSettings, Graha};
use jyotish_desk_lib::commands::compute_chart_blocking;

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
}
