//! İleri modül komut gövdeleri, gerçek `Engine` ile uçtan uca (ileri modüller spec'i §13
//! "Komut katmanı"; §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac};
use jyotish_core::{
    chara_dasha, cheshta_from, co_lords, compute_panchang, compute_shadbala_astro,
    graha_aspect_signs, house_lord, house_sign, jaimini_report, kuja_dosha, mean_longitudes,
    rashi_drishti_signs, shadbala, sign_dasha_children, sign_dasha_path_at, sphuta_drishti,
    sun_entry_before, vara_lord, yoga_rule, yogas, AdvancedSettings, CalcSettings, ChartInput,
    ChartWarning, Dignity, Graha, HouseSystem, KarakaRole, KarakaScheme, Positions, Sign,
    SignDashaPeriod, Varga, YogaGroup, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD,
    JAIMINI_METHOD, MAX_SUNRISE_GAP_DAYS, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD,
    YOGA_CATALOG, YOGA_CATALOG_VERSION,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::module_commands::{
    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
    compute_drishti_blocking, compute_jaimini_blocking, compute_shadbala_blocking,
    compute_yogas_blocking,
};
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
