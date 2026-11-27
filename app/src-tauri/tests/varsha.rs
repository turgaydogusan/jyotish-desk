//! Varshaphala komut gövdeleri, gerçek `Engine` ile uçtan uca (yol haritası §4.14; ileri modüller
//! spec'i §13 "Komut katmanı", §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Ayrı dosyadır: `tests/modules.rs` paralel planlarla paylaşılmaz.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::varsha::{
    mudda_first_lord, muntha_sign, VarshaYear, MAX_VARSHA_AGE, SIDEREAL_YEAR_DAYS, VARSHA_METHOD,
};
use jyotish_core::{
    angular_distance, event_search, CalcSettings, ChartInput, EventSpec, Graha, HouseSystem,
    Positions, SkyEventKind,
};
use jyotish_desk_lib::client_commands::create_client_blocking;
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::varsha_commands::{
    compute_varsha_blocking, save_varsha_chart_blocking, VARSHA_LABEL,
};
use serde_json::Value;
use store::{
    BirthData, ChartKind, HomePlace, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice,
};

/// Bir yay saniyesi, derece.
const ARCSEC: f64 = 1.0 / 3600.0;
/// Bir saniye, gün.
const SECOND: f64 = 1.0 / 86_400.0;

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

/// Girdinin kendi yeri, UTC ofsetiyle.
fn birth_place(input: &ChartInput) -> MuhurtaPlace {
    MuhurtaPlace {
        lat: input.lat,
        lon: input.lon,
        tz: TzChoice::Fixed(0),
    }
}

/// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0 (deneme verisi).
fn delhi() -> BirthData {
    BirthData {
        date: "2000-01-01".into(),
        time: Some("17:30:00".into()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: "New Delhi".into(),
            country: "India".into(),
            admin1: "Delhi".into(),
            lat: 28.6139,
            lon: 77.2090,
            geoname_id: Some(1_261_481),
        },
        tz: TzChoice::Iana("Asia/Kolkata".into()),
        fold: None,
        notes: String::new(),
    }
}

