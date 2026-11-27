//! Sankranti listesi komutu, gerçek `Engine` ile uçtan uca (yol haritası Plan 26, §4.8) ve
//! arayüz testlerinin fikstürü: `app/ui/test/fixtures/sankranti.json`.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti`.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir. Çapraz denetimler ephemeris'in kendi
//! boylamına, olay aramasının Güneş burç girişlerine ve v1'in panchangına dayanır; beklenen
//! değerler başka bir yazılımdan alınmaz (K9).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Body, Engine, Ephemeris, SiderealMode, SwissEph, Zodiac};
use jyotish_core::{
    event_chunks, event_search_chunk, CalcSettings, ChartWarning, EventSpec, Graha, Sign,
    SkyEventKind, ZodiacMode, SANKRANTI_METHOD,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::transit_tools_commands::sankranti::{
    sankranti_list_blocking, SankrantiListView,
};
use serde_json::{json, Value};
use store::TzChoice;

/// Bütçe (D17): bir yılın listesi, hata ayıklama derlemesinde.
const BUDGET: Duration = Duration::from_millis(100);
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

fn delhi() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 28.6139,
        lon: 77.2090,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    }
}

/// Kutup dairesinin kuzeyi: aralık ortasında gün doğmaz (tests/golden/tromso-2050-12-21.json yeri).
fn tromso() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 69.6492,
        lon: 18.9553,
        tz: TzChoice::Iana("Europe/Oslo".into()),
    }
}

/// 2026-11-24T00:00:00Z.
fn now() -> Timestamp {
    Timestamp::from_second(1_795_478_400).expect("timestamp")
}

/// Anın yerdeki takvim günü (`YYYY-MM-DD`), satırın kendi ofsetiyle.
fn local_date(jd_ut: f64, offset_secs: i32) -> String {
    let secs = ((jd_ut - UNIX_EPOCH_JD) * 86_400.0).floor() as i64 + i64::from(offset_secs);
    Timestamp::from_second(secs)
        .expect("timestamp")
        .strftime("%Y-%m-%d")
        .to_string()
}

