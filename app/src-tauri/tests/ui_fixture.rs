//! Arayüz testlerinin (vitest ve Playwright) gerçek veri fikstürü:
//! `app/ui/test/fixtures/delhi.json`. Gerçek motor, store ve atlasla üretilir; saat sabit,
//! UUID'ler sabit dizgiyle değiştirilir. Arayüz kendi haritasını hesaplamadığı için
//! (spec §4) testler bu dosyadaki `OpenedClient`'ı ve modül sonuçlarını çizer.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`.
//! Denetim sayılarda göreli 1e-9 toleranslıdır: CI derleyicisi ve libm son basamakta
//! farklı sonuç verebilir. Motoru tek test açar: süreçte yalnızca bir SE örneği olabilir.

use std::path::PathBuf;
use std::sync::Mutex;

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::muhurta::{NakshatraNature, TithiGroup, MUHURTA_MAX_MIN_MINUTES};
use jyotish_core::{
    kuta_class_ids, ChartInput, ChartQuery, Condition, Graha, GrahaFlag, Kuta, LordLink,
    MuhurtaCriteria, MuhurtaFlag, NatalMoon, QueryCondition, QueryScope, RisingMode, SkyEventKind,
    Vara, Varga, EIGHT_KARAKA_ROLES, MAX_CONDITIONS, MAX_SCOPE_TAGS, MAX_SEARCH_DASHA_LEVEL,
    MAX_SKY_EVENTS, MAX_YOGA_IDS, PRASHNA_NUMBER_MAX, YOGA_CATALOG, YOGA_CONDITIONS,
};
use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
use jyotish_desk_lib::client_commands::create_client_blocking;
use jyotish_desk_lib::commands::dasha_children;
use jyotish_desk_lib::module_commands::{
    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
    compute_compat_blocking, compute_drishti_blocking, compute_jaimini_blocking,
    compute_shadbala_blocking, compute_yogas_blocking,
};
use jyotish_desk_lib::muhurta_commands::{
    day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery,
};
use jyotish_desk_lib::prashna_commands::{
    compute_prashna_facts_blocking, create_prashna_blocking, PrashnaMoment, PrashnaRequest,
};
use jyotish_desk_lib::search_commands::{
    chart_search_blocking, event_search_blocking, EventSearchSpec, SearchRegistry,
};
use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
use serde_json::{json, Value};
use store::{
    BirthData, ChartKind, Gender, HomePlace, NewChart, NewClient, Place, Store, TimeCertainty,
    TzChoice, MAX_QUESTION_CHARS,
};

const CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000001";
const CHART_UUID: &str = "00000000-0000-4000-8000-000000000002";
/// Uyum fikstürünün eş haritası (ikinci danışan).
const PARTNER_CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000011";
const PARTNER_CHART_UUID: &str = "00000000-0000-4000-8000-000000000012";
/// Prashna fikstürünün iki haritası (zaman ve sayı prashnası).
const PRASHNA_CHART_UUID: &str = "00000000-0000-4000-8000-000000000021";
const PRASHNA_NUMBER_CHART_UUID: &str = "00000000-0000-4000-8000-000000000022";

