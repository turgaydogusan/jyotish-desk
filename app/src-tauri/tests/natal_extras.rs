//! Doğum anı eklerinin komut gövdeleri, gerçek `Engine` ile uçtan uca (yol haritası Plan 23;
//! ileri modüller spec'i §13 "Komut katmanı"). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Çapraz denetimler v1'de golden fixture'larla doğrulanmış haritaya (tithi, nitya
//! yoga, nakshatra, varga burçları) ve Shadbala'nın saptavargaja kalemine dayanır.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::{Engine, SwissEph};
use jyotish_core::dignity::{own_signs, sign_lord};
use jyotish_core::{
    compute_positions, compute_transits, nakshatra_lord, nakshatra_pada, normalize,
    saptavargaja_points, tara_bala, tara_cycle, CalcSettings, Chart, ChartInput, Graha, Sign,
    Varga, VargaScheme, ZodiacMode, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD, SAPTAVARGA,
    VIMSHOPAKA_GRAHAS, VIMSHOPAKA_METHOD,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::natal_extras_commands::{
    compute_navatara_blocking, compute_panchang_extras_blocking, compute_vimshopaka_blocking,
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
                lagna_number: None,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Her golden an iki zodyak kipinde.
fn cases() -> Vec<(String, ChartInput)> {
    let mut out = Vec::new();
    for (id, input) in golden_inputs() {
        let tropical = ChartInput {
            settings: CalcSettings {
                zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
                ..input.settings.clone()
            },
            ..input.clone()
        };
        out.push((format!("{id} (tropical rasi)"), tropical));
        out.push((id, input));
    }
    out
}

fn chart_of(engine: &Engine, input: &ChartInput) -> Chart {
    compute_chart_blocking(
        engine,
        input.jd_ut,
        input.lat,
        input.lon,
        input.settings.clone(),
    )
    .expect("chart")
}

fn unknown_time(input: &ChartInput) -> ChartInput {
    ChartInput {
        time_known: false,
        ..input.clone()
    }
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

/// Motoru tek test açar; modül başına denetimler ayrı fonksiyonlardadır.
#[test]
fn natal_extras_commands_run_on_the_engine_thread_and_agree_with_the_chart() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let cases = cases();
    assert!(cases.len() >= 10, "{} cases", cases.len());
    panchang_extras_checks(&engine, &cases);
    navatara_checks(&engine, &cases);
    vimshopaka_checks(&engine, &cases);
    error_and_budget_checks(&engine, &cases);
}

fn panchang_extras_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let extras = compute_panchang_extras_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let chart = chart_of(engine, input);
        assert_eq!(extras.method, PANCHANG_EXTRAS_METHOD, "{id}");
        assert!(!extras.approximate, "{id}");
        // Tithi, v1'in panchangındaki tithidir.
        assert_eq!(extras.tithi, chart.panchang.tithi.no, "{id}");
        // Yogi nakshatrası nitya yogadan yedi ileridedir (27'de sarılır).
        let yogi = &extras.yogi;
        assert_eq!(
            yogi.nakshatra,
            (chart.panchang.yoga.no + 6) % 27 + 1,
            "{id}"
        );
        assert_eq!(
            nakshatra_pada(yogi.point_lon),
            (yogi.nakshatra, yogi.pada),
            "{id}"
        );
        assert_eq!(yogi.yogi, nakshatra_lord(yogi.nakshatra), "{id}");
        // Nokta, haritanın Güneş ve Ay sidereal boylamlarından.
        let sum = chart.grahas[Graha::Sun as usize].sidereal_lon
            + chart.grahas[Graha::Moon as usize].sidereal_lon;
        assert!(
            (yogi.point_lon - normalize(sum + 280.0 / 3.0)).abs() < 1e-9,
            "{id}"
        );
        // Burç rasi çerçevesindedir: tropikal-rasi kipinde nokta ayanamsa kadar ötelenir.
        let frame = match input.settings.zodiac {
            ZodiacMode::Sidereal => yogi.point_lon,
            ZodiacMode::TropicalRasiSiderealNakshatra => {
                normalize(yogi.point_lon + chart.ayanamsa_deg)
            }
        };
        // Burç sınırına bir yay saniyesinden yakın nokta ayrıca denetlenmez.
        let edge = frame % 30.0;
        if (1.0 / 3600.0..30.0 - 1.0 / 3600.0).contains(&edge) {
            assert_eq!(yogi.sign, Sign::of(frame), "{id}");
            assert!((yogi.deg_in_sign - edge).abs() < 1e-6, "{id}");
        }
        assert_eq!(yogi.co_yogi, sign_lord(yogi.sign), "{id}");
        // Avayogi: altıncı nakshatra; ikinci yazımla (nokta + 186°40′) aynı lord.
        assert_eq!(
            yogi.avayogi_nakshatra,
            (yogi.nakshatra + 4) % 27 + 1,
            "{id}"
        );
        assert_eq!(yogi.avayogi, nakshatra_lord(yogi.avayogi_nakshatra), "{id}");
        let (other, _) = nakshatra_pada(yogi.point_lon + 560.0 / 3.0);
        assert_eq!(nakshatra_lord(other), yogi.avayogi, "{id}");
        // Dagdha rasi: dolunay ve yeni ayda yok, chaturdashide dört, ötekilerde iki burç.
        let expected = match (extras.tithi - 1) % 15 + 1 {
            15 => 0,
            14 => 4,
            _ => 2,
        };
        assert_eq!(extras.dagdha_rasis.len(), expected, "{id}");
        // Doğum saati bilinmiyor: aynı an için aynı değerler, yaklaşık işaretiyle.
        let unknown = compute_panchang_extras_blocking(engine, unknown_time(input)).expect(id);
        assert!(unknown.approximate, "{id}");
        assert_eq!(
            (&unknown.yogi, unknown.tithi, &unknown.dagdha_rasis),
            (&extras.yogi, extras.tithi, &extras.dagdha_rasis),
            "{id}"
        );
    }
    // Delhi 2000-01-01 (tests/golden/new-delhi-2000-01-01.json: tithi 26, nitya yoga 8 Dhriti):
    // yogi nakshatrası 8 + 7 = 15 (Swati), lordu Rahu; avayogi nakshatrası 20 (Purva Ashadha),
    // lordu Venüs; Krishna Ekadashi (paksha içi 11): Yay ve Balık.
    let (_, delhi) = cases
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi");
    let extras = compute_panchang_extras_blocking(engine, delhi.clone()).expect("delhi");
    assert_eq!(extras.tithi, 26);
    assert_eq!((extras.yogi.nakshatra, extras.yogi.yogi), (15, Graha::Rahu));
    assert_eq!(
        (extras.yogi.avayogi_nakshatra, extras.yogi.avayogi),
        (20, Graha::Venus)
    );
    assert_eq!(extras.dagdha_rasis, [Sign::Sagittarius, Sign::Pisces]);
}

