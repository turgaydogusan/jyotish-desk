//! `open_client` uçtan uca: gerçek `Engine` ve dosyadaki store. Spec §8 "Danışan açma":
//! gidiş-dönüş (kaydet → kapat → aç: ayarlar, düzen, dasha yolu ve notlar birebir) ve
//! performans (Rust tarafında ≤ 50 ms). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ephemeris::Engine;
use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, SignDashaPath};
use jyotish_desk_lib::client_commands::{
    add_chart_blocking, create_client_blocking, keep_stored_moment_blocking, open_client_blocking,
    restore_session_blocking, save_calc_settings_blocking, update_chart_blocking, DB_FILE_NAME,
};
use serde_json::{json, Map};
use store::{
    AppPrefs, BirthData, ChartKind, Layout, NewChart, NewClient, Panel, PanelLayout, Place, Store,
    TimeCertainty, TzChoice, ViewState,
};

/// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0.
fn delhi() -> BirthData {
    BirthData {
        date: "2000-01-01".into(),
        time: Some("17:30".into()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: "New Delhi".into(),
            country: "India".into(),
            admin1: "NCT".into(),
            lat: 28.6139,
            lon: 77.2090,
            geoname_id: Some(1_261_481),
        },
        tz: TzChoice::Iana("Asia/Kolkata".into()),
        fold: None,
        notes: String::new(),
    }
}

fn layout() -> PanelLayout {
    let mut settings = Map::new();
    settings.insert("varga".into(), json!("D9"));
    settings.insert("style".into(), json!("south"));
    PanelLayout {
        active_layout_id: "user:1".into(),
        layouts: vec![Layout {
            id: "user:1".into(),
            name: "Seans".into(),
            builtin: false,
            panels: vec![Panel {
                id: "d9".into(),
                kind: "chart".into(),
                x: 0,
                y: 0,
                w: 6,
                h: 12,
                settings,
            }],
        }],
    }
}

