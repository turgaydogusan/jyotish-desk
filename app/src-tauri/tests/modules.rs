//! İleri modül komut gövdeleri, gerçek `Engine` ile uçtan uca (ileri modüller spec'i §13
//! "Komut katmanı"; §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac};
use jyotish_core::kuta::{gana_of, nadi_of, varna_of, yoni_of};
use jyotish_core::muhurta::{is_panchaka, is_rikta, is_vishti};
use jyotish_core::timeline::{
    graha_motion_series, graha_nakshatra_series, graha_sign_series, karana_series, lagna_sign_at,
    lagna_sign_series, nakshatra_series, nitya_yoga_series, tithi_series, vara_series, Motion,
    NakshatraPada, STATION_GRAHAS,
};
use jyotish_core::{
    chara_dasha, cheshta_from, co_lords, compute_panchang, compute_shadbala_astro,
    graha_aspect_signs, house_lord, house_sign, jaimini_report, kuja_dosha, kuta_report,
    mean_longitudes, rashi_drishti_signs, shadbala, sign_dasha_children, sign_dasha_path_at,
    sphuta_drishti, sun_entry_before, vara_lord, window_snapshot, yoga_rule, yogas,
    AdvancedSettings, CalcSettings, Chart, ChartInput, ChartWarning, Dignity, Graha, HouseSystem,
    KarakaRole, KarakaScheme, Kuta, KutaRole, MoonDayChange, MuhurtaCriteria, MuhurtaFlag,
    NatalMoon, NodeKind, Positions, Sign, SignDashaPeriod, Vara, Varga, YogaGroup,
    ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD,
    MAX_SUNRISE_GAP_DAYS, MUHURTA_METHOD, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD,
    YOGA_CATALOG, YOGA_CATALOG_VERSION,
};
use jyotish_core::{number_lagna_lon, prashna_facts, rising_mode, ZodiacMode, PRASHNA_METHOD};
use jyotish_desk_lib::commands::{compute_chart_blocking, compute_chart_for_blocking};
use jyotish_desk_lib::module_commands::{
    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
    compute_compat_blocking, compute_drishti_blocking, compute_jaimini_blocking,
    compute_shadbala_blocking, compute_yogas_blocking,
};
use jyotish_desk_lib::muhurta_commands::{
    day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery, MuhurtaSearchView,
    MuhurtaWindowView,
};
use jyotish_desk_lib::prashna_commands::compute_prashna_facts_blocking;
use jyotish_desk_lib::transit_commands::TransitAt;
use serde_json::Value;
use store::TzChoice;

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

/// Motoru tek test açar; modül başına denetimler ayrı fonksiyonlardadır.
#[test]
fn module_commands_run_on_the_engine_thread_and_map_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
    drishti_checks(&engine, &inputs);
    jaimini_checks(&engine, &inputs);
    chara_dasha_checks(&engine, &inputs);
    shadbala_checks(&engine, &inputs);
    yogas_checks(&engine, &inputs);
    compat_checks(&engine, &inputs);
    timeline_checks(&engine, &inputs);
    muhurta_checks(&engine, &inputs);
    prashna_checks(&engine, &inputs);
}

fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    for (id, input) in inputs {
        let table = compute_drishti_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(table.method, DRISHTI_METHOD, "{id}");
        assert_eq!(table.sign_aspects.len(), 13, "{id}");
        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından
        // aynı tablo çıkar.
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let pos = |g: Graha| &chart.grahas[g as usize];
        for a in &table.sign_aspects {
            assert!(
                graha_aspect_signs(a.graha, pos(a.graha).sign).contains(&a.target),
                "{id}: {:?} → {:?}",
                a.graha,
                a.target
            );
            let occupants: Vec<Graha> = chart
                .grahas
                .iter()
                .filter(|p| p.sign == a.target)
                .map(|p| p.graha)
                .collect();
            assert_eq!(a.occupants, occupants, "{id}: occupants of {:?}", a.target);
        }
        for (i, from) in ASPECTING_GRAHAS.into_iter().enumerate() {
            for to in Graha::ALL {
                let want = if to == from {
                    0.0
                } else {
                    sphuta_drishti(from, pos(from).sidereal_lon, pos(to).sidereal_lon)
                };
                let got = table.sphuta[i][to as usize];
                assert!((0.0..=60.0).contains(&got), "{id}: {got}");
                assert_eq!(got, want, "{id}: {from:?} → {to:?}");
            }
        }
        // Bakış tablosu lagnaya bağlı değildir: saat bilinmese de aynıdır.
        let unknown = ChartInput {
            time_known: false,
            lagna_number: None,
            ..input.clone()
        };
        assert_eq!(
            compute_drishti_blocking(engine, unknown).expect("time unknown"),
            table,
            "{id}"
        );
    }

    // Hatalar `compute_chart` ile aynı kodlara eşlenir; parametreler danışan verisi taşımaz.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        ..delhi.clone()
    };
    assert_eq!(
        compute_drishti_blocking(engine, bad_place)
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
    let err = compute_drishti_blocking(engine, old_settings).unwrap_err();
    assert_eq!(err.code, "unsupported_schema_version");
    assert_eq!(err.params["found"], "1");
    let bad_moment = ChartInput {
        jd_ut: f64::NAN,
        ..delhi.clone()
    };
    assert_eq!(
        compute_drishti_blocking(engine, bad_moment)
            .unwrap_err()
            .code,
        "invalid_julian_day"
    );

    // Performans: spec §15 "compute_drishti ≤ 10 ms (temel harita hesabı dahil)". Ortanca,
    // ısınmadan sonra (open_client ölçümüyle aynı düzenek).
    compute_drishti_blocking(engine, delhi.clone()).expect("warm-up");
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            compute_drishti_blocking(engine, delhi.clone()).expect("drishti");
            t.elapsed()
        })
        .collect();
    times.sort();
    let median = times[times.len() / 2];
    eprintln!(
        "compute_drishti median {median:?}, max {:?}",
        times[times.len() - 1]
    );
    assert!(
        median <= Duration::from_millis(10),
        "compute_drishti median {median:?}"
    );
}

/// Isınmadan sonra on beş koşunun ortancası ve en büyüğü (`open_client` ölçümüyle aynı düzenek).
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

fn jaimini_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    for (id, input) in inputs {
        let report = compute_jaimini_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(report.method, JAIMINI_METHOD, "{id}");
        assert_eq!(
            (report.scheme, report.co_lords),
            (KarakaScheme::Eight, true),
            "{id}: fixture settings are the defaults"
        );
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından
        // aynı rapor çıkar.
        let positions = Positions::from(&chart);
        let mut want = jaimini_report(&positions, &input.settings.advanced);
        // Rapor temel haritanın yalnızca `LowPrecision` uyarısını taşır (ev sistemi ve gün
        // doğumu uyarıları bu modülü ilgilendirmez).
        want.warnings = chart
            .meta
            .warnings
            .iter()
            .copied()
            .filter(|w| *w == ChartWarning::LowPrecision)
            .collect();
        assert_eq!(report, want, "{id}");

        // Karaka sırası sekiz grahanın permütasyonudur; Ketu yoktur.
        let mut grahas: Vec<Graha> = report.karakas.iter().map(|k| k.graha).collect();
        grahas.sort();
        assert_eq!(grahas[..], Graha::ALL[..8], "{id}");
        for w in report.karakas.windows(2) {
            assert!(w[0].effective_deg >= w[1].effective_deg, "{id}");
        }
        // Karakamsha haritanın D9'undaki Atmakaraka burcudur; swamsha evleri D9'dan sayılır.
        let d9 = chart
            .vargas
            .iter()
            .find(|v| v.varga == Varga::D9)
            .expect("D9");
        let d9_sign = |g: Graha| d9.placements[g as usize].sign;
        let ak = report.karakamsha.atmakaraka;
        assert_eq!(ak, report.karakas[0].graha, "{id}");
        assert_eq!(report.karakamsha.sign, d9_sign(ak), "{id}");
        for g in Graha::ALL {
            let i = g as usize;
            assert_eq!(
                report.karakamsha.houses_in_d9[i],
                d9_sign(g).house_from(report.karakamsha.sign),
                "{id}: swamsha house of {g:?}"
            );
            assert_eq!(
                report.karakamsha.houses_in_d1[i],
                chart.grahas[i].sign.house_from(report.karakamsha.sign),
                "{id}: D1 house of {g:?}"
            );
        }
        // Rashi drishti simetriktir ve her burç tam üç burca bakar.
        for from in Sign::ALL {
            let row = report.rashi_drishti[from.index()];
            assert_eq!(row.iter().filter(|&&b| b).count(), 3, "{id}: {from:?}");
            for to in Sign::ALL {
                assert_eq!(
                    row[to.index()],
                    report.rashi_drishti[to.index()][from.index()],
                    "{id}"
                );
            }
        }
        assert_eq!(report.graha_rashi_drishti.len(), 27, "{id}");
        for a in &report.graha_rashi_drishti {
            let from = chart.grahas[a.graha as usize].sign;
            assert!(rashi_drishti_signs(from).contains(&a.target), "{id}");
        }
        // Hiçbir pada evin kendi burcuna ya da 7. burcuna düşmez; lord evin lordudur ya da
        // (Akrep ve Kova'da) eş lordu.
        let lagna = chart.lagna.sign;
        let padas = report.arudhas.expect("padas with a known birth time");
        for pada in &padas {
            let sign = house_sign(lagna, pada.house);
            assert!(
                !matches!(pada.sign.house_from(sign), 1 | 7),
                "{id}: A{}",
                pada.house
            );
            let allowed = match co_lords(sign) {
                Some((planet, node)) => vec![planet, node],
                None => vec![house_lord(lagna, pada.house)],
            };
            assert!(allowed.contains(&pada.lord), "{id}: A{} lord", pada.house);
        }
        // Argala lagna burcundan sıralanır; aynı graha bir çiftin iki tarafında görünmez.
        assert_eq!(report.argala[0].reference, lagna, "{id}");
        for row in &report.argala {
            for pair in &row.pairs {
                assert!(
                    pair.argala_grahas
                        .iter()
                        .all(|g| !pair.virodha_grahas.contains(g)),
                    "{id}"
                );
            }
        }

        // Saat bilinmiyorsa padalar Rust'ta boş döner (K2) ve argala Koç'tan sıralanır.
        let unknown = compute_jaimini_blocking(
            engine,
            ChartInput {
                time_known: false,
                lagna_number: None,
                ..input.clone()
            },
        )
        .expect("time unknown");
        assert_eq!(unknown.arudhas, None, "{id}");
        assert_eq!(unknown.argala[0].reference, Sign::Aries, "{id}");
        assert_eq!(unknown.karakas, report.karakas, "{id}");
        assert_eq!(unknown.karakamsha, report.karakamsha, "{id}");

        // İki ayar hesapta kullanılır: yedili şemada Rahu sıralanmaz; eş lordlar devre
        // dışıyken Akrep ve Kova evlerinin lordu Mars ve Satürn'dür.
        let mut plain = input.clone();
        plain.settings.advanced = AdvancedSettings {
            karaka_scheme: KarakaScheme::Seven,
            jaimini_co_lords: false,
        };
        let plain = compute_jaimini_blocking(engine, plain).expect("seven scheme");
        assert_eq!((plain.scheme, plain.co_lords), (KarakaScheme::Seven, false));
        assert_eq!(plain.karakas.len(), 7, "{id}");
        assert!(plain.karakas.iter().all(|k| !k.graha.is_node()), "{id}");
        // Yedili sıra, sekizli sıranın Rahu'suz hâlidir (Rahu dışında etkin derece
        // aynıdır ve sıralama kararlıdır); roller yedili listenin sırasındadır.
        let without_rahu: Vec<Graha> = report
            .karakas
            .iter()
            .map(|k| k.graha)
            .filter(|g| *g != Graha::Rahu)
            .collect();
        let seven: Vec<Graha> = plain.karakas.iter().map(|k| k.graha).collect();
        assert_eq!(seven, without_rahu, "{id}");
        let roles: Vec<KarakaRole> = plain.karakas.iter().map(|k| k.role).collect();
        assert_eq!(roles[..], SEVEN_KARAKA_ROLES[..], "{id}");
        assert_eq!(plain.karakamsha.atmakaraka, seven[0], "{id}");
        for pada in plain.arudhas.expect("padas") {
            assert_eq!(pada.lord, house_lord(lagna, pada.house), "{id}");
        }
    }

    // Hatalar `compute_chart` ile aynı kodlara eşlenir.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        ..delhi.clone()
    };
    assert_eq!(
        compute_jaimini_blocking(engine, bad_place)
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
        compute_jaimini_blocking(engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );
    let bad_moment = ChartInput {
        jd_ut: f64::NAN,
        ..delhi.clone()
    };
    assert_eq!(
        compute_jaimini_blocking(engine, bad_moment)
            .unwrap_err()
            .code,
        "invalid_julian_day"
    );

    // Performans: spec §15 "compute_jaimini ≤ 10 ms (temel harita hesabı dahil)".
    let (median, max) = median_of_15(|| {
        compute_jaimini_blocking(engine, delhi.clone()).expect("jaimini");
    });
    eprintln!("compute_jaimini median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "compute_jaimini median {median:?}"
    );
}