/// Eş harita: İstanbul, 1987-03-17 10:00 (deneme verisi; gerçek kişi değildir).
fn istanbul() -> BirthData {
    BirthData {
        date: "1987-03-17".into(),
        time: Some("10:00:00".into()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: "İstanbul".into(),
            country: "Türkiye".into(),
            admin1: "İstanbul".into(),
            lat: 41.01384,
            lon: 28.94966,
            geoname_id: Some(745_044),
        },
        tz: TzChoice::Iana("Europe/Istanbul".into()),
        fold: None,
        notes: String::new(),
    }
}

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
        lagna_number: None,
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
    let chara_at = chara_dasha_at_blocking(&engine, module_input.clone(), transit.snapshot.jd_ut)
        .expect("chara dasha at")
        .expect("inside the produced span");
    // Transit anındaki alt dönemin 3. seviyesi: dasa şeridinin "pratyantar" görünümü.
    let chara_at_children = chara_dasha_children_blocking(
        &engine,
        module_input.clone(),
        chara_at.signs[..2].to_vec(),
        chara_at.cycle,
    )
    .expect("chara dasha children at the transit moment");
    // Shadbala ve Bhava bala: aynı girdiyle (doğum saati biliniyor, Güneş doğup batıyor).
    let shadbala = compute_shadbala_blocking(&engine, module_input.clone())
        .expect("shadbala")
        .expect("time known and a sunrise");
    // Yogalar: aynı girdiyle ve doğum saati bilinmeyen girdiyle (lagna isteyen kurallar atlanır).
    // Katalog (kimlik ve grup) ve alt koşul kimlikleri: arayüz testleri her birinin üç dilde
    // adı olduğunu bu listeden denetler.
    let yogas = compute_yogas_blocking(&engine, module_input.clone()).expect("yogas");
    let yogas_time_unknown = compute_yogas_blocking(
        &engine,
        ChartInput {
            time_known: false,
            lagna_number: None,
            ..module_input.clone()
        },
    )
    .expect("yogas without a birth time");
    let yoga_catalog: Vec<Value> = YOGA_CATALOG
        .iter()
        .map(|r| json!({ "id": r.id, "group": r.group }))
        .collect();
    // Uyum: ikinci danışan eş haritadır. Eşin girdisi ve kısa kimliği store'dan okunur
    // (arayüzün `chart_input` ve `chart_brief` ile aldığının aynısı). Üç rapor: açık harita
    // damat rolünde, roller değişmiş, ve eşin doğum saati bilinmiyor (gelin rolünde). Sınıf
    // kimlikleri: arayüz testleri her birinin üç dilde adı olduğunu bu listeden denetler.
    let partner = create_client_blocking(
        &engine,
        &store,
        NewClient {
            name: "Eş Danışan".into(),
            gender: Some(Gender::Female),
            chart: NewChart {
                label: String::new(),
                kind: ChartKind::Natal,
                birth: istanbul(),
            },
        },
    )
    .expect("create the partner");
    let (partner_input, partner_brief) = {
        let s = store.lock().expect("store");
        (
            s.chart_input(&partner.chart_record.uuid)
                .expect("chart input")
                .expect("partner chart"),
            s.chart_brief(&partner.chart_record.uuid)
                .expect("chart brief")
                .expect("partner chart"),
        )
    };
    let compat = compute_compat_blocking(&engine, module_input.clone(), partner_input.clone())
        .expect("compat");
    let compat_swapped =
        compute_compat_blocking(&engine, partner_input.clone(), module_input.clone())
            .expect("compat with swapped roles");
    let compat_time_unknown = compute_compat_blocking(
        &engine,
        module_input.clone(),
        ChartInput {
            time_known: false,
            lagna_number: None,
            ..partner_input.clone()
        },
    )
    .expect("compat with an unknown birth time");
    let kuta_classes: serde_json::Map<String, Value> = Kuta::ALL
        .into_iter()
        .map(|k| (format!("{k:?}"), json!(kuta_class_ids(k))))
        .collect();
    // Muhurta: açık haritanın doğum yeri için "şimdi"yi içeren gün (doğum Ay'ıyla), kutup
    // gecesinde bir gün (dilimler hesaplanmaz) ve üç dilimlik bir arama (67 gün; Perşembe
    // gündüzleri, üç işaretli dilim dışlanmış). İşaret ve sınıf kimlikleri: arayüz testleri her
    // birinin üç dilde adı olduğunu bu listeden denetler.
    let fixture_now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let moon = &opened.chart.grahas[Graha::Moon as usize];
    let natal = NatalMoon {
        nakshatra: moon.nakshatra,
        sign: moon.sign,
    };
    let birth = &opened.chart_record.birth;
    let muhurta_place = MuhurtaPlace {
        lat: birth.place.lat,
        lon: birth.place.lon,
        tz: birth.tz.clone(),
    };
    let muhurta_day = day_panchang_blocking(
        &engine,
        TransitAt::Now,
        fixture_now,
        muhurta_place.clone(),
        opened.calc_settings.clone(),
        Some(natal),
    )
    .expect("day panchang");
    // tests/golden/tromso-2050-12-21.json anı: Güneş doğmaz.
    let muhurta_day_polar = day_panchang_blocking(
        &engine,
        TransitAt::Jd(2_470_162.260_416_666_5),
        fixture_now,
        MuhurtaPlace {
            lat: 69.6492,
            lon: 18.9553,
            tz: TzChoice::Iana("Europe/Oslo".into()),
        },
        opened.calc_settings.clone(),
        None,
    )
    .expect("day panchang in the polar night");
    let muhurta_query = MuhurtaQuery {
        from_date: Some("2026-11-26".into()),
        to_date: Some("2027-01-31".into()),
        place: muhurta_place.clone(),
        criteria: MuhurtaCriteria {
            weekdays: vec![Vara::Thursday],
            daytime_only: true,
            min_minutes: 30,
            exclude: vec![
                MuhurtaFlag::RahuSegment,
                MuhurtaFlag::Yamaganda,
                MuhurtaFlag::Gulika,
            ],
            ..MuhurtaCriteria::default()
        },
        natal: Some(natal),
    };
    let mut muhurta_pages = Vec::new();
    let mut cursor = None;
    loop {
        let page = muhurta_search_blocking(
            &engine,
            muhurta_query.clone(),
            opened.calc_settings.clone(),
            cursor,
            fixture_now,
        )
        .expect("muhurta search");
        cursor = page.next;
        muhurta_pages.push(page);
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(muhurta_pages.len(), 3, "sixty-seven days are three chunks");
    let home_place = HomePlace {
        place: birth.place.clone(),
        tz: birth.tz.clone(),
    };
    // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
    assert_eq!(opened.chart_record.birth, delhi());
    let prefs = store
        .lock()
        .expect("store")
        .app_prefs_load()
        .expect("prefs");

    // Prashna: fikstürün sabit saatinde ("şimdi"), açık danışanın altında, bulunulan yerde
    // (Delhi) iki harita: zaman prashnası ve 54 sayısıyla sayı prashnası (deneme sorusu; gerçek
    // kişi ya da soru değildir). Olgular odak evi 7 ile; bir de odak evi seçilmemiş hâli.
    // Öteki bütün okumalardan sonra kurulur: önceki kayıtların kimlikleri değişmez.
    let prashna_now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let prashna_request = |number: Option<u8>| PrashnaRequest {
        client_id: Some(opened.client.id),
        at: PrashnaMoment::Now,
        place: home_place.clone(),
        number,
        question: "Deneme sorusu: yolculuk bu ay olur mu?".into(),
        focus_house: Some(7),
    };
    let prashna_time = create_prashna_blocking(&engine, &store, prashna_request(None), prashna_now)
        .expect("time prashna");
    let prashna_number =
        create_prashna_blocking(&engine, &store, prashna_request(Some(54)), prashna_now)
            .expect("number prashna");
    let prashna_input = |o: &jyotish_desk_lib::client_commands::OpenedClient| ChartInput {
        jd_ut: o.chart.jd_ut,
        lat: o.chart.geo_lat,
        lon: o.chart.geo_lon,
        settings: o.calc_settings.clone(),
        time_known: true,
        lagna_number: o.prashna.as_ref().and_then(|p| p.number),
    };
    let prashna_facts =
        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_time), Some(7))
            .expect("prashna facts");
    let prashna_facts_no_focus =
        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_time), None)
            .expect("prashna facts without a focus house");
    let prashna_facts_number =
        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_number), Some(7))
            .expect("number prashna facts");
    assert_eq!(
        prashna_number.chart.lagna.sign,
        jyotish_core::Sign::Virgo,
        "54 is the sixth sign"
    );
    // Arama (Plan 22): fikstür veritabanındaki dört haritada bir harita araması, bir kayıtlı
    // arama ve açık haritayla bir olay araması.
    let registry = SearchRegistry::default();
    let search_query = ChartQuery {
        all_of: vec![
            QueryCondition {
                cond: Condition::GrahaInSign {
                    graha: Graha::Moon,
                    signs: vec![moon.sign],
                    varga: Varga::D1,
                },
                negate: false,
            },
            QueryCondition {
                cond: Condition::GrahaInHouse {
                    graha: Graha::Sun,
                    houses: vec![1, 10],
                },
                negate: true,
            },
        ],
        scope: QueryScope::default(),
    };
    let search_page = chart_search_blocking(
        &engine,
        &store,
        &registry,
        search_query.clone(),
        1,
        None,
        fixture_now,
    )
    .expect("chart search");
    assert!(search_page.done, "the fixture database is one page");
    assert_eq!(
        search_page.total, 2,
        "prashna charts are out of the default scope"
    );
    assert_eq!(
        search_page.hits.len(),
        1,
        "only the open chart has its own Moon sign"
    );
    let search_all_charts = chart_search_blocking(
        &engine,
        &store,
        &registry,
        ChartQuery {
            scope: QueryScope {
                exclude_prashna: false,
                ..QueryScope::default()
            },
            ..search_query.clone()
        },
        2,
        None,
        fixture_now,
    )
    .expect("chart search with prashna charts");
    assert_eq!(search_all_charts.total, 4);
    let saved_search = store
        .lock()
        .expect("store")
        .save_search("Ay burcu, Güneş 1. ya da 10. evde değil", &search_query)
        .expect("save search");
    let event_spec = EventSearchSpec {
        from_date: Some("2026-11-26".into()),
        to_date: Some("2027-03-31".into()),
        place: muhurta_place.clone(),
        kinds: vec![
            SkyEventKind::SignIngress,
            SkyEventKind::StationRetrograde,
            SkyEventKind::StationDirect,
            SkyEventKind::Conjunction,
            SkyEventKind::NewMoon,
            SkyEventKind::FullMoon,
            SkyEventKind::TransitToNatal,
            SkyEventKind::HouseFromNatalMoon,
        ],
        grahas: vec![Graha::Sun, Graha::Mars, Graha::Jupiter, Graha::Saturn],
        settings: opened.calc_settings.clone(),
        natal: Some(module_input.clone()),
    };
    let event_view = event_search_blocking(&engine, &registry, event_spec.clone(), 1, fixture_now)
        .expect("event search");
    assert!(!event_view.truncated);
    for kind in [
        SkyEventKind::SignIngress,
        SkyEventKind::NewMoon,
        SkyEventKind::FullMoon,
        SkyEventKind::TransitToNatal,
    ] {
        assert!(
            event_view.events.iter().any(|e| e.event.kind == kind),
            "the fixture range has a {kind:?}"
        );
    }
    // Bhava tablosu ve Sudarshana chakra (Plan 25). Açık haritanın girdisiyle (tam burç) tablo ve
    // chakra; aynı an ve yer için Sripati tablosu (bhava ortalarıyla) ile Placidus haritası ve
    // tablosu (chalit görünümünün verisi: üç graha burç evinden başka bir bhavadadır); kutup
    // dairesinde Placidus (Porphyry'ye düşer; tests/golden/tromso-2050-12-21.json anı); doğum
    // saati bilinmeyen girdiyle chakra (lagna halkası yok).
    let bhava = {
        use jyotish_core::{CalcSettings, HouseSystem};
        use jyotish_desk_lib::bhava_commands::{
            compute_bhava_table_blocking, compute_sudarshana_blocking,
        };
        use jyotish_desk_lib::commands::compute_chart_for_blocking;
        let with_system = |system: HouseSystem| ChartInput {
            settings: CalcSettings {
                house_system: system,
                ..module_input.settings.clone()
            },
            ..module_input.clone()
        };
        let table_of = |input: ChartInput| {
            compute_bhava_table_blocking(&engine, input)
                .expect("bhava table")
                .expect("time known")
        };
        let table = table_of(module_input.clone());
        let table_sripati = table_of(with_system(HouseSystem::Sripati));
        let placidus = with_system(HouseSystem::Placidus);
        let chart_placidus =
            compute_chart_for_blocking(&engine, placidus.clone()).expect("placidus chart");
        let table_placidus = table_of(placidus.clone());
        assert_eq!(
            table_placidus.grahas.iter().filter(|g| g.shifted).count(),
            3,
            "three grahas of the fixture chart are outside their sign house"
        );
        let table_placidus_polar = table_of(ChartInput {
            jd_ut: 2_470_162.260_416_666_5,
            lat: 69.6492,
            lon: 18.9553,
            ..placidus
        });
        assert!(table_placidus_polar.porphyry_fallback);
        let sudarshana =
            compute_sudarshana_blocking(&engine, module_input.clone()).expect("sudarshana");
        let sudarshana_time_unknown = compute_sudarshana_blocking(
            &engine,
            ChartInput {
                time_known: false,
                ..module_input.clone()
            },
        )
        .expect("sudarshana without a birth time");
        json!({
            "chart_placidus": chart_placidus,
            "table": table,
            "table_sripati": table_sripati,
            "table_placidus": table_placidus,
            "table_placidus_polar": table_placidus_polar,
            "sudarshana": sudarshana,
            "sudarshana_time_unknown": sudarshana_time_unknown,
            "sudarshana_refs": jyotish_core::SudarshanaRef::ALL,
        })
    };
    let value = json!({
        "opened": opened,
        "bhava": bhava,
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
        "chara_dasha_at_children": chara_at_children,
        "shadbala": shadbala,
        "yogas": yogas,
        "yogas_time_unknown": yogas_time_unknown,
        "yoga_catalog": yoga_catalog,
        "yoga_conditions": YOGA_CONDITIONS,
        "compat": {
            "partner_uuid": partner.chart_record.uuid,
            "brief": partner_brief,
            "input": partner_input,
            "report": compat,
            "report_swapped": compat_swapped,
            "report_time_unknown": compat_time_unknown,
        },
        "kuta_classes": kuta_classes,
        "muhurta": {
            "natal": natal,
            "place": muhurta_place,
            "home_place": home_place,
            "day": muhurta_day,
            "day_polar": muhurta_day_polar,
            "query": muhurta_query,
            "pages": muhurta_pages,
        },
        "prashna": {
            "opened": prashna_time,
            "opened_number": prashna_number,
            "facts": prashna_facts,
            "facts_no_focus": prashna_facts_no_focus,
            "facts_number": prashna_facts_number,
        },
        "prashna_ids": {
            "rising": RisingMode::ALL,
            "links": LordLink::ALL,
            "max_number": PRASHNA_NUMBER_MAX,
            "max_question_chars": MAX_QUESTION_CHARS,
        },
        "search": {
            "query": search_query,
            "page": search_page,
            "page_all_charts": search_all_charts,
            "saved": saved_search,
            "event_spec": event_spec,
            "events": event_view,
        },
        "search_ids": {
            "event_kinds": SkyEventKind::ALL,
            "flags": GrahaFlag::ALL,
            "vargas": Varga::ALL,
            "karaka_roles": EIGHT_KARAKA_ROLES,
            "max_conditions": MAX_CONDITIONS,
            "max_scope_tags": MAX_SCOPE_TAGS,
            "max_yoga_ids": MAX_YOGA_IDS,
            "max_dasha_level": MAX_SEARCH_DASHA_LEVEL,
            "max_events": MAX_SKY_EVENTS,
        },
        "muhurta_ids": {
            "flags": MuhurtaFlag::ALL,
            "tithi_groups": TithiGroup::ALL,
            "nakshatra_natures": NakshatraNature::ALL,
            "max_min_minutes": MUHURTA_MAX_MIN_MINUTES,
        },
    });
    let text = serde_json::to_string(&value)
        .expect("serialize")
        .replace(&opened.client.uuid, CLIENT_UUID)
        .replace(&opened.chart_record.uuid, CHART_UUID)
        .replace(&partner.client.uuid, PARTNER_CLIENT_UUID)
        .replace(&partner.chart_record.uuid, PARTNER_CHART_UUID)
        .replace(&prashna_time.chart_record.uuid, PRASHNA_CHART_UUID)
        .replace(&prashna_number.chart_record.uuid, PRASHNA_NUMBER_CHART_UUID)
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