fn navatara_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let chart = chart_of(engine, input);
        let natal = compute_navatara_blocking(engine, input.clone(), None)
            .expect("navatara without transit");
        assert_eq!(natal.method, NAVATARA_METHOD, "{id}");
        assert_eq!(
            (natal.transit.as_ref(), natal.transit_jd_ut),
            (None, None),
            "{id}"
        );
        let janma = chart.grahas[Graha::Moon as usize].nakshatra;
        assert_eq!(natal.janma_nakshatra, janma, "{id}");
        assert_eq!(natal.rows.len(), 27, "{id}");
        for (i, row) in natal.rows.iter().enumerate() {
            assert_eq!(
                usize::from(row.nakshatra),
                (usize::from(janma) - 1 + i) % 27 + 1
            );
            assert_eq!(row.tara, tara_bala(janma, row.nakshatra), "{id} row {i}");
            assert_eq!(usize::from(row.tara.number), i % 9 + 1, "{id} row {i}");
            assert_eq!(usize::from(row.cycle), i / 9 + 1, "{id} row {i}");
            assert_eq!(row.lord, nakshatra_lord(row.nakshatra), "{id} row {i}");
            // Satırdaki grahalar, haritada o nakshatrada duranlardır.
            let expected: Vec<Graha> = chart
                .grahas
                .iter()
                .filter(|g| g.nakshatra == row.nakshatra)
                .map(|g| g.graha)
                .collect();
            assert_eq!(row.grahas, expected, "{id} row {i}");
        }
        // Geçiş anı doğum anının kendisiyse her graha kendi satırının tarasını alır.
        let same = compute_navatara_blocking(engine, input.clone(), Some(input.jd_ut))
            .expect("navatara at the birth moment");
        assert_eq!(same.rows, natal.rows, "{id}");
        for t in same.transit.as_ref().expect("transit") {
            let g = &chart.grahas[t.graha as usize];
            assert_eq!(
                (t.nakshatra, t.pada),
                (g.nakshatra, g.pada),
                "{id} {:?}",
                t.graha
            );
            let row = natal
                .rows
                .iter()
                .find(|r| r.nakshatra == t.nakshatra)
                .expect("row");
            assert_eq!(
                (t.tara, t.cycle),
                (row.tara, row.cycle),
                "{id} {:?}",
                t.graha
            );
        }
        // Başka bir an (1 000 gün sonra): nakshatralar transit komutunun verdiğiyle aynıdır.
        let later = input.jd_ut + 1000.0;
        let moved = compute_navatara_blocking(engine, input.clone(), Some(later))
            .expect("navatara at a later moment");
        assert_eq!(moved.transit_jd_ut, Some(later), "{id}");
        let snapshot = {
            let (settings, bhavas) = (input.settings.clone(), chart.bhavas.clone());
            engine
                .call(move |se: &mut SwissEph| compute_transits(se, later, &settings, &bhavas))
                .expect("engine")
                .expect("transits")
        };
        let transit = moved.transit.expect("transit");
        assert_eq!(transit.len(), 9, "{id}");
        for (t, g) in transit.iter().zip(&snapshot.grahas) {
            assert_eq!(
                (t.graha, t.nakshatra, t.pada),
                (g.graha, g.nakshatra, g.pada),
                "{id}"
            );
            assert_eq!(t.tara, tara_bala(janma, g.nakshatra), "{id}");
            assert_eq!(t.cycle, tara_cycle(janma, g.nakshatra), "{id}");
        }
        // Doğum saati bilinmiyor: tablo aynı an için aynıdır, yaklaşık işaretiyle.
        let unknown =
            compute_navatara_blocking(engine, unknown_time(input), None).expect("time unknown");
        assert!(unknown.approximate && !natal.approximate, "{id}");
        assert_eq!(unknown.rows, natal.rows, "{id}");
    }
}

