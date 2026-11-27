//! Avastha komut gövdesi, gerçek `Engine` ile uçtan uca (yol haritası §4.4; ileri modüller
//! spec'i §13 "Komut katmanı", §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Ayrı dosyadır: `tests/modules.rs` paralel planlarla paylaşılmaz.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::Engine;
use jyotish_core::avastha::{
    avasthas, elapsed_ghatis, shayanadi_state, Lajjitadi, AVASTHA_GRAHAS, AVASTHA_METHOD,
};
use jyotish_core::{CalcSettings, ChartInput, ChartWarning, HouseSystem, Positions};
use jyotish_desk_lib::avastha_commands::compute_avasthas_blocking;
use jyotish_desk_lib::commands::compute_chart_blocking;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `tests/golden/*.json` fixture'larının anları (ad, girdi).
fn golden_inputs() -> Vec<(String, ChartInput)> {
    let dir = root().join("../../tests/golden");
    let mut out: Vec<(String, ChartInput)> = std::fs::read_dir(&dir)
        .expect("golden dir")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let v: Value =
                serde_json::from_str(&std::fs::read_to_string(&p).expect("read fixture"))
                    .expect("parse fixture");
            let num = |k: &str| v["input"][k].as_f64().expect("number");
            let input = ChartInput {
                jd_ut: num("jd_ut"),
                lat: num("lat"),
                lon: num("lon"),
                settings: serde_json::from_value(v["settings"].clone()).expect("settings"),
                time_known: true,
                lagna_number: None,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn median_of_15(mut run: impl FnMut()) -> (Duration, Duration) {
    run();
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            run();
            t.elapsed()
        })
        .collect();
    times.sort();
    (times[times.len() / 2], times[times.len() - 1])
}