fn chara_dasha_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    for (id, input) in inputs {
        let dasha = compute_chara_dasha_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code))
            .unwrap_or_else(|| panic!("{id}: a dasha with a known birth time"));
        assert_eq!(dasha.method, CHARA_DASHA_METHOD, "{id}");
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından aynı
        // dasha çıkar; ilk dasha doğum anında, haritanın lagna burcuyla başlar.
        let positions = Positions::from(&chart);
        let year = input.settings.dasha_year;
        let want = chara_dasha(&positions, true, input.jd_ut, year).expect("lagna");
        assert_eq!(dasha, want, "{id}");
        let dashas: Vec<&SignDashaPeriod> = dasha.periods.iter().filter(|p| p.level == 1).collect();
        assert_eq!(dashas[0].signs, [chart.lagna.sign], "{id}");
        assert_eq!(dashas[0].start_jd_ut, input.jd_ut, "{id}");

        // Değişmezler: ilk çevrim on iki burcun permütasyonudur; her süre 1–12 yıldır; dönemler
        // boşluksuz ardışıktır; iki çevrimde aynı burcun yılları toplamı 12'dir.
        let first: Vec<Sign> = dashas
            .iter()
            .filter(|p| p.cycle == 1)
            .map(|p| p.signs[0])
            .collect();
        let mut sorted = first.clone();
        sorted.sort_by_key(|s| s.index());
        assert_eq!(sorted[..], Sign::ALL[..], "{id}");
        assert!(dasha.years.iter().all(|y| (1..=12).contains(y)), "{id}");
        for w in dashas.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut, "{id}");
        }
        for p in dashas.iter().filter(|p| p.cycle == 2) {
            let i = first
                .iter()
                .position(|s| *s == p.signs[0])
                .expect("in the first cycle");
            let years = (p.end_jd_ut - p.start_jd_ut) / year;
            assert!(
                (years + f64::from(dasha.years[i]) - 12.0).abs() < 1e-6,
                "{id}: {:?}",
                p.signs
            );
        }

        // Alt dönemler: 2. seviye haritadaki dönemlerdir; 3. seviye ebeveyni on iki parçaya
        // böler ve ebeveynin burcuyla biter.
        let lagna = chart.lagna.sign;
        let second = chara_dasha_children_blocking(engine, input.clone(), vec![lagna], 1)
            .expect("second level");
        assert_eq!(second[..], dasha.periods[1..13], "{id}");
        let parent = &second[0];
        let third = chara_dasha_children_blocking(engine, input.clone(), parent.signs.clone(), 1)
            .expect("third level");
        assert_eq!(
            third,
            sign_dasha_children(&want, &parent.signs, 1).unwrap(),
            "{id}"
        );
        assert_eq!(third.len(), 12, "{id}");
        assert_eq!(third[0].start_jd_ut, parent.start_jd_ut, "{id}");
        assert_eq!(third[11].end_jd_ut, parent.end_jd_ut, "{id}");
        assert_eq!(third[11].signs[2], parent.signs[1], "{id}");
        assert!(
            third.iter().all(|p| p.level == 3 && !p.has_children),
            "{id}"
        );
        let roots =
            chara_dasha_children_blocking(engine, input.clone(), vec![], 2).expect("second cycle");
        assert!(
            !roots.is_empty() && roots.iter().all(|p| p.cycle == 2 && p.level == 1),
            "{id}"
        );

        // Bir anın yolu: doğumdan 30 yıl sonrası ilk çevrimdedir ve dönemleri anı içerir.
        let at = input.jd_ut + 30.0 * year;
        let path = chara_dasha_at_blocking(engine, input.clone(), at)
            .expect("path")
            .expect("inside the produced span");
        assert_eq!(
            Some(path.clone()),
            sign_dasha_path_at(&want, at).unwrap(),
            "{id}"
        );
        assert_eq!(path.signs.len(), 3, "{id}");
        let holder = dashas
            .iter()
            .find(|p| p.cycle == path.cycle && p.signs[0] == path.signs[0])
            .expect("dasha on the path");
        assert!(holder.start_jd_ut <= at && at < holder.end_jd_ut, "{id}");
        assert_eq!(
            chara_dasha_at_blocking(engine, input.clone(), input.jd_ut - 1.0).expect("before"),
            None,
            "{id}"
        );

        // Saat bilinmiyorsa dasha Rust'ta boş döner (K2): sıra lagnaya bağlıdır.
        let unknown = ChartInput {
            time_known: false,
            lagna_number: None,
            ..input.clone()
        };
        assert_eq!(
            compute_chara_dasha_blocking(engine, unknown.clone()).expect("time unknown"),
            None,
            "{id}"
        );
        assert_eq!(
            chara_dasha_children_blocking(engine, unknown.clone(), vec![lagna], 1)
                .expect("time unknown"),
            [],
            "{id}"
        );
        assert_eq!(
            chara_dasha_at_blocking(engine, unknown, at).expect("time unknown"),
            None,
            "{id}"
        );

        // Ayarlar hesapta kullanılır: eş lord kuralı ve yıl uzunluğu.
        let mut plain = input.clone();
        plain.settings.advanced.jaimini_co_lords = false;
        plain.settings.dasha_year = 360.0;
        let plain_dasha = compute_chara_dasha_blocking(engine, plain)
            .expect("plain lords")
            .expect("dasha");
        assert_eq!(
            plain_dasha,
            chara_dasha(&positions, false, input.jd_ut, 360.0).expect("lagna"),
            "{id}"
        );
        assert_eq!(
            plain_dasha.periods[0].end_jd_ut,
            input.jd_ut + f64::from(plain_dasha.years[0]) * 360.0,
            "{id}"
        );
    }

    // Hatalar `compute_chart` ile aynı kodlara eşlenir; yol ve çevrim hataları Vimshottari'nin
    // koduyla (`invalid_dasha_input`) döner. Parametreler danışan verisi taşımaz.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        ..delhi.clone()
    };
    assert_eq!(
        compute_chara_dasha_blocking(engine, bad_place.clone())
            .unwrap_err()
            .code,
        "invalid_location"
    );
    assert_eq!(
        chara_dasha_children_blocking(engine, bad_place.clone(), vec![], 1)
            .unwrap_err()
            .code,
        "invalid_location"
    );
    assert_eq!(
        chara_dasha_at_blocking(engine, bad_place, delhi.jd_ut)
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
        compute_chara_dasha_blocking(engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );
    let bad_moment = ChartInput {
        jd_ut: f64::NAN,
        ..delhi.clone()
    };
    assert_eq!(
        compute_chara_dasha_blocking(engine, bad_moment)
            .unwrap_err()
            .code,
        "invalid_julian_day"
    );
    let err = chara_dasha_children_blocking(engine, delhi.clone(), vec![], 3).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.params["field"].as_str()),
        ("invalid_dasha_input", "cycle")
    );
    let err =
        chara_dasha_children_blocking(engine, delhi.clone(), vec![Sign::Aries; 3], 1).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.params["field"].as_str()),
        ("invalid_dasha_input", "path")
    );
    let err = chara_dasha_at_blocking(engine, delhi.clone(), f64::NAN).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.params["field"].as_str()),
        ("invalid_dasha_input", "jd_ut")
    );

    // Performans: spec §15 "compute_chara_dasha ≤ 10 ms"; §6 "alt dönemler ≤ 5 ms" (temel harita
    // hesabı dahil).
    let first = compute_chara_dasha_blocking(engine, delhi.clone())
        .expect("dasha")
        .expect("time known")
        .periods[1]
        .signs
        .clone();
    let (median, max) = median_of_15(|| {
        compute_chara_dasha_blocking(engine, delhi.clone()).expect("chara dasha");
    });
    eprintln!("compute_chara_dasha median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "compute_chara_dasha median {median:?}"
    );
    let (median, max) = median_of_15(|| {
        chara_dasha_children_blocking(engine, delhi.clone(), first.clone(), 1).expect("children");
    });
    eprintln!("chara_dasha_children median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(5),
        "chara_dasha_children median {median:?}"
    );
    let at = delhi.jd_ut + 9_000.0;
    let (median, max) = median_of_15(|| {
        chara_dasha_at_blocking(engine, delhi.clone(), at).expect("path");
    });
    eprintln!("chara_dasha_at median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "chara_dasha_at median {median:?}"
    );
}

/// (−180°, 180°] aralığına indirgenmiş açı farkı.
fn wrap180(x: f64) -> f64 {
    (x + 180.0).rem_euclid(360.0) - 180.0
}

