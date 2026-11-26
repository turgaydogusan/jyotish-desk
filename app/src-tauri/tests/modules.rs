//! İleri modül komut gövdeleri, gerçek `Engine` ile uçtan uca (ileri modüller spec'i §13
//! "Komut katmanı"; §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::Engine;
use jyotish_core::{
    chara_dasha, co_lords, graha_aspect_signs, house_lord, house_sign, jaimini_report,
    rashi_drishti_signs, sign_dasha_children, sign_dasha_path_at, sphuta_drishti, AdvancedSettings,
    CalcSettings, ChartInput, ChartWarning, Graha, KarakaRole, KarakaScheme, Positions, Sign,
    SignDashaPeriod, Varga, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
    SEVEN_KARAKA_ROLES,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::module_commands::{
    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
    compute_drishti_blocking, compute_jaimini_blocking,
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
