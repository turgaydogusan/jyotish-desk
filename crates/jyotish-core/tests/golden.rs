//! `tests/golden/*.json` referans haritalarını `compute_chart` ile karşılaştırır.
//! Toleranslar spec §8: graha ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″, mahadasha başlangıcı
//! ≤ 1 gün, varga burçları birebir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ephemeris::SwissEph;
use jyotish_core::{
    angular_distance, compute_chart, CalcSettings, Chart, ChartWarning, Graha, Sign, Vara, Varga,
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
/// Panchang bitiş zamanları ve gün doğumu/batımı toleransı (spec §8).
const PANCHANG_TOL_SECONDS: f64 = 60.0;
/// `panchang` ve `sav` taşıyan fixture sayısı bunun altına düşerse test düşer.
const MIN_PANCHANG_FIXTURES: usize = 5;

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
    /// İsteğe bağlı: doğum anının panchangı (Plan 07). Zamanlar UT JD; gün doğumu yoksa null.
    #[serde(default)]
    panchang: Option<ExpectedPanchang>,
    /// İsteğe bağlı: Koç'tan Balık'a sarvashtakavarga.
    #[serde(default)]
    sav: Option<[u8; 12]>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDasha {
    first_lord: Graha,
    first_start_jd: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedPanchang {
    sunrise_jd: Option<f64>,
    sunset_jd: Option<f64>,
    vara: Vara,
    tithi: u8,
    tithi_ends_jd: f64,
    nakshatra: u8,
    nakshatra_ends_jd: f64,
    yoga: u8,
    yoga_ends_jd: f64,
    karana: u8,
    karana_ends_jd: f64,
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

/// İki zaman (UT JD) tolerans dışındaysa, biri yoksa ya da NaN ise açıklama döner.
fn time_mismatch(what: &str, got: Option<f64>, want: Option<f64>) -> Option<String> {
    match (got, want) {
        (None, None) => None,
        (Some(g), Some(w)) => {
            let diff = (g - w).abs() * 86_400.0;
            if diff.is_nan() || diff > PANCHANG_TOL_SECONDS {
                Some(format!(
                    "{what}: got {g:.6}, want {w:.6}, diff {diff:.1} s > {PANCHANG_TOL_SECONDS} s"
                ))
            } else {
                None
            }
        }
        _ => Some(format!("{what}: got {got:?}, want {want:?}")),
    }
}

/// Plan 07 beklentileri: panchang numaraları ve vara birebir, zamanlar ≤ 60 sn; SAV birebir.
fn compare_panchang(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs = Vec::new();
    if let Some(want) = &f.expected.panchang {
        let p = &chart.panchang;
        errs.extend(time_mismatch("sunrise", p.sunrise_jd_ut, want.sunrise_jd));
        errs.extend(time_mismatch("sunset", p.sunset_jd_ut, want.sunset_jd));
        if p.vara != want.vara {
            errs.push(format!("vara: got {:?}, want {:?}", p.vara, want.vara));
        }
        let elements = [
            (
                "tithi",
                p.tithi.no,
                want.tithi,
                p.tithi.ends_jd_ut,
                want.tithi_ends_jd,
            ),
            (
                "nakshatra",
                p.nakshatra.no,
                want.nakshatra,
                p.nakshatra.ends_jd_ut,
                want.nakshatra_ends_jd,
            ),
            (
                "yoga",
                p.yoga.no,
                want.yoga,
                p.yoga.ends_jd_ut,
                want.yoga_ends_jd,
            ),
            (
                "karana",
                p.karana.no,
                want.karana,
                p.karana.ends_jd_ut,
                want.karana_ends_jd,
            ),
        ];
        for (what, got_no, want_no, got_end, want_end) in elements {
            if got_no != want_no {
                errs.push(format!("{what}: got {got_no}, want {want_no}"));
            }
            errs.extend(time_mismatch(
                &format!("{what} end"),
                Some(got_end),
                Some(want_end),
            ));
        }
    }
    if let Some(want) = &f.expected.sav {
        if &chart.ashtakavarga.sav != want {
            errs.push(format!(
                "sav: got {:?}, want {want:?}",
                chart.ashtakavarga.sav
            ));
        }
    }
    errs
}

#[test]
fn time_tolerance_flags_out_of_range_nan_and_missing() {
    let second = 1.0 / 86_400.0;
    let jd = 2_451_545.0;
    assert!(time_mismatch("t", Some(jd), Some(jd + 59.0 * second)).is_none());
    assert!(time_mismatch("t", Some(jd), Some(jd + 61.0 * second)).is_some());
    assert!(time_mismatch("t", Some(f64::NAN), Some(jd)).is_some());
    assert!(time_mismatch("t", None, None).is_none());
    assert!(time_mismatch("t", None, Some(jd)).is_some());
    assert!(time_mismatch("t", Some(jd), None).is_some());
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
    let mut with_panchang = 0;
    let mut panchang_sample: Option<(Fixture, Chart)> = None;
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
            .chain(compare_panchang(f, &chart))
        {
            failures.push(format!(
                "{} [{} {}; {}]: {e}",
                f.id, f.source, f.source_version, f.notes
            ));
        }
        if let (Some(p), Some(_)) = (&f.expected.panchang, &f.expected.sav) {
            with_panchang += 1;
            if panchang_sample.is_none() && p.sunrise_jd.is_some() {
                panchang_sample = Some((f.clone(), chart.clone()));
            }
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
    assert!(
        with_panchang >= MIN_PANCHANG_FIXTURES,
        "found {with_panchang} fixtures with panchang and sav, want >= {MIN_PANCHANG_FIXTURES}"
    );

    // Panchang karşılaştırıcısı da ısırır: tithi, gün doğumu ve SAV'ı boz, üç hata bekle.
    let (mut bad, chart) = panchang_sample.expect("a fixture with panchang and a sunrise");
    if let Some(p) = bad.expected.panchang.as_mut() {
        p.tithi = p.tithi % 30 + 1;
        p.sunrise_jd = p.sunrise_jd.map(|t| t + 0.01); // 864 sn
    }
    if let Some(s) = bad.expected.sav.as_mut() {
        s[0] += 1;
    }
    let errs = compare_panchang(&bad, &chart);
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

#[test]
fn first_sunrise_after_the_midnight_sun_is_found() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    // 67.179 K, 90.14 B; 5 Temmuz 2000 07:36 UT (yerel ortalama 01:36, Çarşamba). Gece yarısı
    // güneşinden sonraki ilk doğuş doğumdan 1.2 saat önce (yerel 00:24); bir önceki doğuş
    // haftalar öncedir. SE'nin yüksek enlem araması başlangıçtan ~26 saat ileriye bakar.
    let p = jyotish_core::compute_panchang(
        &mut eph,
        2_451_730.817,
        67.179,
        -90.14,
        &CalcSettings::default(),
    )
    .expect("panchang");
    let rise = p.sunrise_jd_ut.expect("sunrise");
    assert!(
        (rise - 2_451_730.766_847).abs() * 86_400.0 <= 60.0,
        "{rise}"
    );
    assert_eq!(p.vara, Vara::Wednesday);
}

#[test]
fn last_sunrise_before_the_polar_night_is_found() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    // 80.07 K, 163.64 D; 19 Ekim 1970 03:21:36 UT (yerel ortalama 14:16, Pazartesi). Kutup
    // gecesinden hemen önce gündüz 1.4 saat sürer; doğuşlar 17 Ekim 23:33:20.5 UT ve
    // 19 Ekim 00:06:35.2 UT, araları 24.55 saat. Gündüz SE'nin 2 saatlik örnekleri arasına
    // sığar: ilk doğuştan başlayan arama ikinciyi görmez ve vara Pazar çıkar.
    // target/swetest -rise -hindu -bj2440877.0 -n2 -p0 -geopos163.64,80.07,0 -head -eswe -edirdata/ephe
    let p = jyotish_core::compute_panchang(
        &mut eph,
        2_440_878.64,
        80.07,
        163.64,
        &CalcSettings::default(),
    )
    .expect("panchang");
    let rise = p.sunrise_jd_ut.expect("sunrise");
    assert!(
        (rise - 2_440_878.504_574).abs() * 86_400.0 <= 60.0,
        "{rise}"
    );
    let set = p.sunset_jd_ut.expect("sunset");
    assert!((set - 2_440_878.563_773).abs() * 86_400.0 <= 60.0, "{set}");
    assert_eq!(p.vara, Vara::Monday);
}

#[test]
fn sunrise_minutes_before_birth_is_exact_at_sixty_degrees() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    // Helsinki 60.1699 K, 24.9384 D; 25 Aralık 1990 07:40:10 UT, gün doğumundan 4 dk sonra.
    // SE'nin hızlı yöntemi doğumdan 24 saat önce başlayan aramada bu doğuşu 301 sn erken
    // verir (2448250.813293); ephemeris katmanı 50°'nin üstünde tam aramayı zorlar.
    // target/swetest -rise -hindu -bj2448249.9 -n2 -p0 -geopos24.9384,60.1699,0 -head -eswe -edirdata/ephe
    // rise 25.12.1990 07:36:09.9
    let p = jyotish_core::compute_panchang(
        &mut eph,
        2_448_250.819_559,
        60.1699,
        24.9384,
        &CalcSettings::default(),
    )
    .expect("panchang");
    let rise = p.sunrise_jd_ut.expect("sunrise");
    assert!(
        (rise - 2_448_250.816_781).abs() * 86_400.0 <= 60.0,
        "{rise}"
    );
    assert_eq!(p.vara, Vara::Tuesday);
}

#[test]
fn birth_two_minutes_before_sunrise_keeps_the_previous_vara() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    // Reykjavik 64.1466 K, 21.9426 B; 19 Kasım 1990 10:18:15 UT, o günün doğuşundan
    // (10:20:14.6 UT) 2 dk önce: gün hâlâ 18 Kasım Pazar'dır. SE'nin hızlı yöntemi 19 Kasım
    // doğuşunu 132 sn erken (2448214.929193, doğumdan önce) verir ve vara Pazartesi çıkar.
    // target/swetest -rise -hindu -bj2448213.5 -n2 -p0 -geopos-21.9426,64.1466,0 -head -eswe -edirdata/ephe
    // rise 18.11.1990 10:16:50.9   rise 19.11.1990 10:20:14.6
    let p = jyotish_core::compute_panchang(
        &mut eph,
        2_448_214.929_336,
        64.1466,
        -21.9426,
        &CalcSettings::default(),
    )
    .expect("panchang");
    let rise = p.sunrise_jd_ut.expect("sunrise");
    assert!(
        (rise - 2_448_213.928_367).abs() * 86_400.0 <= 60.0,
        "{rise}"
    );
    assert_eq!(p.vara, Vara::Sunday);
}