/// Ephemeris'ten ölçülen astronomik olgular (ileri modüller spec'i §7 "Astronomik çapraz
/// denetim" ve "Ortalama boylam katsayıları"), 1900–2100 arasında.
#[derive(Debug)]
struct SkyFacts {
    /// (ad, karşı konumlardaki en küçük cheshta, kavuşumlardaki en büyük cheshta, olay sayısı)
    outer: Vec<(&'static str, f64, f64, usize)>,
    /// (ad, alt kavuşumlardaki en küçük cheshta, üst kavuşumlardaki en büyük cheshta, olay
    /// sayısı, alt kavuşumlarda ortalama boylamın Güneş'in karşısından ortalama farkı)
    inner: Vec<(&'static str, f64, f64, usize, f64)>,
    /// (ad, gerçek jeosantrik boylam − ortalama boylam farkının ortalaması)
    mean_offsets: Vec<(&'static str, f64)>,
}

fn sky_facts(se: &mut SwissEph) -> SkyFacts {
    // 1900-01-01 … 2100-01-01 (paketlenen ephemeris dosyalarının içinde).
    const FROM: f64 = 2_415_020.5;
    const TO: f64 = 2_488_069.5;
    fn at(se: &mut SwissEph, t: f64, body: Body) -> (f64, f64) {
        let p = se.body(t, body, Zodiac::Tropical).expect("body");
        (p.lon, p.lon_speed)
    }
    let mean_of = |name: &str, t: f64| {
        let m = mean_longitudes(t);
        match name {
            "sun" => m.sun,
            "mercury" => m.mercury,
            "venus" => m.venus,
            "mars" => m.mars,
            "jupiter" => m.jupiter,
            _ => m.saturn,
        }
    };

    // Uzun dönem ortalaması: gerçek − ortalama.
    let mut mean_offsets = Vec::new();
    for (name, body) in [
        ("sun", Body::Sun),
        ("mars", Body::Mars),
        ("jupiter", Body::Jupiter),
        ("saturn", Body::Saturn),
    ] {
        let (mut sum, mut n, mut t) = (0.0, 0.0, FROM);
        while t < TO {
            sum += wrap180(at(se, t, body).0 - mean_of(name, t));
            n += 1.0;
            t += 3.7;
        }
        mean_offsets.push((name, sum / n));
    }

    // Güneş'le kavuşum (`shift` 0) ya da karşı konum (`shift` 180) anları: iki günlük
    // adımlarla işaret değişimi, sonra ikiye bölme.
    let events = |se: &mut SwissEph, body: Body, shift: f64| -> Vec<f64> {
        let gap =
            |se: &mut SwissEph, t: f64| wrap180(at(se, t, body).0 - at(se, t, Body::Sun).0 - shift);
        let mut out = Vec::new();
        let mut t = FROM;
        let mut prev = gap(se, t);
        while t < TO {
            let next_t = t + 2.0;
            let cur = gap(se, next_t);
            if prev.abs() < 30.0 && cur.abs() < 30.0 && (prev < 0.0) != (cur < 0.0) {
                let (mut lo, mut hi) = (t, next_t);
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    if (gap(se, mid) < 0.0) == (prev < 0.0) {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                out.push(0.5 * (lo + hi));
            }
            prev = cur;
            t = next_t;
        }
        out
    };

    let mut outer = Vec::new();
    for (name, body) in [
        ("mars", Body::Mars),
        ("jupiter", Body::Jupiter),
        ("saturn", Body::Saturn),
    ] {
        let cheshta = |se: &mut SwissEph, t: f64| {
            cheshta_from(mean_of(name, t), at(se, t, body).0, mean_of("sun", t))
        };
        let oppositions = events(se, body, 180.0);
        let conjunctions = events(se, body, 0.0);
        let low = oppositions
            .iter()
            .map(|t| cheshta(se, *t))
            .fold(f64::MAX, f64::min);
        let high = conjunctions
            .iter()
            .map(|t| cheshta(se, *t))
            .fold(f64::MIN, f64::max);
        outer.push((name, low, high, oppositions.len().min(conjunctions.len())));
    }

    let mut inner = Vec::new();
    for (name, body) in [("mercury", Body::Mercury), ("venus", Body::Venus)] {
        let (mut low, mut high) = (f64::MAX, f64::MIN);
        let (mut inferior, mut offset) = (0usize, 0.0);
        for t in events(se, body, 0.0) {
            let (lon, speed) = at(se, t, body);
            let value = cheshta_from(mean_of("sun", t), lon, mean_of(name, t));
            // Alt kavuşumda gezegen geri gider; üst kavuşumda ileri.
            if speed < 0.0 {
                low = low.min(value);
                inferior += 1;
                // Alt kavuşumda güneş merkezli boylam Güneş'in jeosantrik boylamının karşısıdır.
                offset += wrap180(mean_of(name, t) - at(se, t, Body::Sun).0 - 180.0);
            } else {
                high = high.max(value);
            }
        }
        inner.push((name, low, high, inferior, offset / inferior as f64));
    }
    SkyFacts {
        outer,
        inner,
        mean_offsets,
    }
}

fn shadbala_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    let mut reports = 0;
    for (id, input) in inputs {
        let report = compute_shadbala_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        // Doğum saati bilinmiyorsa rapor yoktur (K2).
        let unknown = ChartInput {
            time_known: false,
            lagna_number: None,
            ..input.clone()
        };
        assert_eq!(
            compute_shadbala_blocking(engine, unknown).expect("time unknown"),
            None,
            "{id}"
        );
        let Some(report) = report else {
            // Rapor ancak gün doğumu ya da batımı yoksa boştur (kutup gecesi ya da gündüzü).
            assert!(
                chart
                    .meta
                    .warnings
                    .contains(&ChartWarning::SunriseUnavailable),
                "{id}: no report although the Sun rises and sets"
            );
            continue;
        };
        reports += 1;
        assert_eq!(report.method, SHADBALA_METHOD, "{id}");

        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından ve
        // toplanan astronomik girdiden aynı rapor çıkar.
        let (astro, year_entry, month_entry, year_vara, month_vara) = {
            let (input, chart) = (input.clone(), chart.clone());
            engine
                .call(move |se: &mut SwissEph| {
                    let astro = compute_shadbala_astro(se, &input, &chart)
                        .expect("astro")
                        .expect("sunrise and sunset");
                    let zodiac = Zodiac::Sidereal(input.settings.ayanamsa.sidereal_mode());
                    let sun = chart.grahas[Graha::Sun as usize].sidereal_lon;
                    let month_start = (sun / 30.0).floor() * 30.0;
                    let year = sun_entry_before(se, zodiac, input.jd_ut, 0.0).expect("year entry");
                    let month = sun_entry_before(se, zodiac, input.jd_ut, month_start)
                        .expect("month entry");
                    let sun_at =
                        |se: &mut SwissEph, t: f64| se.body(t, Body::Sun, zodiac).expect("sun").lon;
                    let (year_lon, month_lon) = (sun_at(se, year), sun_at(se, month));
                    // Giriş anının varası, v1'in panchang hesabıyla.
                    let mut vara = |t: f64| {
                        compute_panchang(se, t, input.lat, input.lon, &input.settings)
                            .expect("panchang")
                            .vara
                    };
                    let varas = (vara(year), vara(month));
                    (
                        astro,
                        (year, year_lon),
                        (month, wrap180(month_lon - month_start)),
                        varas.0,
                        varas.1,
                    )
                })
                .expect("engine")
        };
        let want = shadbala(&Positions::from(&chart), &astro).expect("lagna");
        assert_eq!(report.grahas, want.grahas, "{id}");
        assert_eq!(report.bhavas, want.bhavas, "{id}");
        assert_eq!(report.time_lords, want.time_lords, "{id}");

        // Yıl girişi: Güneş'in sidereal boylamı 0° ± 1″; giriş doğumdan önce, en çok bir yıl.
        const ARCSECOND: f64 = 1.0 / 3600.0;
        assert!(
            wrap180(year_entry.1).abs() < ARCSECOND,
            "{id}: Sun at the year entry is at {}",
            year_entry.1
        );
        let back = input.jd_ut - year_entry.0;
        assert!((0.0..366.5).contains(&back), "{id}: year entry {back} days");
        assert!(month_entry.1.abs() < ARCSECOND, "{id}: month entry");
        let back = input.jd_ut - month_entry.0;
        assert!((0.0..32.5).contains(&back), "{id}: month entry {back} days");
        assert_eq!(astro.year_entry_weekday, year_vara, "{id}");
        assert_eq!(astro.month_entry_weekday, month_vara, "{id}");
        assert_eq!(astro.weekday, chart.panchang.vara, "{id}");
        assert_eq!(Some(astro.sunrise_jd_ut), chart.panchang.sunrise_jd_ut);
        assert_eq!(report.time_lords.weekday, vara_lord(chart.panchang.vara));
        assert_eq!(report.time_lords.year, vara_lord(year_vara), "{id}");
        assert_eq!(report.time_lords.month, vara_lord(month_vara), "{id}");
        // Tropikal boylam, ephemeris'in tropikal çerçevedeki değeridir.
        let jd = input.jd_ut;
        let tropical_sun = engine
            .call(move |se: &mut SwissEph| se.body(jd, Body::Sun, Zodiac::Tropical))
            .expect("engine")
            .expect("sun")
            .lon;
        assert!(
            wrap180(astro.tropical_lons[0] - tropical_sun).abs() < 1e-6,
            "{id}: tropical Sun {} vs {tropical_sun}",
            astro.tropical_lons[0]
        );

        // Değişmezler gerçek haritalarda da sağlanır.
        let mut ranks: Vec<u8> = report.grahas.iter().map(|g| g.rank).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, [1, 2, 3, 4, 5, 6, 7], "{id}");
        let mut ranks: Vec<u8> = report.bhavas.iter().map(|b| b.rank).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, (1..=12).collect::<Vec<u8>>(), "{id}");
        for (row, g) in report.grahas.iter().zip(SHADBALA_GRAHAS) {
            assert_eq!(row.graha, g, "{id}");
            let t = &row.temporal;
            assert!((0.0..=60.0).contains(&row.dig), "{id}: {g:?} dig");
            assert!((0.0..=60.0).contains(&row.cheshta), "{id}: {g:?} cheshta");
            assert!((0.0..=60.0).contains(&t.day_night), "{id}: {g:?}");
            assert_eq!(t.year_lord == 15.0, report.time_lords.year == g, "{id}");
            assert_eq!(t.month_lord == 30.0, report.time_lords.month == g, "{id}");
            assert_eq!(t.weekday_lord == 45.0, report.time_lords.weekday == g);
            assert_eq!(t.hora_lord == 60.0, report.time_lords.hora == g, "{id}");
            assert_eq!(
                t.tribhaga == 60.0,
                g == Graha::Jupiter || report.time_lords.tribhaga == g,
                "{id}"
            );
            let total =
                row.sthana.total + row.dig + t.total + row.cheshta + row.naisargika + row.drik;
            assert!((row.total - total).abs() < 1e-9, "{id}: {g:?} total");
        }
        for b in &report.bhavas {
            assert_eq!(b.lord, house_lord(chart.lagna.sign, b.house), "{id}");
            assert_eq!(b.lord_bala, report.grahas[b.lord as usize].total, "{id}");
        }
    }
    // Beş andan biri kutup gecesindedir (Tromsø, Aralık): raporu yoktur.
    assert_eq!(reports, inputs.len() - 1, "reports for the golden moments");

    // Kutup dairesinin hemen altı ve üstü, gündönümlerinde (65°'nin üstünde SE'nin yavaş doğuş
    // araması kullanılır). Rapor varsa ardışık iki doğuş bir günden az ayrıdır ve gün batımı
    // aralarındadır; rapor yoksa harita `SunriseUnavailable` uyarısını taşır. Hiçbiri hata
    // vermez.
    let base = inputs[0].1.clone();
    // (enlem, JD UT, rapor bekleniyor mu)
    let high_latitude = [
        // 2000-06-21 10:00 UT ve 2000-12-21 10:00 UT. 65,5°: yazın Güneş gece yarısında kısa
        // bir süre batar, kışın öğle çevresinde kısa bir süre doğar.
        (65.5, 2_451_716.916_667, true),
        (65.5, 2_451_899.916_667, true),
        // 66,5°: golden ayarının doğuş tanımıyla (disk merkezi, kırılmasız) sınırın hemen
        // altı; gündüz yazın 23,4 saat, kışın 37 dakikadır.
        (66.5, 2_451_716.916_667, true),
        (66.5, 2_451_899.916_667, true),
        // 67°: yazın Güneş batmaz, kışın doğmaz.
        (67.0, 2_451_716.916_667, false),
        (67.0, 2_451_899.916_667, false),
    ];
    for (lat, jd_ut, expect_report) in high_latitude {
        let input = ChartInput {
            jd_ut,
            lat,
            lon: 25.0,
            ..base.clone()
        };
        let id = format!("lat {lat} jd {jd_ut}");
        let report = compute_shadbala_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        assert_eq!(report.is_some(), expect_report, "{id}");
        let Some(report) = report else {
            assert!(
                chart
                    .meta
                    .warnings
                    .contains(&ChartWarning::SunriseUnavailable),
                "{id}: no report although the Sun rises and sets"
            );
            continue;
        };
        let astro = {
            let (input, chart) = (input.clone(), chart.clone());
            engine
                .call(move |se: &mut SwissEph| compute_shadbala_astro(se, &input, &chart))
                .expect("engine")
                .expect("astro")
                .expect("sunrise and sunset")
        };
        let gap = astro.next_sunrise_jd_ut - astro.sunrise_jd_ut;
        let daylight = astro.sunset_jd_ut - astro.sunrise_jd_ut;
        eprintln!("high latitude {id}: sunrise gap {gap:.4} d, daylight {daylight:.4} d");
        assert!((0.9..=1.1).contains(&gap), "{id}: sunrise gap {gap} days");
        assert!(gap <= MAX_SUNRISE_GAP_DAYS, "{id}");
        assert!(
            daylight > 0.0 && daylight < gap,
            "{id}: daylight {daylight}"
        );
        assert!(astro.is_consistent(), "{id}");
        let want = shadbala(&Positions::from(&chart), &astro).expect("lagna");
        assert_eq!(report.grahas, want.grahas, "{id}");
        assert_eq!(report.time_lords, want.time_lords, "{id}");
    }

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
        compute_shadbala_blocking(engine, bad_place)
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
        compute_shadbala_blocking(engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );

    // Astronomik çapraz denetim (spec §7): dış gezegen karşı konumdayken cheshta ≥ 55,
    // kavuşumdayken ≤ 5. İç gezegenlerde alt kavuşumda ≥ 50, üst kavuşumda ≤ 10 (Merkür'ün
    // yörünge dış merkezliği ortalama boylamı gerçek boylamdan 23°'ye kadar ayırır).
    let facts = engine.call(sky_facts).expect("engine");
    eprintln!("{facts:?}");
    for (name, at_opposition, at_conjunction, count) in &facts.outer {
        assert!(*count >= 90, "{name}: {count} oppositions and conjunctions");
        assert!(
            *at_opposition >= 55.0,
            "{name}: cheshta {at_opposition} at an opposition"
        );
        assert!(
            *at_conjunction <= 5.0,
            "{name}: cheshta {at_conjunction} at a conjunction"
        );
    }
    for (name, at_inferior, at_superior, count, offset) in &facts.inner {
        assert!(*count >= 120, "{name}: {count} inferior conjunctions");
        assert!(
            *at_inferior >= 50.0,
            "{name}: cheshta {at_inferior} at an inferior conjunction"
        );
        assert!(
            *at_superior <= 10.0,
            "{name}: cheshta {at_superior} at a superior conjunction"
        );
        // Güneş merkezli ortalama boylamın denetimi: alt kavuşumlar yörüngeye yayıldığı için
        // fark ortalamada sıfırdır. Merkür'de örnekleme gürültüsü büyüktür.
        let limit = if *name == "mercury" { 0.5 } else { 0.05 };
        assert!(
            offset.abs() <= limit,
            "{name}: mean heliocentric longitude is off by {offset}"
        );
    }
    // Ortalama boylam katsayıları (spec §7, §17): gerçek boylamın uzun dönem ortalaması
    // ortalama boylamdır. Güneş ve Mars ±0,05° içindedir; Jüpiter ve Satürn'de ~900 yıllık
    // karşılıklı tedirginlik (büyük eşitsizlik) 200 yılda ortalanmaz (D-kararı; ölçüm: +0,055°
    // ve −0,212°).
    for (name, offset) in &facts.mean_offsets {
        let limit = match *name {
            "jupiter" => 0.1,
            "saturn" => 0.3,
            _ => 0.05,
        };
        assert!(
            offset.abs() <= limit,
            "{name}: long-term mean of true minus mean longitude is {offset}"
        );
    }

    // Performans: spec §15 "compute_shadbala ≤ 40 ms" (temel harita, yıl ve ay girişi dahil).
    let (median, max) = median_of_15(|| {
        compute_shadbala_blocking(engine, delhi.clone()).expect("shadbala");
    });
    eprintln!("compute_shadbala median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(40),
        "compute_shadbala median {median:?}"
    );
}

fn yogas_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    // The v1 cross-check arms below must not be vacuous: the golden moments hold at least one
    // mahapurusha yoga and one Neecha bhanga record.
    let (mut mahapurushas, mut neecha_bhangas) = (0, 0);
    for (id, input) in inputs {
        let report = compute_yogas_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION, "{id}");
        assert_eq!(
            (report.evaluated, report.skipped_time_unknown),
            (41, 0),
            "{id}"
        );
        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından aynı
        // rapor çıkar.
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let positions = Positions::from(&chart);
        assert_eq!(report, yogas(&positions), "{id}");

