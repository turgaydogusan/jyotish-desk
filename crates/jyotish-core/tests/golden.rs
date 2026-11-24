//! `tests/golden/*.json` referans haritalarını `compute_chart` ile karşılaştırır.
//! Toleranslar spec §8: graha ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″, mahadasha başlangıcı
//! ≤ 1 gün, varga burçları birebir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ephemeris::SwissEph;
use jyotish_core::{
    angular_distance, compute_chart, CalcSettings, Chart, ChartWarning, Graha, Sign, Varga,
};
use serde::Deserialize;

const GRAHA_TOL_ARCSEC: f64 = 2.0;
const AYANAMSA_TOL_ARCSEC: f64 = 1.0;
const LAGNA_TOL_ARCSEC: f64 = 5.0;
const DASHA_START_TOL_DAYS: f64 = 1.0;
/// Bilinen kaynaklar; yeni kaynak eklemek bilinçli bir karar olmalıdır.
const SOURCES: [&str; 2] = ["swetest", "jhora"];
/// Commit'li fixture sayısı bunun altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FIXTURES: usize = 5;
/// `d9_signs` ve `dasha` taşıyan fixture sayısı bunun altına düşerse test düşer.
const MIN_DERIVED_FIXTURES: usize = 5;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    source: String,
    source_version: String,
    #[serde(default)]
    notes: String,
    settings: CalcSettings,
    input: Input,
    expected: Expected,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    jd_ut: f64,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    ayanamsa_deg: f64,
    lagna_lon: f64,
    grahas: BTreeMap<String, f64>,
    /// İsteğe bağlı: "Lagna" ve dokuz graha için D9 burcu. Varsa on anahtar da zorunlu.
    #[serde(default)]
    d9_signs: Option<BTreeMap<String, Sign>>,
    /// İsteğe bağlı: ilk mahadashanın lordu ve teorik başlangıcı (UT JD).
    #[serde(default)]
    dasha: Option<ExpectedDasha>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDasha {
    first_lord: Graha,
    first_start_jd: f64,
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

fn ephe_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe")
}

fn load_fixtures(dir: &Path) -> Vec<(PathBuf, Fixture)> {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            (p, f)
        })
        .collect()
}

/// Tolerans dışındaysa (ya da değer NaN ise) açıklama döner.
fn out_of_tolerance(what: &str, got: f64, want: f64, tol_arcsec: f64) -> Option<String> {
    let diff = angular_distance(got, want) * 3600.0;
    if diff.is_nan() || diff > tol_arcsec {
        Some(format!(
            "{what}: got {got:.7}, want {want:.7}, diff {diff:.3}\" > {tol_arcsec}\""
        ))
    } else {
        None
    }
}