#[test]
fn varsha_commands_run_on_the_engine_thread_and_map_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    // Fikstürün sabit saati: 2026-11-25 10:00 UTC.
    let now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
    for (id, input) in &inputs {
        let natal = compute_chart_blocking(
            &engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("natal chart");
        let sun = natal.grahas[Graha::Sun as usize].sidereal_lon;
        let moon_nakshatra = natal.grahas[Graha::Moon as usize].nakshatra;
        let mut previous_end = None;
        for age in [0_u8, 1, 2, 30, 80] {
            let view =
                compute_varsha_blocking(&engine, input.clone(), Some(age), birth_place(input), now)
                    .unwrap_or_else(|e| panic!("{id} age {age}: {}", e.code));
            let r = &view.report;
            assert_eq!((r.age, r.time_known), (age, true), "{id}");
            assert_eq!(r.method, VARSHA_METHOD, "{id}");
            assert_eq!(view.offset_secs, 0, "{id}: a fixed UTC place");

            // Dönüş anı (±1″): yıllık haritanın Güneş'i doğumdaki sidereal boylamındadır.
            let chart = r.chart.as_ref().expect("annual chart");
            assert_eq!(chart.jd_ut, r.start_jd_ut, "{id} age {age}");
            assert_eq!(chart.meta.settings, input.settings, "{id}");
            let back = chart.grahas[Graha::Sun as usize].sidereal_lon;
            assert!(
                angular_distance(back, sun) < ARCSEC,
                "{id} age {age}: the Sun is at {back}, born at {sun}"
            );
            // Yıl bir sidereal yıl sürer (gerçek Güneş'te birkaç dakika oynar).
            let days = r.end_jd_ut - r.start_jd_ut;
            assert!(
                (days - SIDEREAL_YEAR_DAYS).abs() < 0.02,
                "{id} age {age}: {days} days"
            );
            if age == 0 {
                assert_eq!(r.start_jd_ut, input.jd_ut, "{id}: the birth year");
            }
            // Ardışık yıllar boşluksuzdur.
            if let (Some(end), 1 | 2) = (previous_end, age) {
                assert_eq!(r.start_jd_ut, end, "{id} age {age}");
            }
            previous_end = Some(r.end_jd_ut);

            // Muntha ve lagna lordları: iki haritanın lagnasından.
            let facts = r.facts.expect("facts");
            assert_eq!(
                facts.muntha.sign,
                muntha_sign(natal.lagna.sign, age),
                "{id}"
            );
            assert_eq!(facts.muntha.natal_house, age % 12 + 1, "{id}");
            assert_eq!(
                (facts.lagna_sign, facts.natal_lagna_sign),
                (chart.lagna.sign, natal.lagna.sign),
                "{id}"
            );
            // Mudda dasa: ilk lord kuraldan; dokuz dönem yılı boşluksuz doldurur.
            assert_eq!(r.mudda.birth_nakshatra, moon_nakshatra, "{id}");
            assert_eq!(
                r.mudda.first_lord,
                mudda_first_lord(moon_nakshatra, age),
                "{id}"
            );
            let top: Vec<_> = r.mudda.periods.iter().filter(|p| p.level == 1).collect();
            assert_eq!((top.len(), r.mudda.periods.len()), (9, 90), "{id}");
            assert_eq!(
                (top[0].start_jd_ut, top[8].end_jd_ut),
                (r.start_jd_ut, r.end_jd_ut),
                "{id}"
            );
        }

        // İkinci yazım: Plan 22'nin olay araması aynı dönüşü `TransitToNatal` olarak bulur
        // (sabit adımlı yürüyücü, 0,05 sn çözünürlük). İki bağımsız arama bir saniyede buluşur.
        let view =
            compute_varsha_blocking(&engine, input.clone(), Some(30), birth_place(input), now)
                .expect("age 30");
        let start = view.report.start_jd_ut;
        let (settings, positions) = (input.settings.clone(), Positions::from(&natal));
        let events = engine
            .call(move |se: &mut SwissEph| {
                let spec = EventSpec {
                    from_jd_ut: start - 5.0,
                    to_jd_ut: start + 5.0,
                    kinds: vec![SkyEventKind::TransitToNatal],
                    grahas: vec![Graha::Sun],
                };
                event_search(se, &settings, &spec, Some(&positions))
            })
            .expect("engine")
            .expect("event search");
        let returns: Vec<f64> = events
            .iter()
            .filter(|e| e.grahas == [Graha::Sun, Graha::Sun])
            .map(|e| e.jd_ut)
            .collect();
        assert_eq!(returns.len(), 1, "{id}: one return in ten days");
        assert!(
            (returns[0] - start).abs() < SECOND,
            "{id}: event search {} vs {start}",
            returns[0]
        );

        // Ev sistemi dönüş anını, munthayı ve mudda dasayı değiştirmez (burç evleri, K4).
        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
            let other = ChartInput {
                settings: CalcSettings {
                    house_system,
                    ..input.settings.clone()
                },
                ..input.clone()
            };
            let o = compute_varsha_blocking(&engine, other, Some(30), birth_place(input), now)
                .expect("house system")
                .report;
            assert_eq!(
                (o.start_jd_ut, o.end_jd_ut, o.facts, &o.mudda),
                (
                    view.report.start_jd_ut,
                    view.report.end_jd_ut,
                    view.report.facts,
                    &view.report.mudda
                ),
                "{id}: {house_system:?}"
            );
        }

        // Doğum saati bilinmiyorsa yıllık harita ve muntha yoktur; yıl ve mudda dasa aynıdır.
        let unknown = compute_varsha_blocking(
            &engine,
            ChartInput {
                time_known: false,
                ..input.clone()
            },
            Some(30),
            birth_place(input),
            now,
        )
        .expect("time unknown")
        .report;
        assert!(!unknown.time_known, "{id}");
        assert_eq!((&unknown.chart, unknown.facts), (&None, None), "{id}");
        assert_eq!(
            (unknown.start_jd_ut, &unknown.mudda),
            (view.report.start_jd_ut, &view.report.mudda),
            "{id}"
        );
    }

    let delhi_input = inputs
        .iter()
        .find(|(id, _)| id.starts_with("new-delhi"))
        .map(|(_, input)| input.clone())
        .expect("new-delhi golden");
    let delhi_place = MuhurtaPlace {
        lat: delhi_input.lat,
        lon: delhi_input.lon,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    };
    // Yaş verilmezse `now` anında süren yıl: 2000-01-01 doğumlu harita 2026-11-25'te 26 yılını
    // tamamlamıştır. Ofset yerin dilimindendir (IST, +05:30).
    let running =
        compute_varsha_blocking(&engine, delhi_input.clone(), None, delhi_place.clone(), now)
            .expect("running year");
    assert_eq!((running.report.age, running.offset_secs), (26, 19_800));
    let now_jd = now.as_second() as f64 / 86_400.0 + 2_440_587.5;
    assert!(running.report.start_jd_ut <= now_jd && now_jd < running.report.end_jd_ut);
    let by_age = compute_varsha_blocking(
        &engine,
        delhi_input.clone(),
        Some(26),
        delhi_place.clone(),
        now,
    )
    .expect("age 26");
    assert_eq!(running, by_age);
    // Çekirdeğin `At` yolu komutun `now` yoluyla aynı yılı verir.
    let (i, p) = (delhi_input.clone(), (delhi_input.lat, delhi_input.lon));
    let core = engine
        .call(move |se: &mut SwissEph| {
            jyotish_core::varsha::compute_varsha(se, &i, VarshaYear::At(now_jd), Some(p))
        })
        .expect("engine")
        .expect("core");
    assert_eq!(core, running.report);

    // Başka yer: dönüş anı aynıdır, yıllık harita o yerdedir ve lagnası başkadır.
    let sydney = MuhurtaPlace {
        lat: -33.8688,
        lon: 151.2093,
        tz: TzChoice::Iana("Australia/Sydney".into()),
    };
    let moved = compute_varsha_blocking(&engine, delhi_input.clone(), Some(26), sydney, now)
        .expect("moved");
    assert_eq!(moved.report.start_jd_ut, by_age.report.start_jd_ut);
    assert_eq!(moved.report.mudda, by_age.report.mudda);
    let (here, there) = (
        by_age.report.chart.as_ref().expect("chart"),
        moved.report.chart.as_ref().expect("chart"),
    );
    assert_eq!((there.geo_lat, there.geo_lon), (-33.8688, 151.2093));
    assert_ne!(there.lagna.sidereal_lon, here.lagna.sidereal_lon);
    // Sidney yaz saatinde: +11:00.
    assert_eq!(moved.offset_secs, 39_600);

    // Hatalar: `compute_chart`'ın kodları; yaş `invalid_dasha_input`, çözülemeyen dilim
    // `invalid_input`. Hiçbiri yeni bir koddur.
    let code = |input: ChartInput, age: Option<u8>, place: MuhurtaPlace| {
        compute_varsha_blocking(&engine, input, age, place, now)
            .unwrap_err()
            .code
    };
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(MAX_VARSHA_AGE + 1),
            delhi_place.clone()
        ),
        "invalid_dasha_input"
    );
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(26),
            MuhurtaPlace {
                lat: 91.0,
                ..delhi_place.clone()
            }
        ),
        "invalid_location"
    );
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(26),
            MuhurtaPlace {
                tz: TzChoice::Iana("Nowhere/Nothing".into()),
                ..delhi_place.clone()
            }
        ),
        "invalid_input"
    );
    assert_eq!(
        code(
            ChartInput {
                settings: CalcSettings {
                    schema_version: 1,
                    ..CalcSettings::default()
                },
                ..delhi_input.clone()
            },
            Some(26),
            delhi_place.clone()
        ),
        "unsupported_schema_version"
    );
    assert_eq!(
        code(
            ChartInput {
                lagna_number: Some(0),
                ..delhi_input.clone()
            },
            Some(26),
            delhi_place.clone()
        ),
        "invalid_prashna_input"
    );
    // Sayı prashnası: muntha sayının lagnasından sayılır (30 → Yengeç; 26 = 2 × 12 + 2: Başak).
    let numbered = compute_varsha_blocking(
        &engine,
        ChartInput {
            lagna_number: Some(30),
            ..delhi_input.clone()
        },
        Some(26),
        delhi_place.clone(),
        now,
    )
    .expect("number prashna")
    .report;
    assert_eq!(
        numbered.facts.expect("facts").muntha.sign,
        jyotish_core::Sign::Virgo
    );

    // Olay haritası olarak kaydetme: var olan `add_chart` yolu, kaynak haritanın ayarlarıyla.
    let clock = Box::new(|| Timestamp::from_second(1_795_600_800).expect("timestamp"));
    let store = Mutex::new(Store::open_in_memory().expect("store").with_clock(clock));
    let new_client = |name: &str, birth: BirthData| NewClient {
        name: name.into(),
        gender: None,
        chart: NewChart {
            label: String::new(),
            kind: ChartKind::Natal,
            birth,
        },
    };
    let opened = create_client_blocking(&engine, &store, new_client("Örnek Danışan", delhi()))
        .expect("create client");
    // Kaynak haritaya varsayılandan ayrı bir ayar yazılır: kopyalandığı görülsün.
    let sripati = CalcSettings {
        house_system: HouseSystem::Sripati,
        ..opened.calc_settings.clone()
    };
    store
        .lock()
        .expect("store")
        .save_calc_settings(opened.chart_record.id, &sripati)
        .expect("save settings");
    let saved = save_varsha_chart_blocking(&engine, &store, opened.chart_record.id, 26, false)
        .expect("save");
    assert_eq!(saved.kind, ChartKind::Event);
    assert_eq!(saved.client_id, opened.client.id);
    assert_eq!(saved.birth.place, opened.chart_record.birth.place);
    assert_eq!(saved.birth.tz, opened.chart_record.birth.tz);
    assert_eq!(saved.birth.time_certainty, TimeCertainty::Exact);
    assert_eq!(saved.label, format!("{VARSHA_LABEL} {}", saved.birth.date));
    // 26 sidereal yıl 9496,665 gündür; 2000-01-01'den 2026-01-01'e 9497 gün vardır: dönüş doğum
    // saatinden (12:00 UT) yaklaşık sekiz saat öncedir, 04 UT dolayında; Delhi'de 09 suları.
    assert_eq!(saved.birth.date, "2026-01-01");
    let clock = saved.birth.time.as_deref().expect("a time");
    assert!(clock.starts_with("09:") && clock.len() == 8, "{clock}");
    // Kayıt saniyeye yuvarlanır: çözülen an dönüş anından yarım saniyeden çok ayrışmaz.
    assert!(
        (saved.resolved.jd_ut - by_age.report.start_jd_ut).abs() <= 0.5 * SECOND + 1e-9,
        "{} vs {}",
        saved.resolved.jd_ut,
        by_age.report.start_jd_ut
    );
    {
        let s = store.lock().expect("store");
        assert_eq!(
            s.calc_settings(saved.id).expect("settings").settings(),
            &sripati
        );
        assert_eq!(s.charts_of(opened.client.id).expect("charts").len(), 2);
        // Harita açılmaz: danışanın son haritası kaynak haritadır.
        assert_eq!(
            s.client(opened.client.id).expect("client").last_chart_id,
            Some(opened.chart_record.id)
        );
    }
    let save_code = |chart_id: u32, age: u8, home: bool| {
        save_varsha_chart_blocking(&engine, &store, chart_id, age, home)
            .unwrap_err()
            .code
    };
    // Bulunulan yer seçilmemiş; olmayan harita; sınır dışı yaş.
    assert_eq!(save_code(opened.chart_record.id, 26, true), "invalid_input");
    assert_eq!(save_code(9_999, 26, false), "not_found");
    assert_eq!(
        save_code(opened.chart_record.id, MAX_VARSHA_AGE + 1, false),
        "invalid_dasha_input"
    );
    // Bulunulan yer seçilince yıllık harita orada, oranın duvar saatiyle kaydedilir.
    {
        let s = store.lock().expect("store");
        let mut prefs = s.app_prefs().expect("prefs");
        prefs.home_place = Some(HomePlace {
            place: Place {
                name: "Sydney".into(),
                country: "Australia".into(),
                admin1: "New South Wales".into(),
                lat: -33.8688,
                lon: 151.2093,
                geoname_id: None,
            },
            tz: TzChoice::Iana("Australia/Sydney".into()),
        });
        s.set_app_prefs(&prefs).expect("set prefs");
    }
    let at_home = save_varsha_chart_blocking(&engine, &store, opened.chart_record.id, 26, true)
        .expect("save at home");
    assert_eq!(at_home.birth.place.name, "Sydney");
    assert_eq!(at_home.birth.tz, TzChoice::Iana("Australia/Sydney".into()));
    assert!((at_home.resolved.jd_ut - saved.resolved.jd_ut).abs() <= SECOND);
    assert_eq!(at_home.resolved.offset_secs, 39_600);
    assert_eq!(at_home.birth.date, "2026-01-01");
    assert!(at_home
        .birth
        .time
        .as_deref()
        .is_some_and(|t| t.starts_with("14:")));
    // Doğum saati bilinmeyen haritanın yıllık haritası yoktur: kaydedilmez.
    let mut no_time = delhi();
    no_time.time = None;
    no_time.time_certainty = TimeCertainty::Unknown;
    let unknown = create_client_blocking(&engine, &store, new_client("Saatsiz Danışan", no_time))
        .expect("create client without a birth time");
    let err = save_varsha_chart_blocking(&engine, &store, unknown.chart_record.id, 26, false)
        .unwrap_err();
    assert_eq!(
        (
            err.code.as_str(),
            err.params.get("field").map(String::as_str)
        ),
        ("invalid_input", Some("birth.time"))
    );
    assert_eq!(
        store
            .lock()
            .expect("store")
            .charts_of(unknown.client.id)
            .expect("charts")
            .len(),
        1
    );

    // Performans: doğum haritası, iki dönüş araması ve yıllık harita dahil ≤ 20 ms.
    let (median, max) = median_of_15(|| {
        compute_varsha_blocking(
            &engine,
            delhi_input.clone(),
            Some(26),
            delhi_place.clone(),
            now,
        )
        .expect("varsha");
    });
    eprintln!("compute_varsha median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(20),
        "compute_varsha median {median:?}"
    );
}