        // Değişmezler gerçek haritalarda da sağlanır: her kimlik katalogdadır, aynı kimlik ve
        // graha kümesi iki kez dönmez, sayı yogalarından tam biri oluşur.
        let mut seen: Vec<(&str, &[Graha])> = Vec::new();
        for h in &report.hits {
            let rule = yoga_rule(&h.id).unwrap_or_else(|| panic!("{id}: {} unknown", h.id));
            assert_eq!(h.group, rule.group, "{id}: {}", h.id);
            assert!(!h.approximate, "{id}: {}", h.id);
            let key = (h.id.as_str(), h.grahas.as_slice());
            assert!(!seen.contains(&key), "{id}: {key:?} twice");
            seen.push(key);
        }
        let count = |group: YogaGroup| report.hits.iter().filter(|h| h.group == group).count();
        assert_eq!(count(YogaGroup::NabhasaCount), 1, "{id}");

        // v1 olgularıyla çapraz denetim: mahapurusha veren graha v1'in dignity sırasında
        // yücelmiş, moolatrikonada ya da kendi burcundadır ve rasi evi kendradır; Neecha bhanga
        // kaydındaki graha düşüştedir; Budha-Aditya Güneş ile Merkür'ün aynı burcudur; Kuja
        // dosha 3.1'deki olgudur.
        let v1 = |g: Graha| &chart.grahas[g as usize];
        for h in &report.hits {
            match h.group {
                YogaGroup::Mahapurusha => {
                    let g = v1(h.grahas[0]);
                    assert!(
                        matches!(
                            g.dignity,
                            Some(Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign)
                        ),
                        "{id}: {} with dignity {:?}",
                        h.id,
                        g.dignity
                    );
                    let house = g.sign.house_from(chart.lagna.sign);
                    assert!([1, 4, 7, 10].contains(&house), "{id}: {}", h.id);
                }
                YogaGroup::NeechaBhanga => {
                    assert_eq!(v1(h.grahas[0]).dignity, Some(Dignity::Debilitated), "{id}");
                }
                _ => {}
            }
        }
        let has = |yoga: &str| report.hits.iter().any(|h| h.id == yoga);
        // The other direction: a graha that v1 places in its own, moolatrikona or exaltation
        // dignity in a rasi kendra gives its mahapurusha yoga (for these five grahas the
        // moolatrikona sign is an own sign).
        for (yoga, graha) in [
            ("mahapurusha.ruchaka", Graha::Mars),
            ("mahapurusha.bhadra", Graha::Mercury),
            ("mahapurusha.hamsa", Graha::Jupiter),
            ("mahapurusha.malavya", Graha::Venus),
            ("mahapurusha.shasha", Graha::Saturn),
        ] {
            let g = v1(graha);
            let dignified = matches!(
                g.dignity,
                Some(Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign)
            );
            let kendra = [1, 4, 7, 10].contains(&g.sign.house_from(chart.lagna.sign));
            assert_eq!(has(yoga), dignified && kendra, "{id}: {yoga}");
            mahapurushas += usize::from(has(yoga));
        }
        neecha_bhangas += count(YogaGroup::NeechaBhanga);
        assert_eq!(
            has("solar.budha_aditya"),
            v1(Graha::Sun).sign == v1(Graha::Mercury).sign,
            "{id}"
        );
        let dosha = kuja_dosha(&positions);
        assert_eq!(
            has("dosha.kuja"),
            dosha.from_lagna == Some(true) || dosha.from_moon || dosha.from_venus,
            "{id}"
        );

        // Evler rasi evidir (K4): ev sistemi ayarı raporu değiştirmez.
        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
            let other = ChartInput {
                settings: CalcSettings {
                    house_system,
                    ..input.settings.clone()
                },
                ..input.clone()
            };
            // Kutup dairesinin ötesinde Placidus tanımsızdır; harita uyarıyla hesaplanır ve
            // yogalar yine aynıdır.
            assert_eq!(
                compute_yogas_blocking(engine, other).expect("house system"),
                report,
                "{id}: {house_system:?}"
            );
        }

        // Doğum saati bilinmiyorsa lagna isteyen 17 kural atlanır; diğer kayıtlar yaklaşıktır.
        let unknown = compute_yogas_blocking(
            engine,
            ChartInput {
                time_known: false,
                lagna_number: None,
                ..input.clone()
            },
        )
        .expect("time unknown");
        assert_eq!(unknown, yogas(&positions.clone().without_lagna()), "{id}");
        assert_eq!(
            (unknown.evaluated, unknown.skipped_time_unknown),
            (24, 17),
            "{id}"
        );
        assert_eq!(
            (unknown.evaluated + unknown.skipped_time_unknown) as usize,
            YOGA_CATALOG.len()
        );
        for h in &unknown.hits {
            assert!(h.approximate && h.houses.is_empty(), "{id}: {}", h.id);
            assert!(!yoga_rule(&h.id).expect("catalog").needs_lagna, "{id}");
        }
    }

    assert!(
        mahapurushas >= 1,
        "no mahapurusha yoga in the golden moments"
    );
    assert!(
        neecha_bhangas >= 1,
        "no neecha bhanga in the golden moments"
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
        compute_yogas_blocking(engine, bad_place).unwrap_err().code,
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
        compute_yogas_blocking(engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );

    // Performans: spec §15 "compute_yogas ≤ 10 ms" (temel harita hesabı dahil).
    let (median, max) = median_of_15(|| {
        compute_yogas_blocking(engine, delhi.clone()).expect("yogas");
    });
    eprintln!("compute_yogas median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "compute_yogas median {median:?}"
    );
}