#[test]
fn open_client_round_trips_and_is_fast() {
    let ephe = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
    let engine = Engine::spawn(ephe).expect("spawn engine");
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(DB_FILE_NAME);
    let store = Mutex::new(Store::open(&path).expect("open store"));

    let created = create_client_blocking(
        &engine,
        &store,
        NewClient {
            name: "Ayşe".into(),
            gender: None,
            chart: NewChart {
                label: String::new(),
                kind: ChartKind::Natal,
                birth: delhi(),
            },
        },
    )
    .expect("create");
    assert_eq!(created.chart.jd_ut, 2_451_545.0);
    let sun = &created.chart.grahas[0];
    assert!(
        (sun.lon - 256.515_696_0).abs() * 3600.0 <= 2.0,
        "sun {}",
        sun.lon
    );
    assert_eq!(created.calc_settings, CalcSettings::default());
    let (client_id, chart_id) = (created.client.id, created.chart_record.id);

    // Ayar değişikliği yeniden hesaplanır ve kalıcıdır.
    let raman = CalcSettings {
        ayanamsa: AyanamsaSystem::Raman,
        ..CalcSettings::default()
    };
    let recomputed = save_calc_settings_blocking(&engine, &store, chart_id, raman.clone())
        .expect("save settings");
    assert_eq!(recomputed.meta.settings, raman);
    assert!(recomputed.ayanamsa_deg < created.chart.ayanamsa_deg);

    let vs = ViewState {
        panel_layout: Some(layout()),
        dasha_expanded_path: vec![Graha::Rahu, Graha::Jupiter],
        chara_expanded_path: SignDashaPath {
            cycle: 1,
            signs: vec![Sign::Pisces, Sign::Virgo],
        },
        ..ViewState::default()
    };
    {
        let mut s = store.lock().unwrap();
        s.save_view_state(chart_id, &vs).unwrap();
        s.set_chart_tags(chart_id, &["araştırma".into()]).unwrap();
        s.add_note(client_id, Some(chart_id), "ilk seans").unwrap();
    }
    let event = add_chart_blocking(
        &engine,
        &store,
        client_id,
        NewChart {
            label: "Evlilik".into(),
            kind: ChartKind::Event,
            birth: BirthData {
                date: "2016-06-01".into(),
                ..delhi()
            },
        },
    )
    .expect("add chart");
    assert_eq!(event.chart_record.kind, ChartKind::Event);
    // Doğum haritasına dönüş: son açılan o olur.
    // Zaman damgaları milisaniye çözünürlüklüdür; eşitlikte büyük kimlik (olay haritası)
    // kazanırdı ve test aralıklı düşerdi. Yeniden açılışı ayrı bir milisaniyeye it.
    std::thread::sleep(Duration::from_millis(5));
    let before = open_client_blocking(&engine, &store, client_id, Some(chart_id)).expect("reopen");

    // Genel varsayılan (spec §4 "Ayar katmanları") da yeniden başlatmadan sağ çıkar.
    let prefs = AppPrefs {
        locale: "en".into(),
        calc_settings: raman.clone(),
        ..AppPrefs::default()
    };
    store.lock().unwrap().set_app_prefs(&prefs).unwrap();

    // Kapat ve yeniden aç (uygulama yeniden başlatma).
    drop(store);
    let store = Mutex::new(Store::open(&path).expect("reopen store"));
    let after = restore_session_blocking(&engine, &store)
        .expect("restore")
        .expect("a session");
    assert_eq!(after.client.id, client_id);
    assert_eq!(after.chart_record.id, chart_id);
    assert_eq!(after.calc_settings, raman);
    assert_eq!(after.view_state, vs);
    assert_eq!(after.chart_record.tags, ["araştırma"]);
    assert_eq!(after.chart_record.birth, before.chart_record.birth);
    assert_eq!(after.recent_notes, before.recent_notes);
    assert_eq!(after.recent_notes[0].body, "ilk seans");
    assert_eq!(after.chart, before.chart);
    assert!(after.warnings.is_empty());
    let load = store.lock().unwrap().app_prefs_load().unwrap();
    assert_eq!(load.prefs, prefs);
    assert!(load.warnings.is_empty());
    // "Saklanan zamanı koru" haritayı yeniden açar; an ve harita değişmez.
    let kept = keep_stored_moment_blocking(&engine, &store, chart_id).expect("keep");
    assert_eq!(
        kept.chart_record.resolved.utc,
        after.chart_record.resolved.utc
    );
    assert_eq!(kept.chart, after.chart);

    // Düzenle kipi: DST boşluğu kaydı engeller, harita değişmez.
    let gap = update_chart_blocking(
        &engine,
        &store,
        chart_id,
        NewChart {
            label: String::new(),
            kind: ChartKind::Natal,
            birth: BirthData {
                date: "2014-03-31".into(),
                time: Some("03:30".into()),
                tz: TzChoice::Iana("Europe/Istanbul".into()),
                ..delhi()
            },
        },
    )
    .unwrap_err();
    assert_eq!(gap.code, "nonexistent_local_time");
    assert_eq!(gap.params["suggestion"], "2014-03-31T04:30:00");
    // Hata kaydı engeller: harita, doğum verisi ve çözülmüş an çağrıdan öncekiyle aynı.
    let unchanged =
        open_client_blocking(&engine, &store, client_id, Some(chart_id)).expect("reopen after gap");
    assert_eq!(unchanged.chart_record.birth, after.chart_record.birth);
    assert_eq!(
        unchanged.chart_record.resolved.jd_ut,
        after.chart_record.resolved.jd_ut
    );
    assert_eq!(unchanged.chart, after.chart);

    // Arayüzün otomatik kaydı (`save_view_state`) oturum geri yüklemesinden sağ çıkar.
    let vs2 = ViewState {
        dasha_expanded_path: vec![Graha::Saturn, Graha::Mercury, Graha::Ketu],
        ..vs.clone()
    };
    store
        .lock()
        .unwrap()
        .save_view_state(chart_id, &vs2)
        .expect("save view state");
    let restored = restore_session_blocking(&engine, &store)
        .expect("restore")
        .expect("a session");
    assert_eq!(
        restored.view_state.dasha_expanded_path,
        vs2.dasha_expanded_path
    );
    assert_eq!(
        restored.view_state.chara_expanded_path,
        vs.chara_expanded_path
    );
    assert_eq!(restored.view_state.panel_layout, vs2.panel_layout);

    // Harita olmadan düzen kaydı (`save_panel_layout`/`load_panel_layout` gövdeleri).
    let alone = PanelLayout {
        active_layout_id: "user:2".into(),
        ..layout()
    };
    store
        .lock()
        .unwrap()
        .save_panel_layout(Some(&alone))
        .expect("save layout");
    assert_eq!(store.lock().unwrap().panel_layout().unwrap(), Some(alone));
    store
        .lock()
        .unwrap()
        .save_panel_layout(None)
        .expect("clear layout");
    assert_eq!(store.lock().unwrap().panel_layout().unwrap(), None);
    let err = open_client_blocking(&engine, &store, 999, None).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.params["what"].as_str()),
        ("not_found", "client")
    );

    // Performans: spec §4 "Rust tarafında ≤ 50 ms". Ortanca, ısınmadan sonra.
    open_client_blocking(&engine, &store, client_id, None).expect("warm-up");
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            open_client_blocking(&engine, &store, client_id, None).expect("open");
            t.elapsed()
        })
        .collect();
    times.sort();
    let median = times[times.len() / 2];
    eprintln!(
        "open_client median {median:?}, max {:?}",
        times[times.len() - 1]
    );
    assert!(
        median <= Duration::from_millis(50),
        "open_client median {median:?}"
    );
}
