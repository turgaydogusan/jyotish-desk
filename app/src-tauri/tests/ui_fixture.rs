//! Arayüz testlerinin (vitest ve Playwright) gerçek veri fikstürü:
//! `app/ui/test/fixtures/delhi.json`. Gerçek motor, store ve atlasla üretilir; saat sabit,
//! UUID'ler sabit dizgiyle değiştirilir. Arayüz kendi haritasını hesaplamadığı için
//! (spec §4) testler bu dosyadaki `OpenedClient`'ı çizer.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`.
//! Denetim sayılarda göreli 1e-9 toleranslıdır: CI derleyicisi ve libm son basamakta
//! farklı sonuç verebilir. Motoru tek test açar: süreçte yalnızca bir SE örneği olabilir.

use std::path::PathBuf;
use std::sync::Mutex;

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::ChartInput;
use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
use jyotish_desk_lib::client_commands::create_client_blocking;
use jyotish_desk_lib::commands::dasha_children;
use jyotish_desk_lib::module_commands::{
    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
    compute_drishti_blocking, compute_jaimini_blocking,
};
use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
use serde_json::{json, Value};
use store::{BirthData, ChartKind, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice};

const CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000001";
const CHART_UUID: &str = "00000000-0000-4000-8000-000000000002";

/// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0.
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
fn ui_fixture_is_up_to_date() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let clock = Box::new(|| Timestamp::from_second(1_795_600_800).expect("timestamp"));
    let store = Mutex::new(Store::open_in_memory().expect("store").with_clock(clock));
    let opened = create_client_blocking(
        &engine,
        &store,
        NewClient {
            name: "Örnek Danışan".into(),
            gender: None,
            chart: NewChart {
                label: String::new(),
                kind: ChartKind::Natal,
                birth: delhi(),
            },
        },
    )
    .expect("create client");

    // İlk pratyantar dönemin (3. seviye) alt dönemleri: tembel yüklenen 4. seviye.
    let path = opened
        .chart
        .dasha
        .periods
        .iter()
        .find(|p| p.level == 3)
        .expect("level 3")
        .lords
        .clone();
    let children = dasha_children(opened.chart.dasha.seed, path.clone()).expect("children");

    let atlas = AtlasState::open(&root.join("../../data/atlas/atlas.sqlite"));
    let places = search_places_blocking(&atlas, "new delhi", 8).expect("places");
    let preview = preview_moment_body(&delhi()).expect("preview");
    let mut gap = delhi();
    gap.date = "2014-03-31".into();
    gap.time = Some("03:30".into());
    gap.tz = TzChoice::Iana("Europe/Istanbul".into());
    let gap_error = preview_moment_body(&gap).expect_err("DST gap");

    // Transit: sabit saat (fikstürün saati), "şimdi".
    let transit = transits_at_blocking(
        &engine,
        TransitAt::Now,
        Timestamp::from_second(1_795_600_800).expect("timestamp"),
        opened.calc_settings.clone(),
        opened.chart.bhavas.clone(),
        opened.chart.dasha.seed,
    )
    .expect("transit");
    // Modül sonuçları: arayüzün `chartInput(opened)` ile kurduğu girdinin aynısı.
    let module_input = ChartInput {
        jd_ut: opened.chart.jd_ut,
        lat: opened.chart.geo_lat,
        lon: opened.chart.geo_lon,
        settings: opened.chart.meta.settings.clone(),
        time_known: true,
    };
    let drishti = compute_drishti_blocking(&engine, module_input.clone()).expect("drishti");
    let jaimini = compute_jaimini_blocking(&engine, module_input.clone()).expect("jaimini");
    // Chara dasha: iki seviye, ilk alt dönemin 3. seviyesi (istenerek yüklenen seviye) ve
    // transit anının dönem yolu.
    let chara_dasha = compute_chara_dasha_blocking(&engine, module_input.clone())
        .expect("chara dasha")
        .expect("time known");
    let chara_path = chara_dasha.periods[1].signs.clone();
    let chara_children =
        chara_dasha_children_blocking(&engine, module_input.clone(), chara_path.clone(), 1)
            .expect("chara dasha children");
    let chara_at = chara_dasha_at_blocking(&engine, module_input, transit.snapshot.jd_ut)
        .expect("chara dasha at")
        .expect("inside the produced span");
    // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
    assert_eq!(opened.chart_record.birth, delhi());
    let prefs = store
        .lock()
        .expect("store")
        .app_prefs_load()
        .expect("prefs");
    let value = json!({
        "opened": opened,
        "dasha_children": { "path": path, "periods": children },
        "places": places,
        "preview": preview,
        "gap_error": gap_error,
        "transit": transit,
        "prefs": prefs,
        "drishti": drishti,
        "jaimini": jaimini,
        "chara_dasha": chara_dasha,
        "chara_dasha_children": { "path": chara_path, "cycle": 1, "periods": chara_children },
        "chara_dasha_at": chara_at,
    });
    let text = serde_json::to_string(&value)
        .expect("serialize")
        .replace(&opened.client.uuid, CLIENT_UUID)
        .replace(&opened.chart_record.uuid, CHART_UUID)
        + "\n";

    let path = root.join("../ui/test/fixtures/delhi.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().expect("fixture dir")).expect("create fixture dir");
        std::fs::write(&path, &text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    let fresh: Value = serde_json::from_str(&text).expect("parse fresh fixture");
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/delhi.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir.
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

#[test]
fn approx_eq_tolerates_last_digit_noise_only() {
    let a = json!({"lon": 256.515_696_012_345, "name": "Sun", "list": [1, 2.0, null]});
    let mut b = a.clone();
    b["lon"] = json!(256.515_696_012_346);
    assert!(approx_eq(&a, &b));
    b["lon"] = json!(256.515_7);
    assert!(!approx_eq(&a, &b));
    let mut c = a.clone();
    c["name"] = json!("Moon");
    assert!(!approx_eq(&a, &c));
    let mut d = a.clone();
    d["list"] = json!([1, 2.0]);
    assert!(!approx_eq(&a, &d));
    assert!(!approx_eq(&a, &Value::Null));
}