fn compat_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    let chart_of = |input: &ChartInput| -> Chart {
        compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart")
    };
    // Beş golden anının her sıralı çifti (kendisiyle eşleşme dahil): 25 rapor.
    let (mut zero_bhakoot, mut zero_nadi) = (0, 0);
    for (groom_id, groom) in inputs {
        for (bride_id, bride) in inputs {
            let id = format!("{groom_id} × {bride_id}");
            let report = compute_compat_blocking(engine, groom.clone(), bride.clone())
                .unwrap_or_else(|e| panic!("{id}: {}", e.code));
            assert_eq!(report.method, KUTA_METHOD, "{id}");
            assert!(!report.approximate, "{id}");
            assert_eq!(
                report.moon_day_change,
                [MoonDayChange::default(); 2],
                "{id}"
            );

            // Çapraz denetim: v1'de golden fixture'larla doğrulanmış iki haritanın
            // konumlarından aynı rapor çıkar.
            let (g, b) = (chart_of(groom), chart_of(bride));
            assert_eq!(
                report,
                kuta_report(&Positions::from(&g), &Positions::from(&b)),
                "{id}"
            );
            // v1 olgularıyla çapraz denetim: sınıflar v1 haritasının Ay'ından (burç, nakshatra)
            // okunur; Bhakoot sınıfı Ay burcunun adıdır.
            let moon = |c: &Chart| c.grahas[Graha::Moon as usize].clone();
            let (gm, bm) = (moon(&g), moon(&b));
            let score = |kuta: Kuta| &report.scores[kuta as usize];
            assert_eq!(
                (
                    score(Kuta::Bhakoot).groom_class.as_str(),
                    score(Kuta::Bhakoot).bride_class.as_str()
                ),
                (
                    format!("{:?}", gm.sign).as_str(),
                    format!("{:?}", bm.sign).as_str()
                ),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Varna).groom_class,
                varna_of(gm.sign).id(),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Yoni).groom_class,
                yoni_of(gm.nakshatra).id(),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Yoni).bride_class,
                yoni_of(bm.nakshatra).id(),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Gana).bride_class,
                gana_of(bm.nakshatra).id(),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Nadi).groom_class,
                nadi_of(gm.nakshatra).id(),
                "{id}"
            );
            assert_eq!(
                score(Kuta::Nadi).bride_class,
                nadi_of(bm.nakshatra).id(),
                "{id}"
            );
            // Bhakoot 0 ↔ v1 burçlarının karşılıklı sayımı 2/12, 5/9 ya da 6/8.
            let count = bm.sign.house_from(gm.sign);
            assert_eq!(
                score(Kuta::Bhakoot).score == 0.0,
                [2, 12, 5, 9, 6, 8].contains(&count),
                "{id}: count {count}"
            );
            zero_bhakoot += usize::from(score(Kuta::Bhakoot).score == 0.0);
            zero_nadi += usize::from(score(Kuta::Nadi).score == 0.0);

            // Değişmezler gerçek haritalarda da sağlanır.
            let sum: f64 = report.scores.iter().map(|s| s.score).sum();
            assert_eq!(report.total, sum, "{id}");
            assert!((0.0..=36.0).contains(&report.total), "{id}");
            let swapped = compute_compat_blocking(engine, bride.clone(), groom.clone())
                .expect("swapped roles");
            for kuta in [
                Kuta::Nadi,
                Kuta::Bhakoot,
                Kuta::Yoni,
                Kuta::GrahaMaitri,
                Kuta::Tara,
            ] {
                assert_eq!(
                    score(kuta).score,
                    swapped.scores[kuta as usize].score,
                    "{id}: {kuta:?}"
                );
            }
            assert_eq!(
                report.kuja_dosha,
                [swapped.kuja_dosha[1], swapped.kuja_dosha[0]],
                "{id}"
            );
            if groom_id == bride_id {
                // Aynı Ay: nadi 0, yoni 4, bhakoot 7, maitri 5; toplam 28; istisna yok.
                assert_eq!(
                    [Kuta::Nadi, Kuta::Yoni, Kuta::Bhakoot, Kuta::GrahaMaitri]
                        .map(|k| score(k).score),
                    [0.0, 4.0, 7.0, 5.0],
                    "{id}"
                );
                assert_eq!(report.total, 28.0, "{id}");
                assert!(!report.nadi_exception && !report.bhakoot_exception, "{id}");
            }

            // Kuja dosha 3.1'in olgusudur; karşılıklı yerleşim v1 haritalarının burçlarından
            // ve retro işaretinden okunur, evler diğer haritanın lagna burcundan sayılır.
            assert_eq!(
                report.kuja_dosha,
                [
                    kuja_dosha(&Positions::from(&g)),
                    kuja_dosha(&Positions::from(&b))
                ],
                "{id}"
            );
            for (i, row) in report.overlay.iter().enumerate() {
                let (owner, own, other) = if i < 9 {
                    (KutaRole::Groom, &g, &b)
                } else {
                    (KutaRole::Bride, &b, &g)
                };
                let v1 = &own.grahas[i % 9];
                assert_eq!((row.owner, row.graha), (owner, v1.graha), "{id}");
                assert_eq!(
                    (row.sign, row.deg_in_sign, row.retrograde),
                    (v1.sign, v1.deg_in_sign, v1.retrograde),
                    "{id}: {:?}",
                    v1.graha
                );
                assert_eq!(
                    row.house_from_lagna,
                    Some(v1.sign.house_from(other.lagna.sign)),
                    "{id}"
                );
                assert_eq!(
                    row.house_from_moon,
                    v1.sign.house_from(other.grahas[Graha::Moon as usize].sign),
                    "{id}"
                );
            }
        }
    }
    assert!(zero_bhakoot >= 1, "no zero bhakoot among the golden pairs");
    // Beş haritanın kendisiyle eşleşmesi nadi 0 verir.
    assert!(zero_nadi >= 5);

    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let istanbul = inputs
        .iter()
        .find(|(id, _)| id == "istanbul-1987-03-17")
        .expect("istanbul fixture")
        .1
        .clone();
    let base = compute_compat_blocking(engine, delhi.clone(), istanbul.clone()).expect("report");

    // Evler rasi evidir (K4): ev sistemi ayarı raporu değiştirmez.
    for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
        let with = |input: &ChartInput| ChartInput {
            settings: CalcSettings {
                house_system,
                ..input.settings.clone()
            },
            ..input.clone()
        };
        assert_eq!(
            compute_compat_blocking(engine, with(&delhi), with(&istanbul)).expect("house system"),
            base,
            "{house_system:?}"
        );
    }

    // Doğum saati bilinmeyen harita: rapor yaklaşıktır, o haritanın lagnasına bağlı değerler
    // boştur, puanlar değişmez (aynı an), ve Ay'ın o gün içindeki değişimi v1 haritasının
    // gün başındaki ve sonundaki Ay'ıyla aynıdır.
    for (id, input) in inputs {
        let unknown = ChartInput {
            time_known: false,
            lagna_number: None,
            ..input.clone()
        };
        let report = compute_compat_blocking(engine, delhi.clone(), unknown.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let known =
            compute_compat_blocking(engine, delhi.clone(), input.clone()).expect("time known");
        assert!(report.approximate, "{id}");
        assert_eq!(report.scores, known.scores, "{id}");
        assert_eq!(report.total, known.total, "{id}");
        assert_eq!(report.kuja_dosha[0], known.kuja_dosha[0], "{id}");
        assert_eq!(report.kuja_dosha[1].from_lagna, None, "{id}");
        assert_eq!(
            report.kuja_dosha[1].from_moon, known.kuja_dosha[1].from_moon,
            "{id}"
        );
        // Damat rolünün grahaları gelin rolünün (bilinmeyen) lagnasından sayılamaz; tersi sayılır.
        assert!(
            report.overlay[..9]
                .iter()
                .all(|r| r.house_from_lagna.is_none()),
            "{id}"
        );
        assert!(
            report.overlay[9..]
                .iter()
                .all(|r| r.house_from_lagna.is_some()),
            "{id}"
        );
        for (a, b) in report.overlay.iter().zip(&known.overlay) {
            assert_eq!(
                (a.sign, a.house_from_moon),
                (b.sign, b.house_from_moon),
                "{id}"
            );
        }
        let moon_at = |offset: f64| {
            let chart = chart_of(&ChartInput {
                jd_ut: input.jd_ut + offset,
                ..input.clone()
            });
            let moon = &chart.grahas[Graha::Moon as usize];
            (moon.nakshatra, moon.sign)
        };
        let (start, end) = (moon_at(-0.5), moon_at(0.5));
        assert_eq!(
            report.moon_day_change,
            [
                MoonDayChange::default(),
                MoonDayChange {
                    nakshatra: start.0 != end.0,
                    sign: start.1 != end.1
                }
            ],
            "{id}"
        );
        // Rol değişince olgu da yer değiştirir.
        let swapped = compute_compat_blocking(engine, unknown, delhi.clone()).expect("swapped");
        assert_eq!(
            swapped.moon_day_change[0], report.moon_day_change[1],
            "{id}"
        );
        assert_eq!(swapped.moon_day_change[1], MoonDayChange::default(), "{id}");
    }

    // Hatalar `compute_chart` ile aynı kodlara ve aynı parametrelere eşlenir, hangi haritadan
    // gelirse gelsin: parametreler yalnızca reddedilen girdinin kendi alanıdır (`compute_chart`
    // ile aynı anahtarlar); geçerli diğer haritanın anı, konumu ya da ayarı hataya girmez.
    // Parametreler loglanmaz (C1).
    let bad_place = ChartInput {
        lat: 91.0,
        time_known: false,
        lagna_number: None,
        ..delhi.clone()
    };
    for (groom, bride) in [(&bad_place, &istanbul), (&istanbul, &bad_place)] {
        let err = compute_compat_blocking(engine, groom.clone(), bride.clone()).unwrap_err();
        assert_eq!(err.code, "invalid_location");
        let params: Vec<(&str, &str)> = err
            .params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            params,
            [
                ("lat", bad_place.lat.to_string().as_str()),
                ("lon", bad_place.lon.to_string().as_str())
            ]
        );
        let other = [
            istanbul.jd_ut.to_string(),
            istanbul.lat.to_string(),
            istanbul.lon.to_string(),
        ];
        assert!(err.params.values().all(|v| !other.contains(v)));
    }
    let old_settings = ChartInput {
        settings: CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        },
        ..delhi.clone()
    };
    let err = compute_compat_blocking(engine, istanbul.clone(), old_settings).unwrap_err();
    assert_eq!(err.code, "unsupported_schema_version");
    assert_eq!(
        err.params.keys().map(String::as_str).collect::<Vec<_>>(),
        ["found", "supported"]
    );

    // Performans: spec §15 "compute_compat ≤ 20 ms" (iki temel harita dahil). En pahalı
    // durum iki haritanın da doğum saatinin bilinmemesidir (altı harita hesabı).
    let (median, max) = median_of_15(|| {
        compute_compat_blocking(engine, delhi.clone(), istanbul.clone()).expect("compat");
    });
    eprintln!("compute_compat median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(20),
        "compute_compat median {median:?}"
    );
    let unknown = |input: &ChartInput| ChartInput {
        time_known: false,
        lagna_number: None,
        ..input.clone()
    };
    let (median, max) = median_of_15(|| {
        compute_compat_blocking(engine, unknown(&delhi), unknown(&istanbul)).expect("compat");
    });
    eprintln!("compute_compat (both times unknown) median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(20),
        "compute_compat (both times unknown) median {median:?}"
    );
}

const SECOND: f64 = 1.0 / 86_400.0;