fn vimshopaka_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let chart = chart_of(engine, input);
        let report = compute_vimshopaka_blocking(engine, input.clone()).expect("vimshopaka");
        assert_eq!(report.method, VIMSHOPAKA_METHOD, "{id}");
        assert!(!report.approximate, "{id}");
        let order: Vec<Graha> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, VIMSHOPAKA_GRAHAS, "{id}");
        let positions = {
            let input = input.clone();
            engine
                .call(move |se: &mut SwissEph| compute_positions(se, &input))
                .expect("engine")
                .expect("positions")
        };
        for row in &report.grahas {
            assert_eq!(row.vargas.len(), 16, "{id}");
            for (v, varga_chart) in row.vargas.iter().zip(&chart.vargas) {
                // Varga burcu, v1'in on altı varga haritasındaki burçtur.
                assert_eq!(v.varga, varga_chart.varga, "{id}");
                let placed = &varga_chart.placements[row.graha as usize];
                assert_eq!(
                    (placed.graha, placed.sign),
                    (row.graha, v.sign),
                    "{id} {:?}",
                    v.varga
                );
                assert_eq!(
                    v.points == 20.0,
                    own_signs(row.graha).contains(&v.sign),
                    "{id} {:?} {:?}",
                    row.graha,
                    v.varga
                );
            }
            // Çapraz denetim: Shadbala'nın saptavargaja kalemi aynı ilişkiyi başka puanlarla
            // okur (kendi burcu 30, çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2;
            // D1'de moolatrikona 45). Yedi vargada basamaklar birebir eşleşmelidir.
            for varga in SAPTAVARGA {
                let v = row.vargas.iter().find(|v| v.varga == varga).expect("varga");
                let sapta = saptavargaja_points(row.graha, varga, &positions);
                // Moolatrikona (45; yalnızca D1) Vimshopaka'da ayrı basamak değildir: eşlenmez.
                // Başka hiçbir puan eşlemesiz geçemez (tanınmayan puan testi düşürür).
                if sapta == 45.0 {
                    assert_eq!(varga, Varga::D1, "{id} {:?}", row.graha);
                    continue;
                }
                let expected = [
                    (30.0, 20.0),
                    (20.0, 18.0),
                    (15.0, 15.0),
                    (10.0, 10.0),
                    (4.0, 7.0),
                    (2.0, 5.0),
                ]
                .into_iter()
                .find(|(points, _)| *points == sapta)
                .map(|(_, rung)| rung)
                .unwrap_or_else(|| panic!("{id} {:?} {varga:?}: saptavargaja {sapta}", row.graha));
                assert_eq!(v.points, expected, "{id} {:?} {varga:?}", row.graha);
            }
            let scores = [
                row.shadvarga,
                row.saptavarga,
                row.dashavarga,
                row.shodashavarga,
            ];
            for (scheme, score) in VargaScheme::ALL.into_iter().zip(scores) {
                let again: f64 = scheme
                    .weights()
                    .iter()
                    .map(|&(varga, w)| {
                        w * row
                            .vargas
                            .iter()
                            .find(|v| v.varga == varga)
                            .expect("varga")
                            .points
                    })
                    .sum::<f64>()
                    / 20.0;
                assert!(
                    (score - again).abs() < 1e-9,
                    "{id} {:?} {scheme:?}",
                    row.graha
                );
                assert!(
                    (5.0..=20.0).contains(&score),
                    "{id} {:?} {scheme:?}",
                    row.graha
                );
            }
            assert_eq!(row.vargas[0].varga, Varga::D1, "{id}");
            assert_eq!(
                row.vargas[0].sign, chart.grahas[row.graha as usize].sign,
                "{id}"
            );
        }
        let unknown =
            compute_vimshopaka_blocking(engine, unknown_time(input)).expect("time unknown");
        assert!(unknown.approximate, "{id}");
        assert_eq!(unknown.grahas, report.grahas, "{id}");
    }
}