fn compare(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs: Vec<String> = [
        out_of_tolerance(
            "ayanamsa",
            chart.ayanamsa_deg,
            f.expected.ayanamsa_deg,
            AYANAMSA_TOL_ARCSEC,
        ),
        out_of_tolerance(
            "lagna",
            chart.lagna.lon,
            f.expected.lagna_lon,
            LAGNA_TOL_ARCSEC,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    for g in Graha::ALL {
        let name = format!("{g:?}");
        let got = chart
            .grahas
            .iter()
            .find(|p| p.graha == g)
            .map_or(f64::NAN, |p| p.lon);
        match f.expected.grahas.get(&name) {
            Some(&want) => errs.extend(out_of_tolerance(&name, got, want, GRAHA_TOL_ARCSEC)),
            None => errs.push(format!("{name}: missing from fixture")),
        }
    }
    if f.expected.grahas.len() != Graha::ALL.len() {
        errs.push(format!(
            "expected.grahas has {} keys, want exactly {:?}",
            f.expected.grahas.len(),
            Graha::ALL
        ));
    }
    errs
}

/// Plan 06 beklentileri: D9 burçları birebir, ilk mahadasha lordu birebir ve başlangıcı
/// ≤ 1 gün. Alan yoksa denetim yapılmaz (eski ve JHora fixture'ları).
fn compare_derived(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs = Vec::new();
    if let Some(want) = &f.expected.d9_signs {
        match chart.vargas.iter().find(|v| v.varga == Varga::D9) {
            Some(d9) => {
                let mut got: BTreeMap<String, Sign> = d9
                    .placements
                    .iter()
                    .map(|p| (format!("{:?}", p.graha), p.sign))
                    .collect();
                got.insert("Lagna".to_owned(), d9.lagna_sign);
                if &got != want {
                    errs.push(format!("d9_signs: got {got:?}, want {want:?}"));
                }
            }
            None => errs.push("D9 missing from chart.vargas".to_owned()),
        }
    }
    if let Some(want) = &f.expected.dasha {
        match chart.dasha.periods.first() {
            Some(first) => {
                if first.lords != [want.first_lord] {
                    errs.push(format!(
                        "dasha first lord: got {:?}, want {:?}",
                        first.lords, want.first_lord
                    ));
                }
                let diff = (first.start_jd_ut - want.first_start_jd).abs();
                if diff.is_nan() || diff > DASHA_START_TOL_DAYS {
                    errs.push(format!(
                        "dasha first start: got {:.6}, want {:.6}, diff {diff:.3} d > {DASHA_START_TOL_DAYS} d",
                        first.start_jd_ut, want.first_start_jd
                    ));
                }
            }
            None => errs.push("dasha has no periods".to_owned()),
        }
    }
    errs
}

#[test]
fn tolerance_check_flags_out_of_range_nan_and_wraps() {
    let arcsec = 1.0 / 3600.0;
    assert!(out_of_tolerance("x", 10.0, 10.0 + 1.9 * arcsec, 2.0).is_none());
    assert!(out_of_tolerance("x", 10.0, 10.0 + 2.1 * arcsec, 2.0).is_some());
    assert!(out_of_tolerance("x", f64::NAN, 10.0, 2.0).is_some());
    assert!(out_of_tolerance("x", 359.9999, 0.0001, 1.0).is_none()); // 0.72″
}

#[test]
fn golden_charts_match_references() {
    let fixtures = load_fixtures(&golden_dir());
    assert!(
        fixtures.len() >= MIN_FIXTURES,
        "found {} golden fixtures in {}, want >= {MIN_FIXTURES}",
        fixtures.len(),
        golden_dir().display()
    );
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    let mut failures = Vec::new();
    let mut derived = 0;
    let mut sample: Option<(Fixture, Chart)> = None;
    for (path, f) in &fixtures {
        assert_eq!(
            path.file_stem().and_then(|s| s.to_str()),
            Some(f.id.as_str()),
            "file name must equal id"
        );
        assert!(
            SOURCES.contains(&f.source.as_str()),
            "{}: unknown source {:?}",
            f.id,
            f.source
        );
        assert!(
            !f.source_version.is_empty(),
            "{}: empty source_version",
            f.id
        );
        let chart = compute_chart(
            &mut eph,
            f.input.jd_ut,
            f.input.lat,
            f.input.lon,
            &f.settings,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", f.id));
        for e in compare(f, &chart)
            .into_iter()
            .chain(compare_derived(f, &chart))
        {
            failures.push(format!(
                "{} [{} {}; {}]: {e}",
                f.id, f.source, f.source_version, f.notes
            ));
        }
        if f.expected.d9_signs.is_some() && f.expected.dasha.is_some() {
            derived += 1;
            if sample.is_none() {
                sample = Some((f.clone(), chart));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "golden mismatches:\n{}",
        failures.join("\n")
    );
    assert!(
        derived >= MIN_DERIVED_FIXTURES,
        "found {derived} fixtures with d9_signs and dasha, want >= {MIN_DERIVED_FIXTURES}"
    );

    // Karşılaştırıcı gerçekten ısırır: beklentiyi üç yerden boz, üç hata bekle.
    let (mut bad, chart) = sample.expect("a fixture with derived expectations");
    if let Some(d) = bad.expected.dasha.as_mut() {
        d.first_lord = if d.first_lord == Graha::Sun {
            Graha::Moon
        } else {
            Graha::Sun
        };
        d.first_start_jd += 2.0;
    }
    if let Some(s) = bad
        .expected
        .d9_signs
        .as_mut()
        .and_then(|m| m.get_mut("Lagna"))
    {
        *s = Sign::from_index(s.index() + 1);
    }
    let errs = compare_derived(&bad, &chart);
    assert_eq!(errs.len(), 3, "{errs:?}");
}

#[test]
fn out_of_file_range_date_warns_low_precision() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    let settings = CalcSettings::default();
    // 1700-01-01: ephemeris dosyalarının dışında, Moshier'e düşer.
    let old = compute_chart(&mut eph, 2_341_972.5, 41.0, 29.0, &settings).expect("1700 chart");
    assert_eq!(old.meta.warnings, [ChartWarning::LowPrecision]);
    let j2000 = compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &settings).expect("J2000 chart");
    assert!(j2000.meta.warnings.is_empty());
}