/// Zaman çizelgesi çekirdeği, gerçek ephemeris'le (ileri modüller spec'i §10 "Test"): v1
/// panchangıyla çapraz denetim ve astronomik olgular.
fn timeline_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    // Çapraz denetim (asıl güvence): beş golden anın çevresindeki altı günde, aralıkların
    // içinden örneklenen anlarda `compute_panchang` aynı beş uzvu verir; aralığın sonu onun
    // bitiş anıyla 1 saniye içinde örtüşür. Tromsø örneği kutup gecesindedir: vara dizisi
    // 06:00 yerel ortalama saat kuralını yürür.
    for (id, input) in inputs {
        let (id, input) = (id.clone(), input.clone());
        let checked = engine
            .call(move |se: &mut SwissEph| {
                let s = &input.settings;
                let (lat, lon) = (input.lat, input.lon);
                let (from, to) = (input.jd_ut - 3.0, input.jd_ut + 3.0);
                let tithis = tithi_series(se, s, from, to).expect("tithis");
                let karanas = karana_series(se, s, from, to).expect("karanas");
                let nakshatras = nakshatra_series(se, s, from, to).expect("nakshatras");
                let yogas = nitya_yoga_series(se, s, from, to).expect("yogas");
                let varas = vara_series(se, s, lat, lon, from, to).expect("varas");
                for series_ends in [
                    (tithis[0].start_jd_ut, tithis[tithis.len() - 1].end_jd_ut),
                    (karanas[0].start_jd_ut, karanas[karanas.len() - 1].end_jd_ut),
                    (
                        nakshatras[0].start_jd_ut,
                        nakshatras[nakshatras.len() - 1].end_jd_ut,
                    ),
                    (yogas[0].start_jd_ut, yogas[yogas.len() - 1].end_jd_ut),
                    (varas[0].start_jd_ut, varas[varas.len() - 1].end_jd_ut),
                ] {
                    assert_eq!(series_ends, (from, to), "{id}: series cover the span");
                }
                let mut ends = 0;
                for k in 0..120 {
                    let jd = from + 0.011 + 0.0499 * f64::from(k);
                    let p = compute_panchang(se, jd, lat, lon, s).expect("panchang");
                    let at = |what: &str| format!("{id}: {what} at +{:.4}", jd - from);
                    let t = tithis.iter().find(|i| i.contains(jd)).expect("tithi");
                    let ka = karanas.iter().find(|i| i.contains(jd)).expect("karana");
                    let n = nakshatras
                        .iter()
                        .find(|i| i.contains(jd))
                        .expect("nakshatra");
                    let y = yogas.iter().find(|i| i.contains(jd)).expect("yoga");
                    let v = varas.iter().find(|i| i.contains(jd)).expect("vara");
                    assert_eq!(t.value, p.tithi.no, "{}", at("tithi"));
                    assert_eq!(ka.value, p.karana.no, "{}", at("karana"));
                    assert_eq!(
                        (n.value.nakshatra, n.value.pada),
                        (p.nakshatra.no, p.nakshatra.pada),
                        "{}",
                        at("nakshatra")
                    );
                    assert_eq!(y.value, p.yoga.no, "{}", at("yoga"));
                    assert_eq!(v.value, p.vara, "{}", at("vara"));
                    let nakshatra_end = nakshatras
                        .iter()
                        .rfind(|i| i.value.nakshatra == n.value.nakshatra)
                        .expect("nakshatra end")
                        .end_jd_ut;
                    for (end, want, what) in [
                        (t.end_jd_ut, p.tithi.ends_jd_ut, "tithi end"),
                        (ka.end_jd_ut, p.karana.ends_jd_ut, "karana end"),
                        (y.end_jd_ut, p.yoga.ends_jd_ut, "yoga end"),
                        (nakshatra_end, p.nakshatra.ends_jd_ut, "nakshatra end"),
                    ] {
                        // Aralığın kırpılan ucu gerçek bir sınır değildir.
                        if end < to {
                            assert!((end - want).abs() <= SECOND, "{}", at(what));
                            ends += 1;
                        }
                    }
                }
                ends
            })
            .expect("engine");
        assert!(checked > 300, "{checked} boundaries compared");
    }

    // Astronomik olgular: bulunan burç ve nakshatra girişinde boylam sınırın 1″ içindedir;
    // dönüş anında hız işaret değiştirir; lagna burcu sınırın iki yanında farklıdır.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    engine
        .call(move |se: &mut SwissEph| {
            let arcsecond = 1.0 / 3600.0;
            let off_grid = |lon: f64, span: f64| {
                let r = lon.rem_euclid(span);
                r.min(span - r)
            };
            let (from, to) = (delhi.jd_ut, delhi.jd_ut + 800.0);
            for node in [NodeKind::Mean, NodeKind::True] {
                let s = CalcSettings {
                    node,
                    ..delhi.settings.clone()
                };
                let sidereal = Zodiac::Sidereal(s.ayanamsa.sidereal_mode());
                let bodies = [
                    (Graha::Sun, Body::Sun, 0.0),
                    (Graha::Moon, Body::Moon, 0.0),
                    (Graha::Mars, Body::Mars, 0.0),
                    (Graha::Mercury, Body::Mercury, 0.0),
                    (Graha::Jupiter, Body::Jupiter, 0.0),
                    (Graha::Venus, Body::Venus, 0.0),
                    (Graha::Saturn, Body::Saturn, 0.0),
                    (Graha::Rahu, s.node.body(), 0.0),
                    (Graha::Ketu, s.node.body(), 180.0),
                ];
                for (graha, body, shift) in bodies {
                    // Gerçek düğüm 800 günde yüzlerce kez yön değiştirir; ortalama düğümle
                    // denetlemek yeterlidir.
                    if node == NodeKind::True && !graha.is_node() {
                        continue;
                    }
                    let span_days = if graha == Graha::Moon { 60.0 } else { 800.0 };
                    let to = from + span_days;
                    let signs = graha_sign_series(se, &s, graha, from, to).expect("signs");
                    assert!(signs.len() > 1, "{graha:?}: at least one ingress");
                    for i in &signs[1..] {
                        let lon = se.body(i.start_jd_ut, body, sidereal).expect("lon").lon + shift;
                        assert!(
                            off_grid(lon, 30.0) < arcsecond,
                            "{graha:?} sign ingress at {lon}"
                        );
                        assert_eq!(Sign::of(lon), i.value, "{graha:?}");
                    }
                    let naks = graha_nakshatra_series(se, &s, graha, from, to).expect("naks");
                    assert!(naks.len() >= signs.len(), "{graha:?}");
                    for i in &naks[1..] {
                        let lon = se.body(i.start_jd_ut, body, sidereal).expect("lon").lon + shift;
                        assert!(
                            off_grid(lon, 40.0 / 3.0) < arcsecond,
                            "{graha:?} nakshatra ingress at {lon}"
                        );
                    }
                }
            }
            let s = delhi.settings.clone();
            let sidereal = Zodiac::Sidereal(s.ayanamsa.sidereal_mode());
            // 800 günde her grahanın en az bir retro dönemi vardır (Mars ~26 ayda bir).
            for (graha, body) in [
                (Graha::Mars, Body::Mars),
                (Graha::Mercury, Body::Mercury),
                (Graha::Jupiter, Body::Jupiter),
                (Graha::Venus, Body::Venus),
                (Graha::Saturn, Body::Saturn),
            ] {
                assert!(STATION_GRAHAS.contains(&graha));
                let motion = graha_motion_series(se, &s, graha, from, to).expect("motion");
                assert!(motion.len() >= 3, "{graha:?}: {} intervals", motion.len());
                for w in motion.windows(2) {
                    assert_ne!(w[0].value, w[1].value);
                    let station = w[1].start_jd_ut;
                    let before = se.body(station - 0.02, body, sidereal).expect("speed");
                    let after = se.body(station + 0.02, body, sidereal).expect("speed");
                    assert!(
                        before.lon_speed * after.lon_speed < 0.0,
                        "{graha:?}: speed changes sign at the station"
                    );
                    assert_eq!(after.lon_speed < 0.0, w[1].value == Motion::Retrograde);
                }
            }
            // Merkür yılda üç kez geri gider: 800 günde en az altı retro dönemi.
            let mercury = graha_motion_series(se, &s, Graha::Mercury, from, to).expect("motion");
            let retro = mercury
                .iter()
                .filter(|i| i.value == Motion::Retrograde)
                .count();
            assert!((6..=8).contains(&retro), "{retro} mercury retrogrades");
            assert_eq!(
                graha_motion_series(se, &s, Graha::Rahu, from, to)
                    .unwrap_err()
                    .to_string(),
                "invalid muhurta input: graha"
            );

            // Lagna: 3 günde 36 burç değişimi; sınırın 1 saniye öncesinde önceki burç.
            let (lat, lon) = (delhi.lat, delhi.lon);
            let lagnas = lagna_sign_series(se, &s, lat, lon, from, from + 3.0).expect("lagna");
            assert!((36..=38).contains(&lagnas.len()), "{}", lagnas.len());
            for w in lagnas.windows(2) {
                let at = w[1].start_jd_ut;
                assert_eq!(lagna_sign_at(se, &s, at, lat, lon).unwrap(), w[1].value);
                assert_eq!(
                    lagna_sign_at(se, &s, at - SECOND, lat, lon).unwrap(),
                    w[0].value
                );
                assert_eq!(w[1].value.index(), (w[0].value.index() + 1) % 12);
            }
        })
        .expect("engine");
}

fn fixed_place(input: &ChartInput) -> MuhurtaPlace {
    MuhurtaPlace {
        lat: input.lat,
        lon: input.lon,
        tz: TzChoice::Fixed(0),
    }
}

/// Bir aramanın bütün dilimleri; her dilimin süresiyle.
fn search_all(
    engine: &Engine,
    query: &MuhurtaQuery,
    settings: &CalcSettings,
    now: Timestamp,
) -> (Vec<MuhurtaWindowView>, Vec<Duration>, MuhurtaSearchView) {
    let mut windows = Vec::new();
    let mut times = Vec::new();
    let mut cursor = None;
    loop {
        let t = Instant::now();
        let page = muhurta_search_blocking(engine, query.clone(), settings.clone(), cursor, now)
            .unwrap_or_else(|e| panic!("search: {} {:?}", e.code, e.params));
        times.push(t.elapsed());
        assert_eq!(page.scanned as usize, times.len());
        assert_eq!(page.method, MUHURTA_METHOD);
        windows.extend(page.windows.iter().copied());
        match page.next {
            Some(next) => cursor = Some(next),
            None => {
                assert_eq!(page.scanned, page.chunks);
                return (windows, times, page);
            }
        }
    }
}