#[test]
fn avastha_command_runs_on_the_engine_thread_and_maps_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
    // Aşağıdaki kollar boş geçmemelidir: golden anlarda en az bir Shayanadi'li ve en az bir
    // gün doğumsuz harita, en az bir Lajjita dışı durum bulunur.
    let (mut with_shayanadi, mut without_sunrise, mut states) = (0, 0, 0);
    for (id, input) in &inputs {
        let report = compute_avasthas_blocking(&engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(report.method, AVASTHA_METHOD, "{id}");
        assert!(report.time_known, "{id}");
        let order: Vec<_> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, AVASTHA_GRAHAS, "{id}");

        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından ve
        // panchangının gün doğumundan aynı rapor çıkar.
        let chart = compute_chart_blocking(
            &engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let positions = Positions::from(&chart);
        let ghatis = chart
            .panchang
            .sunrise_jd_ut
            .and_then(|rise| elapsed_ghatis(input.jd_ut, rise));
        let mut want = avasthas(&positions, ghatis);
        want.warnings = chart
            .meta
            .warnings
            .iter()
            .copied()
            .filter(|w| *w == ChartWarning::LowPrecision)
            .collect();
        assert_eq!(report, want, "{id}");

        // Değişmezler gerçek haritalarda da sağlanır.
        match report.shayanadi_basis {
            Some(basis) => {
                with_shayanadi += 1;
                assert!((1..=27).contains(&basis.birth_nakshatra), "{id}");
                assert!((1..=12).contains(&basis.lagna_sign_number), "{id}");
                // Doğum, son gün doğumundan en çok bir gün ve çeyrek sonradır (v1 penceresi).
                assert!(basis.ghatis <= 75, "{id}: {} ghatis", basis.ghatis);
                assert_eq!(
                    basis.birth_nakshatra, chart.grahas[1].nakshatra,
                    "{id}: the Moon's nakshatra is the chart's"
                );
                assert_eq!(
                    usize::from(basis.lagna_sign_number),
                    chart.lagna.sign.index() + 1,
                    "{id}"
                );
                for row in &report.grahas {
                    let hit = row.shayanadi.expect("a state for each graha");
                    assert_eq!(hit.state, shayanadi_state(hit.total), "{id}");
                    assert_eq!(
                        hit.nakshatra, chart.grahas[row.graha as usize].nakshatra,
                        "{id}: {:?}",
                        row.graha
                    );
                    assert!((1..=9).contains(&hit.navamsa), "{id}");
                }
            }
            None => {
                without_sunrise += 1;
                assert!(
                    chart.panchang.sunrise_jd_ut.is_none(),
                    "{id}: no Shayanadi only without a sunrise"
                );
                assert!(chart
                    .meta
                    .warnings
                    .contains(&ChartWarning::SunriseUnavailable));
                assert!(report.grahas.iter().all(|g| g.shayanadi.is_none()), "{id}");
            }
        }
        for row in &report.grahas {
            let listed: Vec<_> = row.lajjitadi.iter().map(|h| h.state).collect();
            let mut sorted = listed.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(listed, sorted, "{id}: {:?} rule order", row.graha);
            states += listed.len();
            for h in &row.lajjitadi {
                assert!(!h.grahas.contains(&row.graha), "{id}: {:?}", row.graha);
            }
        }

        // Evler burç evidir (K4): ev sistemi ayarı raporu değiştirmez.
        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
            let other = ChartInput {
                settings: CalcSettings {
                    house_system,
                    ..input.settings.clone()
                },
                ..input.clone()
            };
            assert_eq!(
                compute_avasthas_blocking(&engine, other).expect("house system"),
                report,
                "{id}: {house_system:?}"
            );
        }

        // Doğum saati bilinmiyorsa lagna yoktur: Lajjita değerlendirilmez, Shayanadi yoktur;
        // öteki dizi ve durumlar aynıdır.
        let unknown = compute_avasthas_blocking(
            &engine,
            ChartInput {
                time_known: false,
                lagna_number: None,
                ..input.clone()
            },
        )
        .expect("time unknown");
        assert_eq!(
            unknown,
            {
                let mut w = avasthas(&positions.clone().without_lagna(), None);
                w.warnings.clone_from(&report.warnings);
                w
            },
            "{id}"
        );
        assert!(
            !unknown.time_known && unknown.shayanadi_basis.is_none(),
            "{id}"
        );
        for (a, b) in unknown.grahas.iter().zip(&report.grahas) {
            assert_eq!((a.baladi, a.jagradadi), (b.baladi, b.jagradadi), "{id}");
            assert!(a.shayanadi.is_none(), "{id}");
            let rest: Vec<_> = b
                .lajjitadi
                .iter()
                .filter(|h| h.state != Lajjitadi::Lajjita)
                .cloned()
                .collect();
            assert_eq!(a.lajjitadi, rest, "{id}: {:?}", a.graha);
        }
    }
    assert!(
        with_shayanadi >= 3,
        "{with_shayanadi} charts with a sunrise"
    );
    assert!(without_sunrise >= 1, "no polar chart in the golden moments");
    assert!(
        states >= 10,
        "{states} lajjitadi states in the golden moments"
    );

    // Hatalar `compute_chart` ile aynı kodlara eşlenir; saat bilinmese de denetlenir.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        time_known: false,
        lagna_number: None,
        ..delhi.clone()
    };
    assert_eq!(
        compute_avasthas_blocking(&engine, bad_place)
            .unwrap_err()
            .code,
        "invalid_location"
    );
    let old_settings = ChartInput {
        settings: CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        },
        ..delhi.clone()
    };
    assert_eq!(
        compute_avasthas_blocking(&engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );
    let bad_number = ChartInput {
        lagna_number: Some(0),
        ..delhi.clone()
    };
    assert_eq!(
        compute_avasthas_blocking(&engine, bad_number)
            .unwrap_err()
            .code,
        "invalid_prashna_input"
    );
    // Sayı prashnası: lagna sayıdan gelir (30 → Yengeç, 4); öteki terimler aynıdır.
    let numbered = compute_avasthas_blocking(
        &engine,
        ChartInput {
            lagna_number: Some(30),
            ..delhi.clone()
        },
    )
    .expect("number prashna");
    let plain = compute_avasthas_blocking(&engine, delhi.clone()).expect("delhi");
    let (n, p) = (
        numbered.shayanadi_basis.expect("basis"),
        plain.shayanadi_basis.expect("basis"),
    );
    assert_eq!(n.lagna_sign_number, 4);
    assert_eq!((n.birth_nakshatra, n.ghatis), (p.birth_nakshatra, p.ghatis));

    // Performans: temel harita hesabı dahil ≤ 10 ms (yogalarla aynı bütçe; spec §15).
    let (median, max) = median_of_15(|| {
        compute_avasthas_blocking(&engine, delhi.clone()).expect("avasthas");
    });
    eprintln!("compute_avasthas median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "compute_avasthas median {median:?}"
    );
}