fn error_and_budget_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    let (_, delhi) = cases
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi");
    // Hata eşlemesi: geçersiz konum haritanın hatasıdır; geçiş anı yalnızca alan adıyla
    // reddedilir (değer taşımaz) ve haritadan önce denetlenir.
    let mut bad = delhi.clone();
    bad.lat = 91.0;
    let e = compute_panchang_extras_blocking(engine, bad.clone()).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    let e = compute_vimshopaka_blocking(engine, bad.clone()).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    let e = compute_navatara_blocking(engine, bad.clone(), None).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    for jd in [f64::NAN, f64::INFINITY, 0.0, 1e9] {
        let e = compute_navatara_blocking(engine, bad.clone(), Some(jd))
            .expect_err("bad transit moment");
        assert_eq!(e.code, "invalid_transit_input");
        let params: Vec<(&str, &str)> = e
            .params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(params, [("field", "transit_jd_ut")]);
    }

    // Bütçeler (Plan 23 D17): temel harita hesabı dahil komut başına ≤ 20 ms, hata ayıklama
    // derlemesinde. Boş makinede ölçüm 1 ms'nin altındadır; pay, yan yana derleyen çalışma
    // ağaçlarının yükü içindir.
    let budget = Duration::from_millis(20);
    let (median, max) = median_of_15(|| {
        compute_panchang_extras_blocking(engine, delhi.clone()).expect("extras");
    });
    eprintln!("compute_panchang_extras: median {median:?}, max {max:?}");
    assert!(
        median <= budget,
        "compute_panchang_extras median {median:?}"
    );
    let (median, max) = median_of_15(|| {
        compute_navatara_blocking(engine, delhi.clone(), Some(delhi.jd_ut + 9000.0))
            .expect("navatara");
    });
    eprintln!("compute_navatara: median {median:?}, max {max:?}");
    assert!(median <= budget, "compute_navatara median {median:?}");
    let (median, max) = median_of_15(|| {
        compute_vimshopaka_blocking(engine, delhi.clone()).expect("vimshopaka");
    });
    eprintln!("compute_vimshopaka: median {median:?}, max {max:?}");
    assert!(median <= budget, "compute_vimshopaka median {median:?}");
}