fn muhurta_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    let now: Timestamp = "2026-11-26T12:00:00Z".parse().expect("timestamp");

    // Gün panchangı: golden anı içeren gün. Vara, gün doğumu ve batımı v1 haritasının
    // panchangıyla aynıdır; doğum Ay'ı o anın Ay'ıysa tara Janma (1), chandra bala 1. evdir.
    for (id, input) in inputs {
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let moon = &chart.grahas[Graha::Moon as usize];
        let natal = NatalMoon {
            nakshatra: moon.nakshatra,
            sign: moon.sign,
        };
        let view = day_panchang_blocking(
            engine,
            TransitAt::Jd(input.jd_ut),
            now,
            fixed_place(input),
            input.settings.clone(),
            Some(natal),
        )
        .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let d = &view.day;
        assert_eq!(view.offset_secs, 0, "{id}");
        assert_eq!(d.method, MUHURTA_METHOD, "{id}");
        assert!(
            d.day_start_jd_ut <= input.jd_ut && input.jd_ut < d.day_end_jd_ut,
            "{id}: the day contains the moment"
        );
        assert_eq!(d.vara, chart.panchang.vara, "{id}: vara");
        let polar = chart
            .meta
            .warnings
            .contains(&ChartWarning::SunriseUnavailable);
        assert_eq!(
            d.warnings.contains(&ChartWarning::SunriseUnavailable),
            polar,
            "{id}"
        );
        if polar {
            // Kutup gecesi: dilimler yok, uzuvlar var; gün 24 saattir.
            assert!(d.segments.is_none() && d.sunrise_jd_ut.is_none(), "{id}");
            assert!(
                (d.day_end_jd_ut - d.day_start_jd_ut - 1.0).abs() < 1e-9,
                "{id}"
            );
        } else {
            assert_eq!(d.sunrise_jd_ut, chart.panchang.sunrise_jd_ut, "{id}");
            assert_eq!(d.sunset_jd_ut, chart.panchang.sunset_jd_ut, "{id}");
            let seg = d.segments.as_ref().expect("segments");
            assert_eq!(seg.eighths.len(), 8, "{id}");
            assert_eq!(seg.horas.len(), 24, "{id}");
            assert_eq!(seg.horas[0].value, vara_lord(d.vara), "{id}");
            assert_eq!(Some(seg.eighths[7].end_jd_ut), d.sunset_jd_ut, "{id}");
            assert_eq!(seg.horas[23].end_jd_ut, d.day_end_jd_ut, "{id}");
        }
        let at = |jd: f64| move |i: &&jyotish_core::Interval<_>| i.contains(jd);
        let tithi = d.tithis.iter().find(at(input.jd_ut)).expect("tithi").value;
        assert_eq!(
            (tithi.no, tithi.paksha),
            (chart.panchang.tithi.no, chart.panchang.tithi.paksha),
            "{id}"
        );
        let nakshatra = d.nakshatras.iter().find(|i| i.contains(input.jd_ut));
        let nakshatra = nakshatra.expect("nakshatra").value;
        assert_eq!((nakshatra.no, nakshatra.pada), (moon.nakshatra, moon.pada));
        let karana = d.karanas.iter().find(|i| i.contains(input.jd_ut));
        assert_eq!(
            karana.expect("karana").value.name,
            chart.panchang.karana.name,
            "{id}"
        );
        let yoga = d.yogas.iter().find(|i| i.contains(input.jd_ut));
        assert_eq!(yoga.expect("yoga").value, chart.panchang.yoga.no, "{id}");
        let personal = d.personal.as_ref().expect("personal");
        let tara = personal.tara.iter().find(|i| i.contains(input.jd_ut));
        assert_eq!(tara.expect("tara").value.number, 1, "{id}: janma tara");
        let chandra = personal.chandra.iter().find(|i| i.contains(input.jd_ut));
        assert_eq!(chandra.expect("chandra").value.house, 1, "{id}");
        // Yanık Jüpiter ya da Venüs işareti, haritanın yanma olgusuyla aynıdır.
        for (flag, graha) in [
            (MuhurtaFlag::JupiterCombust, Graha::Jupiter),
            (MuhurtaFlag::VenusCombust, Graha::Venus),
        ] {
            let flagged = d
                .flags
                .iter()
                .any(|f| f.value == flag && f.contains(input.jd_ut));
            assert_eq!(
                flagged, chart.grahas[graha as usize].combust,
                "{id}: {flag:?}"
            );
        }
        // Doğum Ay'ı verilmezse kişiye bağlı etken yoktur.
        let plain = day_panchang_blocking(
            engine,
            TransitAt::Jd(input.jd_ut),
            now,
            fixed_place(input),
            input.settings.clone(),
            None,
        )
        .expect("day");
        assert_eq!(plain.day.personal, None, "{id}");
        assert_eq!(plain.day.tithis, d.tithis, "{id}");
    }

    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let kolkata_time = MuhurtaPlace {
        lat: delhi.lat,
        lon: delhi.lon,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    };
    // "Şimdi" Rust'ta okunur; saatler yerin ofsetiyle gösterilir (+05:30).
    let today = day_panchang_blocking(
        engine,
        TransitAt::Now,
        now,
        kolkata_time.clone(),
        delhi.settings.clone(),
        None,
    )
    .expect("today");
    assert_eq!(today.offset_secs, 19_800);
    // 2026-11-26 12:00 UTC = JD 2461371.0; Delhi'de Perşembe, gün doğumundan sonra.
    assert!(today.day.day_start_jd_ut <= 2_461_371.0 && 2_461_371.0 < today.day.day_end_jd_ut);
    assert_eq!(today.day.vara, Vara::Thursday);

    // Hatalar alan adıyla döner; yer ve tarih hata parametresine girmez.
    let bad_place = MuhurtaPlace {
        lat: 95.0,
        ..kolkata_time.clone()
    };
    let e = day_panchang_blocking(
        engine,
        TransitAt::Now,
        now,
        bad_place,
        delhi.settings.clone(),
        None,
    )
    .unwrap_err();
    assert_eq!(e.code, "invalid_muhurta_input");
    assert_eq!(e.params.len(), 1);
    assert_eq!(e.params["field"], "place");
    let bad_natal = NatalMoon {
        nakshatra: 0,
        sign: Sign::Aries,
    };
    let e = day_panchang_blocking(
        engine,
        TransitAt::Now,
        now,
        kolkata_time.clone(),
        delhi.settings.clone(),
        Some(bad_natal),
    )
    .unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.params["field"].as_str()),
        ("invalid_muhurta_input", "natal")
    );
    // The moment itself: an unknown zone, a wall time inside a daylight saving gap, a broken
    // date and a non-finite day are all reported by the field name alone (no zone id, no local
    // time, no suggestion). A bad place is reported first, whatever the moment is.
    let local = |date: &str, time: &str, tz: &str| TransitAt::Local {
        date: date.to_owned(),
        time: time.to_owned(),
        tz: tz.to_owned(),
    };
    for (at, want) in [
        (local("2026-11-25", "10:00", "Mars/Olympus"), "at"),
        (local("2026-03-29", "02:30", "Europe/Berlin"), "at"),
        (local("25.11.2026", "10:00", "Europe/Berlin"), "at"),
        (local("2026-11-25", "25:00", "Europe/Berlin"), "at"),
        (TransitAt::Jd(f64::NAN), "at"),
        (TransitAt::Jd(1e9), "at"),
    ] {
        let e = day_panchang_blocking(
            engine,
            at.clone(),
            now,
            kolkata_time.clone(),
            delhi.settings.clone(),
            None,
        )
        .unwrap_err();
        assert_eq!(e.code, "invalid_muhurta_input", "{at:?}");
        assert_eq!(e.params.len(), 1, "{at:?}: {:?}", e.params);
        assert_eq!(e.params["field"], want, "{at:?}");
        let e = day_panchang_blocking(
            engine,
            at.clone(),
            now,
            MuhurtaPlace {
                lat: 95.0,
                ..kolkata_time.clone()
            },
            delhi.settings.clone(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.params.len(), e.params["field"].as_str()),
            ("invalid_muhurta_input", 1, "place"),
            "{at:?}"
        );
    }
    let old = CalcSettings {
        schema_version: 1,
        ..CalcSettings::default()
    };
    assert_eq!(
        day_panchang_blocking(engine, TransitAt::Now, now, kolkata_time.clone(), old, None)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );

    // Performans: spec §15 "day_panchang ≤ 50 ms".
    let (median, max) = median_of_15(|| {
        day_panchang_blocking(
            engine,
            TransitAt::Jd(delhi.jd_ut),
            now,
            kolkata_time.clone(),
            delhi.settings.clone(),
            Some(NatalMoon {
                nakshatra: 15,
                sign: Sign::Libra,
            }),
        )
        .expect("day");
    });
    eprintln!("day_panchang median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(50),
        "day_panchang median {median:?}"
    );

    // Arama: 366 gün, lagna süzgeci olmadan. Bütün diğer koşullar etkin (her dizi hesaplanır).
    let natal = NatalMoon {
        nakshatra: 15,
        sign: Sign::Libra,
    };
    let criteria = MuhurtaCriteria {
        weekdays: vec![Vara::Monday, Vara::Wednesday, Vara::Thursday, Vara::Friday],
        tithis: (1..=30).filter(|t| t % 15 != 0).collect(),
        nakshatras: (1..=27).filter(|n| n % 3 != 0).collect(),
        lagna_signs: Vec::new(),
        daytime_only: true,
        min_minutes: 30,
        exclude: MuhurtaFlag::ALL.to_vec(),
        require_tara: true,
        require_chandra: true,
    };
    let query = MuhurtaQuery {
        from_date: Some("2026-01-01".into()),
        to_date: Some("2027-01-01".into()),
        place: kolkata_time.clone(),
        criteria: criteria.clone(),
        natal: Some(natal),
    };
    let s = delhi.settings.clone();
    let started = Instant::now();
    let (windows, times, last) = search_all(engine, &query, &s, now);
    let total = started.elapsed();
    let slowest = times.iter().max().copied().unwrap_or_default();
    eprintln!(
        "muhurta_search (366 days) total {total:?}, first page {:?}, slowest page {slowest:?}, {} windows",
        times[0],
        windows.len()
    );
    assert_eq!(times.len(), 12, "twelve chunks for 366 days");
    assert_eq!(last.chunks, 12);
    // 2026-01-01 00:00 +05:30 = 2025-12-31 18:30 UTC.
    assert!((last.from_jd_ut - 2_461_041.270_833_333).abs() < 1e-6);
    assert!((last.to_jd_ut - last.from_jd_ut - 366.0).abs() < 1e-9);
    assert_eq!(
        (last.from_offset_secs, last.to_offset_secs),
        (19_800, 19_800)
    );
    assert!(total <= Duration::from_secs(2), "366 days {total:?}");
    assert!(
        times[0] <= Duration::from_millis(500),
        "first page {:?}",
        times[0]
    );
    // Motor iş parçacığı dilim aralarında bırakılır: araya giren hesap en çok bir dilim bekler.
    assert!(
        slowest <= Duration::from_millis(300),
        "slowest page {slowest:?}"
    );
    assert!(windows.len() > 20, "{} windows", windows.len());
    let check = |windows: &[MuhurtaWindowView], lagna: &[Sign]| {
        let windows = windows.to_vec();
        let (lat, lon) = (delhi.lat, delhi.lon);
        let (s, c) = (delhi.settings.clone(), criteria.clone());
        let lagna = lagna.to_vec();
        engine
            .call(move |se: &mut SwissEph| {
                let mut previous_end = f64::MIN;
                for w in &windows {
                    assert!(previous_end <= w.start_jd_ut, "ordered and disjoint");
                    previous_end = w.end_jd_ut;
                    assert!(
                        w.end_jd_ut - w.start_jd_ut >= 30.0 / 1440.0,
                        "minimum length"
                    );
                    assert_eq!(w.start_offset_secs, 19_800);
                    // Başlangıç anının özeti, noktasal hesapla aynıdır.
                    let at_start = window_snapshot(se, &s, w.start_jd_ut, lat, lon).unwrap();
                    assert_eq!(w.snapshot, at_start);
                    for jd in [
                        w.start_jd_ut + SECOND,
                        0.5 * (w.start_jd_ut + w.end_jd_ut),
                        w.end_jd_ut - SECOND,
                    ] {
                        let x = window_snapshot(se, &s, jd, lat, lon).unwrap();
                        let p = compute_panchang(se, jd, lat, lon, &s).unwrap();
                        assert_eq!((x.vara, x.tithi, x.yoga), (p.vara, p.tithi.no, p.yoga.no));
                        assert!(c.weekdays.contains(&x.vara), "weekday at {jd}");
                        assert!(c.tithis.contains(&x.tithi) && !is_rikta(x.tithi));
                        assert!(c.nakshatras.contains(&x.nakshatra));
                        assert!(!is_vishti(x.karana) && x.yoga != 17 && x.yoga != 27);
                        assert!(!is_panchaka(NakshatraPada {
                            nakshatra: x.nakshatra,
                            pada: x.pada
                        }));
                        let (rise, set) = (p.sunrise_jd_ut.unwrap(), p.sunset_jd_ut.unwrap());
                        assert!(rise <= jd && jd < set, "daytime at {jd}");
                        // Rahu dilimi: Pazartesi 2., Çarşamba 5., Perşembe 6., Cuma 4.
                        let eighth = ((jd - rise) / (set - rise) * 8.0).floor() as u8 + 1;
                        let rahu = match x.vara {
                            Vara::Monday => 2,
                            Vara::Wednesday => 5,
                            Vara::Thursday => 6,
                            _ => 4,
                        };
                        assert_ne!(eighth, rahu, "rahu segment at {jd}");
                        if !lagna.is_empty() {
                            assert!(lagna.contains(&x.lagna_sign), "lagna at {jd}");
                        }
                    }
                }
            })
            .expect("engine");
    };
    check(&windows, &[]);

    // Lagna süzgeciyle: bütçe ≤ 5 sn; pencereler ilk aramanınkilerin içindedir.
    let lagna = vec![Sign::Taurus, Sign::Leo, Sign::Scorpio, Sign::Aquarius];
    let with_lagna = MuhurtaQuery {
        criteria: MuhurtaCriteria {
            lagna_signs: lagna.clone(),
            ..criteria.clone()
        },
        ..query.clone()
    };
    let started = Instant::now();
    let (lagna_windows, times, _) = search_all(engine, &with_lagna, &s, now);
    let total = started.elapsed();
    let slowest = times.iter().max().copied().unwrap_or_default();
    eprintln!(
        "muhurta_search (366 days, lagna) total {total:?}, first page {:?}, slowest page {slowest:?}, {} windows",
        times[0],
        lagna_windows.len()
    );
    assert!(
        total <= Duration::from_secs(5),
        "366 days with lagna {total:?}"
    );
    assert!(
        times[0] <= Duration::from_millis(500),
        "first page {:?}",
        times[0]
    );
    assert!(
        slowest <= Duration::from_millis(300),
        "slowest page {slowest:?}"
    );
    assert!(!lagna_windows.is_empty());
    check(&lagna_windows, &lagna);
    for w in &lagna_windows {
        assert!(
            windows
                .iter()
                .any(|outer| outer.start_jd_ut <= w.start_jd_ut && w.end_jd_ut <= outer.end_jd_ut),
            "a lagna window lies inside a window of the wider search"
        );
    }
    // En pahalı durum: yalnızca lagna koşulu (hiçbir koşul aralığı daraltmaz).
    let only_lagna = MuhurtaQuery {
        criteria: MuhurtaCriteria {
            lagna_signs: vec![Sign::Leo],
            ..MuhurtaCriteria::default()
        },
        natal: None,
        ..query.clone()
    };
    let started = Instant::now();
    let (leo, times, _) = search_all(engine, &only_lagna, &s, now);
    let total = started.elapsed();
    let slowest = times.iter().max().copied().unwrap_or_default();
    eprintln!(
        "muhurta_search (366 days, lagna only) total {total:?}, slowest page {slowest:?}, {} windows",
        leo.len()
    );
    // Yıldız günü 23 sa 56 dk: 366 günde lagna Aslan'a 367 kez girer.
    assert!((366..=368).contains(&leo.len()), "{}", leo.len());
    assert!(total <= Duration::from_secs(5), "lagna only {total:?}");
    assert!(
        slowest <= Duration::from_millis(300),
        "slowest page {slowest:?}"
    );

    // Tarihsiz arama bugünden başlar ve 31 gün sürer (tek dilim).
    let default_range = MuhurtaQuery {
        from_date: None,
        to_date: None,
        ..only_lagna.clone()
    };
    let page = muhurta_search_blocking(engine, default_range, s.clone(), None, now).expect("page");
    assert_eq!((page.scanned, page.chunks, page.next), (1, 1, None));
    // 2026-11-26 00:00 +05:30.
    assert!((page.from_jd_ut - 2_461_370.270_833_333).abs() < 1e-6);
    assert!((page.to_jd_ut - page.from_jd_ut - 31.0).abs() < 1e-9);

    // Hatalar: 366 günü aşan aralık, ters aralık, kişisiz kişi koşulu.
    let field = |q: MuhurtaQuery| {
        let e = muhurta_search_blocking(engine, q, s.clone(), None, now).unwrap_err();
        assert_eq!(e.code, "invalid_muhurta_input");
        assert_eq!(e.params.len(), 1, "{:?}", e.params);
        e.params["field"].clone()
    };
    assert_eq!(
        field(MuhurtaQuery {
            to_date: Some("2027-01-02".into()),
            ..query.clone()
        }),
        "range"
    );
    assert_eq!(
        field(MuhurtaQuery {
            to_date: Some("2025-12-30".into()),
            ..query.clone()
        }),
        "range"
    );
    assert_eq!(
        field(MuhurtaQuery {
            natal: None,
            ..query.clone()
        }),
        "natal"
    );
    // Yer, tarihlerden önce denetlenir: geçersiz enlem ya da bilinmeyen dilim tarih verilsin
    // verilmesin yalnızca `place` alanıyla döner.
    for bad in [
        MuhurtaPlace {
            lat: 95.0,
            ..query.place.clone()
        },
        MuhurtaPlace {
            tz: TzChoice::Iana("Mars/Olympus".into()),
            ..query.place.clone()
        },
    ] {
        for (from_date, to_date) in [
            (None, None),
            (Some("2026-11-26".to_owned()), Some("2026-11-28".to_owned())),
        ] {
            assert_eq!(
                field(MuhurtaQuery {
                    from_date,
                    to_date,
                    place: bad.clone(),
                    ..query.clone()
                }),
                "place"
            );
        }
    }
}