fn list(
    engine: &Engine,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> SankrantiListView {
    sankranti_list_blocking(engine, year, place, settings, now()).expect("sankranti list")
}

#[test]
fn sankranti_command_runs_on_the_engine_thread_and_maps_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let settings = CalcSettings::default();

    let started = Instant::now();
    let view = list(&engine, Some(2026), delhi(), settings.clone());
    let spent = started.elapsed();
    eprintln!("sankranti_list(2026): {spent:?}");
    assert!(spent <= BUDGET, "sankranti_list took {spent:?}");

    assert_eq!((view.year, view.method.as_str()), (2026, SANKRANTI_METHOD));
    assert!(view.warnings.is_empty());
    // Yıl yerin diliminde 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a: 2026 artık yıl değildir.
    assert!((view.to_jd_ut - view.from_jd_ut - 365.0).abs() < 1e-9);
    assert_eq!(local_date(view.from_jd_ut, 19_800), "2026-01-01");

    // On iki giriş, burç sırasıyla; yılın ilki Oğlak'tır (Makara). Türetme: Güneş tropikal
    // 270°'ye Aralık gündönümünde (21–22 Aralık) varır; Lahiri ayanamsası 2026'da yaklaşık
    // 24,2°'dir, yani sidereal 270° tropikal 294,2°'dir. Güneş Ocak'ta günde yaklaşık 1,02°
    // yürür: 24,2 / 1,02 ≈ 23,7 gün → 14 Ocak dolayı. Pencere: 13–16 Ocak.
    assert_eq!(view.rows.len(), 12);
    for (k, row) in view.rows.iter().enumerate() {
        assert_eq!(
            row.sankranti.sign,
            Sign::from_index((Sign::Capricorn.index() + k) % 12),
            "row {k}"
        );
        assert_eq!(row.offset_secs, 19_800, "row {k}");
        assert!(view.from_jd_ut <= row.sankranti.jd_ut && row.sankranti.jd_ut < view.to_jd_ut);
    }
    let first = local_date(view.rows[0].sankranti.jd_ut, view.rows[0].offset_secs);
    assert!(
        ("2026-01-13".."2026-01-17").contains(&first.as_str()),
        "{first}"
    );
    // Ardışık girişler 29–32 gün arayla gelir (Güneş'in hızı yıl içinde değişir).
    for w in view.rows.windows(2) {
        let gap = w[1].sankranti.jd_ut - w[0].sankranti.jd_ut;
        assert!((29.0..32.0).contains(&gap), "{gap}");
    }

    // İkinci yazım 1: ephemeris'in sidereal Güneş boylamı giriş anında burcun başındadır.
    let moments: Vec<(f64, Sign)> = view
        .rows
        .iter()
        .map(|r| (r.sankranti.jd_ut, r.sankranti.sign))
        .collect();
    let lons = {
        let moments = moments.clone();
        engine
            .call(move |se: &mut SwissEph| {
                moments
                    .iter()
                    .map(|(jd, _)| {
                        se.body(*jd, Body::Sun, Zodiac::Sidereal(SiderealMode::Lahiri))
                            .map(|p| p.lon)
                    })
                    .collect::<Result<Vec<f64>, _>>()
            })
            .expect("engine")
            .expect("sun")
    };
    for ((_, sign), lon) in moments.iter().zip(&lons) {
        let start = 30.0 * sign.index() as f64;
        let off = (lon - start + 180.0).rem_euclid(360.0) - 180.0;
        assert!(off.abs() < 1e-5, "{sign:?}: {off}");
    }

    // İkinci yazım 2: olay aramasının Güneş burç girişleri (sidereal kipte rasi çerçevesi
    // sidereal çerçevedir) aynı anları verir.
    let spec = EventSpec {
        from_jd_ut: view.from_jd_ut,
        to_jd_ut: view.to_jd_ut,
        kinds: vec![SkyEventKind::SignIngress],
        grahas: vec![Graha::Sun],
    };
    let events = {
        let (spec, settings) = (spec.clone(), settings.clone());
        engine
            .call(move |se: &mut SwissEph| {
                let mut out = Vec::new();
                for chunk in event_chunks(&spec) {
                    out.extend(event_search_chunk(se, &settings, &spec, None, chunk)?);
                }
                Ok::<_, jyotish_core::CoreError>(out)
            })
            .expect("engine")
            .expect("events")
    };
    assert_eq!(events.len(), 12);
    for (event, (jd, sign)) in events.iter().zip(&moments) {
        assert_eq!(event.sign, Some(*sign));
        assert!((event.jd_ut - jd).abs() < 2.0 / 86_400.0, "{sign:?}");
    }

    // Giriş anının tithisi, varası ve Ay nakshatrası v1'in o an ve yer için hesapladığı
    // haritanın panchangıdır.
    for row in &view.rows {
        let s = &row.sankranti;
        let chart = compute_chart_blocking(&engine, s.jd_ut, 28.6139, 77.2090, settings.clone())
            .expect("chart");
        let p = &chart.panchang;
        assert_eq!((s.tithi, s.paksha), (p.tithi.no, p.tithi.paksha));
        assert_eq!(s.vara, p.vara);
        assert_eq!((s.nakshatra, s.pada), (p.nakshatra.no, p.nakshatra.pada));
        assert_eq!(
            (s.nakshatra, s.pada),
            (chart.grahas[1].nakshatra, chart.grahas[1].pada)
        );
    }

    // Liste zodyak kipine bakmaz: tropikal-rasi kipinde de sidereal girişlerdir.
    let tropical = CalcSettings {
        zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
        ..settings.clone()
    };
    let tropical_view = list(&engine, Some(2026), delhi(), tropical);
    assert_eq!(tropical_view, view);

    // Yıl verilmezse yerin bugünkü yılıdır. 2026-12-31 19:00 UT Delhi'de 1 Ocak 2027 00:30'dur.
    assert_eq!(list(&engine, None, delhi(), settings.clone()).year, 2026);
    let new_year = Timestamp::from_second(1_798_743_600).expect("timestamp");
    let next =
        sankranti_list_blocking(&engine, None, delhi(), settings.clone(), new_year).expect("list");
    assert_eq!(next.year, 2027);

    // Kutup gecesi: Yay girişi (Aralık ortası) Tromsø'da gün doğmayan bir güne düşer.
    let polar = list(&engine, Some(2026), tromso(), settings.clone());
    assert_eq!(polar.rows.len(), 12);
    assert_eq!(polar.warnings, [ChartWarning::SunriseUnavailable]);
    // Anlar yere bağlı değildir; yılın uçları yerin dilimine göre kayar.
    for (a, b) in polar.rows.iter().zip(&view.rows) {
        assert_eq!(a.sankranti.sign, b.sankranti.sign);
        assert!((a.sankranti.jd_ut - b.sankranti.jd_ut).abs() < 1e-6);
    }

    // Hatalar yalnızca alan adını taşır; yer hataya girmez.
    let bad_place = MuhurtaPlace {
        lat: 95.0,
        ..delhi()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), bad_place, settings.clone(), now())
        .unwrap_err();
    assert_eq!(e.code, "invalid_transit_input");
    assert_eq!(e.params.len(), 1);
    assert_eq!(e.params["field"], "place");
    let bad_zone = MuhurtaPlace {
        tz: TzChoice::Iana("Mars/Olympus".into()),
        ..delhi()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), bad_zone, settings.clone(), now())
        .unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.params["field"].as_str()),
        ("invalid_transit_input", "place")
    );
    for year in [0, -44, 3000, i32::MAX] {
        let e = sankranti_list_blocking(&engine, Some(year), delhi(), settings.clone(), now())
            .unwrap_err();
        assert_eq!(e.code, "invalid_transit_input", "{year}");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "year");
    }
    let old = CalcSettings {
        schema_version: 1,
        ..settings.clone()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), delhi(), old, now()).unwrap_err();
    assert_eq!(e.code, "unsupported_schema_version");

    // Fikstür: Delhi 2026 ve Tromsø 2026 (gün doğumu uyarısıyla).
    let fresh = json!({ "place": delhi(), "list": view, "list_polar": polar });
    let path = root.join("../ui/test/fixtures/sankranti.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        let text = serde_json::to_string(&fresh).expect("serialize fixture") + "\n";
        std::fs::write(&path, text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/sankranti.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir (`ui_fixture.rs`
/// ile aynı kural).
fn approx_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(1.0),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| approx_eq(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| approx_eq(v, w)))
        }
        _ => a == b,
    }
}