/// Prashna (ileri modüller spec'i §11): gerçek ephemeris'le olgular, sayı prashnasının
/// değişmezi, v1 haritasıyla çapraz denetim, alan adlı hatalar ve bütçe.
fn prashna_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
    for (id, input) in inputs {
        let chart = compute_chart_blocking(
            engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        // `ChartInput`'un haritası, sayı yokken v1 haritasının aynısıdır.
        assert_eq!(
            compute_chart_for_blocking(engine, input.clone()).expect("chart for input"),
            chart,
            "{id}"
        );
        let positions = Positions::from(&chart);
        for focus in [None, Some(1), Some(7), Some(10)] {
            let facts = compute_prashna_facts_blocking(engine, input.clone(), focus)
                .unwrap_or_else(|e| panic!("{id}: {}", e.code));
            assert_eq!(facts.method, PRASHNA_METHOD, "{id}");
            // Çapraz denetim: saf fonksiyon v1 haritasının konumlarından aynı olguları verir.
            let mut pure =
                prashna_facts(&positions, &input.settings.combustion_orbs, focus).expect("pure");
            pure.warnings.clone_from(&facts.warnings);
            assert_eq!(facts, pure, "{id}");
            // v1 olgularıyla çapraz denetim: lordların burcu, evi, dignity'si, retrosu ve
            // yanması haritanın graha tablosundakiyle aynıdır (golden fixture'larla doğrulanmış).
            let lagna = facts.lagna.as_ref().expect("lagna");
            assert_eq!(lagna.sign, chart.lagna.sign, "{id}");
            assert_eq!(lagna.rising, rising_mode(chart.lagna.sign), "{id}");
            let v1 = |g: Graha| chart.grahas.iter().find(|p| p.graha == g).expect("graha");
            let mut lords = vec![&lagna.lord];
            if let Some(f) = &facts.focus {
                assert_eq!(Some(f.house), focus, "{id}");
                assert_eq!(f.lord.graha, house_lord(chart.lagna.sign, f.house), "{id}");
                lords.push(&f.lord);
            } else {
                assert_eq!(focus, None, "{id}");
            }
            for l in lords {
                let g = v1(l.graha);
                assert_eq!(
                    (l.sign, l.dignity, l.retrograde, l.combust),
                    (g.sign, g.dignity, g.retrograde, g.combust),
                    "{id}: {:?}",
                    l.graha
                );
                assert_eq!(l.house, g.sign.house_from(chart.lagna.sign), "{id}");
            }
            // Ay ve anın panchangı (v1) birbirini tutar.
            let moon = v1(Graha::Moon);
            assert_eq!(
                (facts.moon.sign, facts.moon.nakshatra, facts.moon.pada),
                (moon.sign, moon.nakshatra, moon.pada),
                "{id}"
            );
            assert_eq!(
                (facts.moon.tithi, facts.moon.paksha),
                (chart.panchang.tithi.no, chart.panchang.tithi.paksha),
                "{id}"
            );
            assert_eq!(facts.yogas, yogas(&positions), "{id}");
        }

        // Sayı prashnası: grahalar aynı anın grahalarıdır; yalnızca lagna ve evler değişir.
        for number in [1, 9, 10, 54, 108] {
            let numbered = ChartInput {
                lagna_number: Some(number),
                ..input.clone()
            };
            let c = compute_chart_for_blocking(engine, numbered.clone()).expect("number chart");
            let want = number_lagna_lon(number).expect("lagna");
            assert!(
                (c.lagna.sidereal_lon - want).abs() < 1e-12,
                "{id}: {number}"
            );
            assert_eq!(c.bhavas.system, HouseSystem::WholeSign, "{id}");
            for (a, b) in c.grahas.iter().zip(&chart.grahas) {
                assert_eq!(
                    (a.lon, a.sidereal_lon, a.speed, a.sign, a.nakshatra),
                    (b.lon, b.sidereal_lon, b.speed, b.sign, b.nakshatra),
                    "{id}: {number} {:?}",
                    a.graha
                );
                assert_eq!(a.house, a.sign.house_from(c.lagna.sign), "{id}");
            }
            assert_eq!(c.dasha, chart.dasha, "{id}");
            assert_eq!(c.panchang, chart.panchang, "{id}");
            // Sayı lagna verir, meridyen vermez: Shadbala ve Bhava bala sayı prashnasında yoktur.
            assert_eq!(
                compute_shadbala_blocking(engine, numbered.clone()).expect("shadbala"),
                None,
                "{id}: {number}"
            );
            let facts = compute_prashna_facts_blocking(engine, numbered.clone(), Some(7))
                .expect("number facts");
            assert_eq!(facts.lagna.as_ref().unwrap().sign, c.lagna.sign, "{id}");
            // Her modül aynı lagnayı görür (ör. bakış tablosu ve yoga listesi).
            let numbered_positions = Positions::from(&c);
            assert_eq!(facts.yogas, yogas(&numbered_positions), "{id}");
            assert_eq!(
                compute_yogas_blocking(engine, numbered).expect("yogas"),
                yogas(&numbered_positions),
                "{id}"
            );
        }
        // Gerçek lagna verilince (burç evi sistemi, sidereal kip) harita v1 haritasının aynısıdır.
        if input.settings.house_system == HouseSystem::WholeSign
            && input.settings.zodiac == ZodiacMode::Sidereal
        {
            let (jd, lat, lon, s) = (input.jd_ut, input.lat, input.lon, input.settings.clone());
            let lagna = chart.lagna.sidereal_lon;
            let again = engine
                .call(move |se: &mut ephemeris::SwissEph| {
                    jyotish_core::compute_chart_with_lagna(se, jd, lat, lon, &s, lagna)
                })
                .expect("engine")
                .expect("chart");
            assert_eq!(again, chart, "{id}");
        }
    }

    // Hatalar yalnızca alan adı taşır; odak evi ve sayı konumdan önce denetlenir.
    let (_, first) = &inputs[0];
    let bad_place = ChartInput {
        lat: 95.0,
        ..first.clone()
    };
    for (input, focus, field) in [
        (first.clone(), Some(0), "focus_house"),
        (first.clone(), Some(13), "focus_house"),
        (bad_place.clone(), Some(13), "focus_house"),
        (
            ChartInput {
                lagna_number: Some(0),
                ..bad_place.clone()
            },
            None,
            "number",
        ),
        (
            ChartInput {
                lagna_number: Some(109),
                ..first.clone()
            },
            Some(3),
            "number",
        ),
    ] {
        let e = compute_prashna_facts_blocking(engine, input, focus).unwrap_err();
        assert_eq!(e.code, "invalid_prashna_input");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], field);
    }
    // Saat bilinmiyorsa lagnaya bağlı bölümler boştur (K2).
    let unknown = ChartInput {
        time_known: false,
        ..first.clone()
    };
    let facts = compute_prashna_facts_blocking(engine, unknown, Some(5)).expect("facts");
    assert!(facts.lagna.is_none() && facts.focus.is_none() && facts.moon.house.is_none());

    // Bütçe: spec §15 `compute_prashna_facts` ≤ 15 ms (temel harita hesabı dahil).
    let numbered = ChartInput {
        lagna_number: Some(54),
        ..first.clone()
    };
    let (median, max) = median_of_15(|| {
        compute_prashna_facts_blocking(engine, numbered.clone(), Some(7)).expect("facts");
    });
    eprintln!("compute_prashna_facts median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(15),
        "compute_prashna_facts median {median:?}"
    );
}
